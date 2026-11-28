use std::{fs, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, ImportedStl, LoadSettingsRequest,
    LoadSettingsSliceRequest, SlicedPlate, SlicerContext,
};

use crate::{
    CliFailure,
    args::DirectSliceArgs,
    preset_sources::{as_load_settings_files, read_preset_list},
};

fn destination(output: &Path, outputdir: Option<&Path>) -> std::path::PathBuf {
    outputdir.map_or_else(
        || output.to_owned(),
        |dir| {
            // Native concatenates the string rather than treating an absolute
            // export filename as overriding --outputdir.
            let mut path = dir.as_os_str().to_owned();
            path.push("/");
            path.push(output);
            path.into()
        },
    )
}

pub(super) async fn run_direct(args: &DirectSliceArgs, output: &Path) -> Result<(), CliFailure> {
    let output = destination(output, args.outputdir.as_deref());
    let stl = args.inputs.iter().all(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("stl"))
    });
    let sliced = args.selection.is_some();
    let placement = if args.arrange.arrange == Some(0) {
        ares_core::Placement::None
    } else {
        ares_core::Placement::Arrange
    };
    let arranged_stl = stl
        && args.inputs.len() > 1
        && args.selection == Some(0)
        && args.arrange.arrange == Some(1);
    let unsupported = [
        (
            args.selection
                .is_some_and(|plate| plate != 1 && !(stl && plate == 0)),
            "--slice",
        ),
        (sliced && args.outputdir.is_some(), "--outputdir"),
        (
            sliced && !stl && args.load_settings.is_some(),
            "--load-settings",
        ),
        (
            sliced && !stl && args.load_filaments.is_some(),
            "--load-filaments",
        ),
        (!stl && args.arrange.arrange == Some(1), "--arrange"),
        (!stl && args.convert_unit, "--convert-unit"),
        (args.orient.is_some_and(|value| value != 0), "--orient"),
        (args.min_save && (stl || !sliced), "--min-save"),
        (args.load_defaultfila, "--load-defaultfila"),
        (args.enable_timelapse, "--enable-timelapse"),
        (args.export_statistics, "--export-statistics"),
    ];
    reject_unsupported(&output, &unsupported)?;
    let failed = |detail| CliFailure::project_export(&output, false, detail);
    if args.inputs.len() != 1 && !arranged_stl {
        return Err(failed(
            "3MF export requires exactly one input model".to_owned(),
        ));
    }
    let models = args
        .inputs
        .iter()
        .map(|path| read_model(path, &output))
        .collect::<Result<Vec<_>, _>>()?;
    let settings_files = read_preset_list(args.load_settings.as_deref())?;
    let filament_files = read_preset_list(args.load_filaments.as_deref())?;
    let settings = as_load_settings_files(&settings_files);
    let filaments = as_load_settings_files(&filament_files);
    let presets = LoadSettingsRequest {
        loaded_sources: None,
        settings: &settings,
        filaments: &filaments,
    };
    let mut upward_report = Vec::new();
    let archive = if stl {
        export_stl(
            models,
            presets,
            (args.selection, placement, args.convert_unit),
        )
        .await
    } else {
        let (input, filename) = &models[0];
        if sliced {
            let admitted = ares_core::CliSavedSlicedProject::admit(input, args.allow_newer_file)
                .and_then(|mut project| Ok((project.take_upward_source()?, project)));
            match admitted {
                Ok((source, project)) => {
                    let resources = crate::installed_resources::resolve(None)?;
                    let original =
                        crate::compatible_machine::read_machine_full(source.as_ref(), &resources)?;
                    let catalog = crate::project_presets::retained_printer_catalog(
                        &resources,
                        source.as_ref(),
                        false,
                    )?;
                    project.export(
                        filename,
                        args.min_save,
                        (
                            catalog.as_ref(),
                            ares_core::CliUpwardMachine {
                                source,
                                input: ares_core::UpwardMachineInput::FetchOriginal(
                                    original.as_deref(),
                                ),
                                report: &mut upward_report,
                            },
                        ),
                        SlicerContext::new(0),
                    )
                }
                Err(error) => Err(error),
            }
        } else {
            ares_core::export_3mf(input, filename, presets)
        }
    }
    .map_err(|err| {
        use ares_core::SliceError;
        let slice_failure = |error| {
            crate::failure::slice_error_to_failure(
                error,
                &output,
                #[cfg(target_os = "linux")]
                0,
            )
        };
        match err {
            SliceError::InvalidValuesInProject(_) if sliced && !stl => {
                slice_failure(err).with_upward_report(upward_report)
            }
            SliceError::ProcessNotCompatible | SliceError::FileVersionNotSupported => {
                slice_failure(err)
            }
            other => failed(other.to_string()),
        }
    })?;
    fs::write(&output, archive).map_err(|err| failed(err.to_string()))
}

