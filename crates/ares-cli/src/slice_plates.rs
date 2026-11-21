use std::{fs, io, path::Path};

use ares_core::{
    CliPlateFailureKind, CliPlatesRequest, CliPlatesSettings, CliUpwardMachine, ConfigOverrides,
    GenerationMetadata, LoadSettingsFile, SliceError, SlicerContext, UpwardMachineInput,
};

use crate::{
    SliceArgs,
    args::ProjectActionArgs,
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
    run_project(
        PlateArgs {
            input: args.input(),
            directory: output_directory(args),
            actions: &args.project_actions,
            settings: args.load_settings.as_deref(),
            filaments: args.load_filaments.as_deref(),
            catalog: args.project_preset_catalog.as_deref(),
            datadir: args.datadir.as_deref(),
            resources: args.resources_dir.as_deref(),
            uptodate: args.uptodate,
            orient: args.orient == Some(1),
        },
        selection,
        input,
        overrides,
        context,
    )
}

pub(super) struct PlateArgs<'a> {
    pub input: &'a Path,
    pub directory: Option<&'a Path>,
    pub actions: &'a ProjectActionArgs,
    pub settings: Option<&'a str>,
    pub filaments: Option<&'a str>,
    pub catalog: Option<&'a Path>,
    pub datadir: Option<&'a Path>,
    pub resources: Option<&'a Path>,
    pub uptodate: bool,
    pub orient: bool,
}

pub(super) fn run_project(
    args: PlateArgs<'_>,
    selection: i32,
    input: &[u8],
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<(), CliFailure> {
    let directory = args.directory;
    let result_path = directory
        .unwrap_or_else(|| Path::new("."))
        .join("plate_1.gcode");
    let file = args.input.to_string_lossy();
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
    let loaded_files = args.settings.is_some() || args.filaments.is_some();
    let admitted = if loaded_files {
        Some(
            ares_core::CliPresetProject::admit(input, &context).map_err(|error| {
                slice_error_to_failure(
                    error,
                    &result_path,
                    #[cfg(target_os = "linux")]
                    1,
                )
            })?,
        )
    } else {
        None
    };
    let catalog = args.catalog.map(fs::read_to_string).transpose()?;
    let settings_files = preset_files(args.settings)?;
    let filament_files = preset_files(args.filaments)?;
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
        loaded_sources =
            crate::preset_sources::load(args.datadir, settings.iter().chain(&filaments).copied())
                .map_err(|error| match error.downcast::<SliceError>() {
                Ok(error) => slice_error_to_failure(
                    *error,
                    &result_path,
                    #[cfg(target_os = "linux")]
                    1,
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
        let resources = crate::installed_resources::resolve(args.resources)?;
        let (source, machine) = crate::compatible_machine::load(input, &resources)?;
        system_catalog = args
            .resources
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
    let mut downward_report = Vec::new();
    let context = crate::custom_gcodes::load(
        context,
        args.actions.load_custom_gcodes.as_deref(),
        &result_path,
        selection.saturating_sub(1).max(0),
    )?;
    let targets = crate::downward::load(args.actions, &result_path)?;
    let mut completed = Vec::new();
    let mut publish_failure = None;
    let request = CliPlatesRequest {
        selection,
        filename,
        settings,
        overrides,
        orient: args.orient,
        upward: CliUpwardMachine {
            source,
            input: upward_input,
            report: &mut upward_report,
        },
    };
    let metadata = GenerationMetadata::default();
    let context = context.with_downward_check(&targets, &mut downward_report);
    let publish = |plate, gcode: &[u8], report: &ares_core::SlicedPlate| {
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
    };
    let result = match admitted {
        Some(project) => project.slice_plates(request, metadata, context, publish),
        None => ares_core::slice_cli_project_plates(input, request, metadata, context, publish),
    };
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
            .with_downward_report(downward_report)
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
        (&upward_report, &downward_report),
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
