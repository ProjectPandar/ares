use std::{fs, io};

use ares_core::{
    ConfigOverrides, GenerationMetadata, ImportedStl, LoadSettingsFile, LoadSettingsSliceRequest,
    Placement, SliceError, SlicedPlate, SlicerContext, StlFile,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    preset_sources::read_preset_files,
    split_native_list,
};

pub(super) async fn run(args: DirectSliceArgs) -> Result<(), CliFailure> {
    if args.no_check {
        if args.strict {
            return Err(CliFailure::strict_no_check(
                #[cfg(target_os = "linux")]
                &args
                    .outputdir
                    .as_ref()
                    .expect("required output dir")
                    .join("plate_1.gcode"),
            ));
        }
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "--no-check is not supported").into(),
        );
    }
    if args.convert_unit
        && (args.inputs.len() != 1 || args.inputs[0].extension().is_none_or(|ext| ext != "stl"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--convert-unit requires one STL input",
        )
        .into());
    }
    if args.inputs.len() == 1 && args.inputs[0].extension().is_some_and(|ext| ext == "3mf") {
        return crate::direct_project::run(args).await;
    }
    let selection = args
        .selection
        .expect("--slice is required without a subcommand");
    if !matches!(selection, 0 | 1) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected --slice 0 or 1").into());
    }
    if args.inputs.len() > 1 && args.arrange.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multiple STL files require --arrange 0",
        )
        .into());
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
    let imported = if args.convert_unit {
        let mut model = ImportedStl::read(bytes, filename).map_err(|_| {
            CliFailure::invalid_stl_input(
                &args.inputs[0],
                #[cfg(target_os = "linux")]
                &output,
            )
        })?;
        if let Some(unit) = model.infer_unit() {
            model.convert_to_millimeters(unit);
        }
        Some(model)
    } else {
        None
    };
    let gcode =
        if args.load_settings.is_none() && args.load_filaments.is_none() && inputs.len() == 1 {
            if let Some(imported) = &imported {
                ares_core::slice_imported_stl_with_defaults_processed(
                    imported,
                    &mut sliced_plate,
                    args.export_statistics,
                )
                .await
            } else {
                ares_core::slice_cli_stl_with_defaults_processed(
                    bytes,
                    filename,
                    &mut sliced_plate,
                    args.export_statistics,
                )
                .await
            }
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
                .map(|(label, json, source_identity)| LoadSettingsFile {
                    label,
                    json,
                    source_identity: source_identity.as_ref(),
                })
                .collect::<Vec<_>>();
            let mut filaments = filaments
                .iter()
                .map(|(label, json, source_identity)| LoadSettingsFile {
                    label,
                    json,
                    source_identity: source_identity.as_ref(),
                })
                .collect::<Vec<_>>();
            // OrcaSlicer.cpp:2379–2469 fills holes in the list, not beyond it.
            if args.load_defaultfila && !filaments.is_empty() {
                let first = filaments
                    .iter()
                    .copied()
                    .find(|file| !file.label.is_empty())
                    .ok_or_else(|| {
                        CliFailure::missing_default_filament(
                            #[cfg(target_os = "linux")]
                            &output,
                        )
                    })?;
                for file in filaments.iter_mut().filter(|file| file.label.is_empty()) {
                    *file = first;
                }
            }
            let request = LoadSettingsSliceRequest {
                filename,
                catalog_json: None,
                loaded_sources: None,
                uptodate: false,
                settings: &settings,
                filaments: &filaments,
                // Unlike 3MF, a fresh STL model never enters the selected-plate importer.
                // OrcaSlicer.cpp:2042–2045 resets the import selection for non-project files.
                plate: None,
                overrides: &ConfigOverrides::default(),
                placement: if args.arrange.is_none() {
                    Placement::Center
                } else {
                    Placement::None
                },
                orient: false,
                sliced_plate: &mut sliced_plate,
            };
            let metadata = GenerationMetadata::default();
            let context = SlicerContext::new(0);
            if let Some(imported) = &imported {
                ares_core::slice_imported_stl_with_load_settings_processed(
                    imported,
                    request,
                    metadata,
                    context,
                    args.export_statistics,
                )
                .await
            } else if inputs.len() == 1 {
                ares_core::slice_with_load_settings_processed(
                    bytes,
                    request,
                    metadata,
                    context,
                    args.export_statistics,
                )
                .await
            } else {
                let files = inputs
                    .iter()
                    .map(|(name, bytes)| StlFile { name, bytes })
                    .collect::<Vec<_>>();
                ares_core::slice_stl_files_with_load_settings_processed(
                    &files,
                    request,
                    metadata,
                    context,
                    args.export_statistics,
                )
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
    if args.strict {
        crate::failure::slicing::check_strict(
            #[cfg(target_os = "linux")]
            &output,
            1,
            &sliced_plate,
            &[],
            None,
        )?;
    }
    crate::statistics::write(&output, gcode, args.export_statistics, 1)?;
    #[cfg(target_os = "linux")]
    crate::result_json::Success {
        strict_mode: args.strict,
        output: &output,
        plate: 1,
        plate_index: selection as u32,
        sliced_plate,
        upward_machines: &[],
    }
    .write(filename, Vec::new());
    Ok(())
}