async fn export_stl(
    inputs: Vec<(Vec<u8>, &str)>,
    presets: LoadSettingsRequest<'_>,
    (selection, placement, convert): (Option<i32>, ares_core::Placement, bool),
) -> Result<Vec<u8>, ares_core::SliceError> {
    let filename = inputs[0].1;
    let models = inputs
        .into_iter()
        .map(|(bytes, name)| {
            let mut model = ImportedStl::read(&bytes, name)?;
            if convert && let Some(unit) = model.infer_unit() {
                model.convert_to_millimeters(unit);
            }
            Ok(model)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [model] = models.as_slice() else {
        let plates = ares_core::slice_stl_models_with_load_settings(
            models,
            stl_slice_request(
                filename,
                presets,
                placement,
                &mut SlicedPlate::default(),
                &ConfigOverrides::default(),
            ),
            0,
            SlicerContext::new(0),
            false,
        )
        .map_err(|failure| *failure.error)?
        .map(|plate| plate.map_err(|failure| *failure.error))
        .collect::<Result<Vec<_>, _>>()?;
        return ares_core::export_arranged_sliced_3mf(plates);
    };
    if selection.is_none() {
        return ares_core::export_imported_stl_3mf(model, presets, placement);
    }
    let processed = ares_core::slice_stl_with_load_settings_for_export(
        model,
        stl_slice_request(
            filename,
            presets,
            placement,
            &mut SlicedPlate::default(),
            &ConfigOverrides::default(),
        ),
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await?;
    ares_core::export_sliced_3mf(
        &[],
        filename,
        processed,
        ares_core::print_object_id(1, 1, false),
        false,
    )
}

fn stl_slice_request<'a>(
    filename: &'a str,
    presets: LoadSettingsRequest<'a>,
    placement: ares_core::Placement,
    sliced_plate: &'a mut SlicedPlate,
    overrides: &'a ConfigOverrides,
) -> LoadSettingsSliceRequest<'a> {
    LoadSettingsSliceRequest {
        filename,
        catalog_json: None,
        loaded_sources: presets.loaded_sources,
        uptodate: false,
        settings: presets.settings,
        filaments: presets.filaments,
        plate: None,
        overrides,
        placement,
        orient: false,
        sliced_plate,
    }
}

fn reject_unsupported(output: &Path, unsupported: &[(bool, &str)]) -> Result<(), CliFailure> {
    if let Some((_, option)) = unsupported.iter().find(|(present, _)| *present) {
        return Err(CliFailure::project_export(
            output,
            true,
            format!("unsupported 3MF export option {option}"),
        ));
    }
    Ok(())
}

fn read_model<'a>(path: &'a Path, output: &Path) -> Result<(Vec<u8>, &'a str), CliFailure> {
    let failed = |detail| CliFailure::project_export(output, false, detail);
    let input = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                path,
                #[cfg(target_os = "linux")]
                output,
            ));
        }
        Err(err) => return Err(failed(err.to_string())),
    };
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| failed("invalid source filename".to_owned()))?;
    Ok((input, filename))
}
