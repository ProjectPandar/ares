use std::{fs, io, path::Path};

use ares_core::{
    CliPlateFailureKind, CliPlatesRequest, CliPlatesSettings, CliUpwardMachine, ConfigOverrides,
    GenerationMetadata, LoadSettingsFile, SliceError, SlicerContext, UpwardMachineInput,
};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    substitutions,
};

pub(super) fn run(
    args: &SliceArgs,
    selection: i32,
    input: &[u8],
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<(), CliFailure> {
    let directory = output_directory(args);
    let result_path = directory
        .unwrap_or_else(|| Path::new("."))
        .join("plate_1.gcode");
    let file = args.input().to_string_lossy();
    let filename = args
        .input()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
    let loaded_files = args.load_settings.is_some() || args.load_filaments.is_some();
    if loaded_files && let Err(error) = ares_core::validate_load_settings_input(input) {
        return Err(slice_error_to_failure(
            error,
            &result_path,
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        ));
    }
    let catalog = args
        .project_preset_catalog
        .as_ref()
        .map(fs::read_to_string)
        .transpose()?;
    let settings_files = preset_files(args.load_settings.as_deref())?;
    let filament_files = preset_files(args.load_filaments.as_deref())?;
    let settings = settings_files
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect::<Vec<_>>();
    let filaments = filament_files
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect::<Vec<_>>();
    let loaded_sources;
    let system_catalog;
    let (settings, source, machine_full) = if loaded_files {
        loaded_sources = crate::preset_sources::load(
            args.datadir.as_deref(),
            settings.iter().chain(&filaments).copied(),
        )
        .map_err(|error| match error.downcast::<SliceError>() {
            Ok(error) => slice_error_to_failure(
                *error,
                &result_path,
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            ),
            Err(error) => CliFailure::config_file_error(
                #[cfg(target_os = "linux")]
                &result_path,
                format!("{error}\n"),
            ),
        })?;
        (
            CliPlatesSettings::Files {
                settings: &settings,
                filaments: &filaments,
                catalog_json: catalog.as_deref(),
                loaded_sources: Some(&loaded_sources),
                uptodate: args.uptodate,
            },
            None,
            None,
        )
    } else if let Some(json) = catalog.as_deref() {
        let source = ares_core::UpwardMachineSource::from_project(input)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        (
            CliPlatesSettings::Catalog {
                json,
                uptodate: args.uptodate,
            },
            source,
            None,
        )
    } else {
        let resources = crate::installed_resources::resolve(args.resources_dir.as_deref())?;
        let (source, machine) = crate::compatible_machine::load(input, &resources)?;
        system_catalog = args
            .resources_dir
            .as_deref()
            .map(crate::project_presets::load_resources_catalog)
            .transpose()?;
        (
            CliPlatesSettings::Embedded {
                system_catalog: system_catalog.as_ref(),
            },
            source,
            machine,
        )
    };
    let upward_input = if catalog.is_some() {
        UpwardMachineInput::Embedded
    } else {
        UpwardMachineInput::FetchOriginal(machine_full.as_deref())
    };
    let mut upward_report = Vec::new();
    let mut completed = Vec::new();
    let mut publish_failure = None;
    let result = ares_core::slice_cli_project_plates(
        input,
        CliPlatesRequest {
            selection,
            filename,
            settings,
            overrides,
            orient: args.orient == Some(1),
            upward: CliUpwardMachine {
                source,
                input: upward_input,
                report: &mut upward_report,
            },
        },
        GenerationMetadata::default(),
        context,
        |plate, gcode, report| {
            if let Some(failure) = crate::failure::slicing::critical_failure(
                #[cfg(target_os = "linux")]
                &result_path,
                plate,
                report,
            ) {
                publish_failure = Some(failure);
                return Err(SliceError::InvalidInput(
                    "plate failed slicing admission".to_owned(),
                ));
            }
            if let Some(directory) = directory
                && let Err(error) = fs::create_dir_all(directory)
                    .and_then(|()| fs::write(directory.join(format!("plate_{plate}.gcode")), gcode))
            {
                publish_failure = Some(CliFailure::slice_export_error(
                    #[cfg(target_os = "linux")]
                    &result_path,
                    #[cfg(target_os = "linux")]
                    plate,
                    error,
                ));
                return Err(SliceError::InvalidInput("cannot export G-code".to_owned()));
            }
            completed.push((plate, report.clone()));
            Ok(())
        },
    );
    let substitutions = substitutions(input);
    if let Err(error) = result {
        let failure = publish_failure
            .unwrap_or_else(|| match error.kind {
                CliPlateFailureKind::Selection => CliFailure::invalid_slice_selection(
                    #[cfg(target_os = "linux")]
                    &result_path,
                ),
                CliPlateFailureKind::Slice(reason) => slice_error_to_failure(
                    *reason,
                    &result_path,
                    #[cfg(target_os = "linux")]
                    error.plate,
                ),
            })
            .with_upward_report(upward_report)
            .with_config_substitutions(&file, substitutions);
        #[cfg(target_os = "linux")]
        let failure = failure.with_plate_batch(error.plate, *error.report, completed);
        return Err(failure);
    }
    eprint!(
        "{}",
        crate::failure::substitution_warnings(&file, &substitutions)
    );
    #[cfg(target_os = "linux")]
    crate::result_json::write_plate_batch(
        &result_path,
        (selection as u32, false),
        (&file, substitutions),
        &upward_report,
        &completed,
    );
    Ok(())
}

pub(super) fn output_directory(args: &SliceArgs) -> Option<&Path> {
    args.outputdir
        .as_deref()
        .filter(|path| !path.as_os_str().is_empty())
}

fn preset_files(
    value: Option<&str>,
) -> Result<Vec<(String, String, Option<ares_core::SourceIdentity>)>, CliFailure> {
    value.map_or_else(
        || Ok(Vec::new()),
        |list| crate::preset_sources::read_preset_files(&crate::split_native_list(list)),
    )
}
