use std::{fs, io, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, ImportedStl, LoadSettingsRequest,
    LoadSettingsSliceRequest, SlicedPlate, SlicedStlPlate, SlicerContext,
};

use crate::{
    args::DirectSliceArgs,
    failure::CliFailure,
    preset_sources::{as_load_settings_files, read_preset_list},
    settings_export,
};

pub(crate) mod arrange;
mod plates;

pub(super) async fn run(
    mut args: DirectSliceArgs,
    export_before_slice: bool,
    warnings: &mut crate::PositionalWarnings,
) -> Result<(), CliFailure> {
    let dir = args.outputdir.as_deref().unwrap_or_else(|| Path::new(""));
    let output = dir.join("plate_1.gcode");
    if args.no_check && args.strict {
        return Err(CliFailure::strict_no_check(
            #[cfg(target_os = "linux")]
            &output,
        ));
    }
    if let Some(destination) = args.export_settings.as_deref() {
        if let [input] = args.inputs.as_slice()
            && args.selection.is_none()
            && input
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("3mf"))
        {
            args.diagnostics.startup_version();
            return settings_export::run(input, destination, &args, warnings);
        }
        // OrcaSlicer.cpp:1453–1467 checks actions before transforms, before any input is read.
        if destination == Path::new("-")
            && let Some((_, flag)) = [
                (args.selection.is_some(), "slice"),
                (args.strict, "strict"),
                (args.convert_unit, "convert-unit"),
                (args.arrange.requested(), "arrange"),
            ]
            .into_iter()
            .find(|(present, _)| *present)
        {
            return Err(CliFailure::invalid_parameters(
                #[cfg(target_os = "linux")]
                &output,
                format!("--export-settings - cannot be combined with --{flag}\n"),
            ));
        }
    }
    args.diagnostics.startup_version();
    if args.export_settings.is_some()
        && (args.inputs.len() != 1
            || args.inputs[0].extension().is_none_or(|ext| ext != "stl")
            || args.load_settings.is_none()
            || args.load_filaments.is_none())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--export-settings requires a saved 3MF project or one STL with loaded presets",
        )
        .into());
    }
    if args.inputs.len() == 1 && args.inputs[0].extension().is_some_and(|ext| ext == "3mf") {
        if args.convert_unit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--convert-unit requires STL input",
            )
            .into());
        }
        if args.mtcpp.is_some() {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "--mtcpp requires STL input").into(),
            );
        }
        return crate::direct_project::run(args, warnings).await;
    }
    if args.datadir.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--datadir requires a saved 3MF project",
        )
        .into());
    }
    let selection = u32::try_from(args.selection.unwrap_or_default()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a nonnegative --slice",
        )
    })?;
    let multiple = args.inputs.len() > 1;
    let arranged = multiple || selection > 1;
    let mut models = Vec::new();
    let mut imported = None;
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
        if arranged || args.convert_unit {
            let mut model = ImportedStl::read(&bytes, name).map_err(|_| {
                CliFailure::invalid_stl_input(
                    path,
                    #[cfg(target_os = "linux")]
                    &output,
                )
            })?;
            if args.convert_unit
                && let Some(unit) = model.infer_unit()
            {
                model.convert_to_millimeters(unit);
            }
            if arranged {
                models.push(model);
            } else {
                imported = Some(model);
            }
        }
        if !multiple {
            single_input = bytes;
        }
    }
    let mut sliced_plate = SlicedPlate::default();
    let context = crate::custom_gcodes::load(
        SlicerContext::new(0)
            .with_max_triangle_count(args.mtcpp.unwrap_or_default())
            .with_no_check(args.no_check),
        args.project_actions.load_custom_gcodes.as_deref(),
        &output,
        0,
    )?;
    let mut export_after = None;
    let mut completed = Vec::new();
    let context = context.with_cli_path_conflict_check();
    let plates = if args.load_settings.is_none() && args.load_filaments.is_none() && !multiple {
        if selection > 1 {
            Vec::new()
        } else {
            let processed = if let Some(imported) = &imported {
                ares_core::slice_imported_stl_with_defaults_processed(
                    imported,
                    &mut sliced_plate,
                    args.export_statistics,
                    context,
                )
                .await
            } else {
                ares_core::slice_cli_stl_with_defaults_processed(
                    &single_input,
                    filename,
                    &mut sliced_plate,
                    args.export_statistics,
                    context,
                )
                .await
            }
            .map_err(|error| {
                plates::slice_failure(
                    (&args.inputs[0], &output),
                    &mut args.diagnostics,
                    error,
                    (1, Some(&sliced_plate)),
                )
            })?;
            vec![SlicedStlPlate {
                id: 1,
                processed,
                report: sliced_plate,
            }]
        }
    } else {
        let settings_files = read_preset_list(args.load_settings.as_deref())?;
        let filament_files = read_preset_list(args.load_filaments.as_deref())?;
        let settings = as_load_settings_files(&settings_files);
        let mut filaments = as_load_settings_files(&filament_files);
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
                &single_input,
                filename,
                LoadSettingsRequest {
                    loaded_sources: None,
                    settings: &settings,
                    filaments: &filaments,
                },
            )
            .map_err(io::Error::other);
            if args.selection.is_none() {
                settings_export::write(destination, &json?)?;
                #[cfg(target_os = "linux")]
                crate::result_json::write_plate_batch(
                    &output,
                    (0, args.strict),
                    Vec::new(),
                    (&[], &[]),
                    &[],
                );
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
            loaded_sources: None,
            uptodate: false,
            settings: &settings,
            filaments: &filaments,
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: args.arrange.placement(args.inputs.len()),
            orient: false,
            sliced_plate: &mut sliced_plate,
        };
        if models.is_empty() {
            let processed = if let Some(imported) = &imported {
                ares_core::slice_imported_stl_with_load_settings_processed(
                    imported,
                    request,
                    GenerationMetadata::default(),
                    context,
                    args.export_statistics,
                )
                .await
            } else {
                ares_core::slice_with_load_settings_processed(
                    &single_input,
                    request,
                    GenerationMetadata::default(),
                    context,
                    args.export_statistics,
                )
                .await
            }
            .map_err(|error| {
                plates::slice_failure(
                    (&args.inputs[0], &output),
                    &mut args.diagnostics,
                    error,
                    (1, Some(&sliced_plate)),
                )
            })?;
            vec![SlicedStlPlate {
                id: 1,
                processed,
                report: sliced_plate,
            }]
        } else {
            plates::slice_arranged(
                (&args.inputs[0], &output),
                (
                    &mut args.diagnostics,
                    args.no_check,
                    args.strict,
                    args.export_statistics,
                ),
                (models, request, selection, context),
                &mut completed,
            )?;
            Vec::new()
        }
    };
    for plate in plates {
        completed.push(plates::publish(
            &output,
            (&mut args.diagnostics, args.no_check, args.strict),
            plate,
            args.export_statistics,
        )?);
    }

    if let Some((destination, json)) = export_after {
        settings_export::write(destination, &json?)?;
    }
    #[cfg(target_os = "linux")]
    crate::result_json::write_plate_batch(
        &output,
        (selection, args.strict),
        Vec::new(),
        (&[], &[]),
        &completed,
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
