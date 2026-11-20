use std::{fs, io};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, SliceError,
    SlicedPlate, SlicerContext, StlFile,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    read_preset_files, split_native_list,
};

pub(crate) mod arrange;

pub(super) async fn run(args: DirectSliceArgs) -> Result<(), CliFailure> {
    let selection = args
        .selection
        .expect("--slice is required without a subcommand");
    if !matches!(selection, 0 | 1) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected --slice 0 or 1").into());
    }

    let output = args
        .outputdir
        .expect("--outputdir is required without a subcommand")
        .join("plate_1.gcode");
    let mut inputs = Vec::with_capacity(args.inputs.len());
    for path in &args.inputs {
        if path.extension().is_none_or(|ext| ext != "stl") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected .stl input").into());
        }
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(CliFailure::missing_input(
                    path,
                    #[cfg(target_os = "linux")]
                    &output,
                ));
            }
            Err(error) => return Err(error.into()),
        };
        inputs.push((filename, bytes));
    }
    let mut sliced_plate = SlicedPlate::default();
    let (filename, bytes) = &inputs[0];
    let gcode =
        if args.load_settings.is_none() && args.load_filaments.is_none() && inputs.len() == 1 {
            ares_core::slice_cli_stl_with_defaults(bytes, filename, &mut sliced_plate).await
        } else {
            let settings = read_preset_files(
                &args
                    .load_settings
                    .as_deref()
                    .map(split_native_list)
                    .unwrap_or_default(),
            )?;
            let filaments = read_preset_files(
                &args
                    .load_filaments
                    .as_deref()
                    .map(split_native_list)
                    .unwrap_or_default(),
            )?;
            let settings = settings
                .iter()
                .map(|(label, json)| LoadSettingsFile { label, json })
                .collect::<Vec<_>>();
            let filaments = filaments
                .iter()
                .map(|(label, json)| LoadSettingsFile { label, json })
                .collect::<Vec<_>>();
            let request = LoadSettingsSliceRequest {
                filename,
                catalog_json: None,
                uptodate: false,
                settings: &settings,
                filaments: &filaments,
                // Unlike 3MF, a fresh STL model never enters the selected-plate importer.
                // OrcaSlicer.cpp:2042–2045 resets the import selection for non-project files.
                plate: None,
                overrides: &ConfigOverrides::default(),
                placement: args.arrange.placement(inputs.len()),
                orient: false,
                sliced_plate: &mut sliced_plate,
            };
            let metadata = GenerationMetadata::default();
            let context = SlicerContext::new(0);
            if inputs.len() == 1 {
                ares_core::slice_with_load_settings(bytes, request, metadata, context).await
            } else {
                let files = inputs
                    .iter()
                    .map(|(name, bytes)| StlFile { name, bytes })
                    .collect::<Vec<_>>();
                ares_core::slice_stl_files_with_load_settings(&files, request, metadata, context)
                    .await
            }
        };
    let gcode = gcode.map_err(|error| match error {
        SliceError::OptionStlModelRead(_) => CliFailure::invalid_stl_input(
            &args.inputs[0],
            #[cfg(target_os = "linux")]
            &output,
        ),
        error => slice_error_to_failure(
            error,
            &output,
            #[cfg(target_os = "linux")]
            1,
        ),
    })?;
    fs::write(&output, gcode)?;
    #[cfg(target_os = "linux")]
    crate::result_json::Success {
        output: &output,
        plate: 1,
        plate_index: selection as u32,
        sliced_plate,
        upward_machines: &[],
    }
    .write(filename, Vec::new());
    Ok(())
}
