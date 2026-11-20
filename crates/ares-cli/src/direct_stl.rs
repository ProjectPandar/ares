use std::{fs, io, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, ImportedStl, LoadSettingsFile, LoadSettingsRequest,
    LoadSettingsSliceRequest, Placement, SliceError, SlicedPlate, SlicerContext, StlFile,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    read_preset_files, settings_export, split_native_list,
};

pub(super) async fn run(
    args: DirectSliceArgs,
    export_before_slice: bool,
) -> Result<(), CliFailure> {
    let output = args
        .outputdir
        .as_deref()
        .unwrap_or_else(|| Path::new(""))
        .join("plate_1.gcode");
    if let Some(destination) = args.export_settings.as_deref() {
        if let [input] = args.inputs.as_slice()
            && args.selection.is_none()
            && input
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("3mf"))
        {
            return settings_export::run(input, destination);
        }
        // OrcaSlicer.cpp:1453–1467 checks actions before transforms, before any input is read.
        if destination == Path::new("-")
            && let Some((_, flag)) = [
                (args.selection.is_some(), "slice"),
                (args.strict, "strict"),
                (args.convert_unit, "convert-unit"),
                (args.arrange.is_some(), "arrange"),
            ]
            .into_iter()
            .find(|(present, _)| *present)
        {
            return Err(CliFailure::invalid_parameters(
                format!("--export-settings - cannot be combined with --{flag}\n"),
                #[cfg(target_os = "linux")]
                &output,
            ));
        }
        if args.inputs.len() != 1
            || args.inputs[0].extension().is_none_or(|ext| ext != "stl")
            || args.load_settings.is_none()
            || args.load_filaments.is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--export-settings requires a saved 3MF project or one STL with loaded presets",
            )
            .into());
        }
    }
    if args.no_check {
        if args.strict {
            return Err(CliFailure::strict_no_check(
                #[cfg(target_os = "linux")]
                &output,
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
        if args.mtcpp.is_some() {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "--mtcpp requires STL input").into(),
            );
        }
        return crate::direct_project::run(args).await;
    }
    let selection = args.selection.unwrap_or_default();
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
    let context = SlicerContext::new(0).with_max_triangle_count(args.mtcpp.unwrap_or_default());
    let mut export_after = None;
    let gcode =
        if args.load_settings.is_none() && args.load_filaments.is_none() && inputs.len() == 1 {
            if let Some(imported) = &imported {
                ares_core::slice_imported_stl_with_defaults_processed(
                    imported,
                    &mut sliced_plate,
                    args.export_statistics,
                    context,
                )
                .await
            } else {
                ares_core::slice_cli_stl_with_defaults_processed(
                    bytes,
                    filename,
                    &mut sliced_plate,
                    args.export_statistics,
                    context,
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
                .map(|(label, json)| LoadSettingsFile { label, json })
                .collect::<Vec<_>>();
            let mut filaments = filaments
                .iter()
                .map(|(label, json)| LoadSettingsFile { label, json })
                .collect::<Vec<_>>();
            // OrcaSlicer.cpp:2379–2469 fills holes in the list, not beyond it.
            if args.load_defaultfila && !filaments.is_empty() {
                let first = filaments
                    .iter()
                    .copied()
                    .find(|file| !file.label.is_empty())
                    .ok_or_else(|| CliFailure::invalid_preset_file(&output))?;
                for file in filaments.iter_mut().filter(|file| file.label.is_empty()) {
                    *file = first;
                }
            }
            if let Some(destination) = args.export_settings.as_deref() {
                let json = ares_core::export_stl_settings(
                    bytes,
                    filename,
                    LoadSettingsRequest {
                        settings: &settings,
                        filaments: &filaments,
                    },
                )
                .map_err(io::Error::other);
                if args.selection.is_none() {
                    settings_export::write(destination, &json?)?;
                    #[cfg(target_os = "linux")]
                    crate::result_json::ResultJson::write_without_slice(&output, args.strict);
                    return Ok(());
                } else if export_before_slice {
                    settings_export::write(destination, &json?)?;
                } else {
                    export_after = Some((destination, json));
                }
            }
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
                placement: if args.arrange.is_none() {
                    Placement::Center
                } else {
                    Placement::None
                },
                orient: false,
                sliced_plate: &mut sliced_plate,
            };
            let metadata = GenerationMetadata::default();
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
    let gcode = gcode.map_err(|error| {
        if let SliceError::TriangleCountExceedsLimit { count, limit } = &error {
            // OrcaSlicer.cpp:6639 writes to the file sink, not to stderr.
            args.diagnostics.error(format_args!(
                "plate 1: triangle count {count} exceeds the limit:{limit}"
            ));
        }
        match error {
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
        }
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
    if let Some((destination, json)) = export_after {
        settings_export::write(destination, &json?)?;
    }
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
