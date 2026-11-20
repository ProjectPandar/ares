use std::{fs, io, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, SliceError,
    SlicedPlate, SlicedStlPlate, SlicerContext,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    read_preset_files, split_native_list,
};

pub(crate) mod arrange;

pub(super) async fn run(args: DirectSliceArgs) -> Result<(), CliFailure> {
    let selection = u32::try_from(
        args.selection
            .expect("--slice is required without a subcommand"),
    )
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a nonnegative --slice",
        )
    })?;
    let dir = args
        .outputdir
        .as_ref()
        .expect("--outputdir is required without a subcommand");
    let output = dir.join("plate_1.gcode");
    let multiple = args.inputs.len() > 1;
    let mut models = Vec::new();
    let mut single_input = Vec::new();
    let mut filename = "";
    for path in &args.inputs {
        if path.extension().is_none_or(|ext| ext != "stl") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected .stl input").into());
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
        if filename.is_empty() {
            filename = name;
        }
        let bytes = read_input(path, &output)?;
        if multiple || selection > 1 {
            let model = ares_core::load_model(&bytes).map_err(|_| {
                CliFailure::invalid_stl_input(
                    path,
                    #[cfg(target_os = "linux")]
                    &output,
                )
            })?;
            models.push((name, model));
        } else {
            single_input = bytes;
        }
    }
    let mut report = SlicedPlate::default();
    let plates = if args.load_settings.is_none() && args.load_filaments.is_none() && !multiple {
        if selection > 1 {
            Vec::new()
        } else {
            let gcode =
                ares_core::slice_cli_stl_with_defaults(&single_input, filename, &mut report)
                    .await
                    .map_err(|error| match error {
                        SliceError::OptionStlModelRead(_) => CliFailure::invalid_stl_input(
                            &args.inputs[0],
                            #[cfg(target_os = "linux")]
                            &output,
                        ),
                        error => slice_failure(error, &output),
                    })?;
            vec![SlicedStlPlate {
                id: 1,
                gcode,
                report,
            }]
        }
    } else {
        let settings = read_presets(args.load_settings.as_deref())?;
        let filaments = read_presets(args.load_filaments.as_deref())?;
        let request = LoadSettingsSliceRequest {
            filename,
            catalog_json: None,
            uptodate: false,
            settings: &preset_refs(&settings),
            filaments: &preset_refs(&filaments),
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: args.arrange.placement(args.inputs.len()),
            orient: false,
            sliced_plate: &mut report,
        };
        if models.is_empty() {
            let gcode = ares_core::slice_with_load_settings(
                &single_input,
                request,
                GenerationMetadata::default(),
                SlicerContext::new(0),
            )
            .await
            .map_err(|error| slice_failure(error, &output))?;
            vec![SlicedStlPlate {
                id: 1,
                gcode,
                report,
            }]
        } else {
            ares_core::slice_stl_models_with_load_settings(&models, request, selection)
                .await
                .map_err(|error| slice_failure(error, &output))?
        }
    };
    for plate in &plates {
        fs::write(dir.join(format!("plate_{}.gcode", plate.id)), &plate.gcode)?;
    }
    #[cfg(target_os = "linux")]
    crate::result_json::write_success(
        &output,
        selection,
        &[],
        plates.into_iter().map(|plate| (plate.id, plate.report)),
        Vec::new(),
    );
    Ok(())
}

fn read_input(path: &Path, output: &Path) -> Result<Vec<u8>, CliFailure> {
    #[cfg(not(target_os = "linux"))]
    let _ = output;
    match fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(CliFailure::missing_input(
            path,
            #[cfg(target_os = "linux")]
            output,
        )),
        Err(_) => Err(CliFailure::invalid_stl_input(
            path,
            #[cfg(target_os = "linux")]
            output,
        )),
    }
}

fn slice_failure(error: SliceError, output: &Path) -> CliFailure {
    slice_error_to_failure(
        error,
        output,
        #[cfg(target_os = "linux")]
        1,
    )
}

fn read_presets(value: Option<&str>) -> Result<Vec<(String, String)>, CliFailure> {
    read_preset_files(&value.map(split_native_list).unwrap_or_default())
}

fn preset_refs(files: &[(String, String)]) -> Vec<LoadSettingsFile<'_>> {
    files
        .iter()
        .map(|(label, json)| LoadSettingsFile { label, json })
        .collect()
}
