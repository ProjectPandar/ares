use std::{cell::RefCell, fs, io, path::Path, rc::Rc};

use ares_core::{
    CliPlateFailureKind, CliPlatesRequest, CliPlatesSettings, CliUpwardMachine, ConfigOverrides,
    GenerationMetadata, LoadSettingsFile, SliceError, SlicerContext, UpwardMachineInput,
};

use crate::{
    SliceArgs,
    args::ProjectActionArgs,
    failure::{CliFailure, slice_error_to_failure},
    native_refresh::{self, RefreshInputs},
    preset_sources::{as_load_settings_files, read_preset_list},
};

pub(super) struct PlateOptions<'a> {
    pub input: &'a Path,
    pub directory: Option<&'a Path>,
    pub actions: &'a ProjectActionArgs,
    pub datadir: Option<&'a Path>,
    pub load_settings: Option<&'a str>,
    pub load_filaments: Option<&'a str>,
    pub project_preset_catalog: Option<&'a Path>,
    pub resources_dir: Option<&'a Path>,
    pub uptodate: bool,
    pub uptodate_settings: Option<&'a str>,
    pub uptodate_filaments: Option<&'a str>,
    pub orient: bool,
    #[cfg(target_os = "linux")]
    pub report_plate: u32,
}

impl<'a> From<&'a SliceArgs> for PlateOptions<'a> {
    fn from(args: &'a SliceArgs) -> Self {
        Self {
            input: args.input(),
            directory: output_directory(args),
            actions: &args.project_actions,
            datadir: args.datadir.as_deref(),
            load_settings: args.load_settings.as_deref(),
            load_filaments: args.load_filaments.as_deref(),
            project_preset_catalog: args.project_preset_catalog.as_deref(),
            resources_dir: args.resources_dir.as_deref(),
            uptodate: args.uptodate,
            uptodate_settings: args.uptodate_settings.as_deref(),
            uptodate_filaments: args.uptodate_filaments.as_deref(),
            orient: args.orient == Some(1),
            #[cfg(target_os = "linux")]
            report_plate: args.plate.unwrap_or(1),
        }
    }
}

pub(super) fn run(
    args: PlateOptions<'_>,
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
    let loaded_files = args.load_settings.is_some() || args.load_filaments.is_some();
    let map_err = |error| {
        slice_error_to_failure(
            error,
            &result_path,
            #[cfg(target_os = "linux")]
            args.report_plate,
        )
    };
    let mut admitted = ares_core::CliPresetProject::open(input, true).map_err(map_err)?;
    if loaded_files {
        admitted.check_post_process(&context).map_err(map_err)?;
    }
    let substitutions = admitted.substitutions().to_vec();
    let catalog = args
        .project_preset_catalog
        .map(fs::read_to_string)
        .transpose()?;
    let settings_files = read_preset_list(args.load_settings)?;
    let filament_files = read_preset_list(args.load_filaments)?;
    let settings = as_load_settings_files(&settings_files);
    let filaments = as_load_settings_files(&filament_files);
    let loaded_sources;
    let system_catalog;
    let refresh;
    let (settings, source, machine_full) = if loaded_files {
        loaded_sources = sources_for_slice(
            args.datadir,
            settings.iter().chain(&filaments).copied(),
            &result_path,
            #[cfg(target_os = "linux")]
            args.report_plate,
        )?;
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
        let source = admitted
            .take_upward_source()
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
        let source = admitted
            .take_upward_source()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        if args.uptodate
            && let Some(source) = source
        {
            admitted.check_post_process(&context).map_err(map_err)?;
            refresh = native_refresh::load(
                &RefreshInputs {
                    resources_dir: args.resources_dir,
                    settings: args.uptodate_settings,
                    filaments: args.uptodate_filaments,
                    #[cfg(target_os = "linux")]
                    output: &result_path,
                },
                &source,
                &[],
            )?;
            (
                CliPlatesSettings::RetainedRefresh(&refresh),
                Some(source),
                None,
            )
        } else {
            let resources = crate::installed_resources::resolve(args.resources_dir)?;
            let machine =
                crate::compatible_machine::read_machine_full(source.as_ref(), &resources)?;
            system_catalog = crate::project_presets::retained_printer_catalog(
                &resources,
                source.as_ref(),
                args.resources_dir.is_some(),
            )?;
            (
                CliPlatesSettings::Embedded {
                    system_catalog: system_catalog.as_ref(),
                },
                source,
                machine,
            )
        }
    };
    let upward_input = match &settings {
        CliPlatesSettings::RetainedRefresh(refresh) => UpwardMachineInput::RetainedRefresh(refresh),
        _ if catalog.is_some() => UpwardMachineInput::Embedded,
        _ => UpwardMachineInput::FetchOriginal(machine_full.as_deref()),
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
    let sink_error: Rc<RefCell<Option<io::Error>>> = Rc::new(RefCell::new(None));
    let make_sink = |plate, _report: &ares_core::SlicedPlate| {
        Ok(match directory {
            Some(directory) => crate::gcode_file_sink::PlateSink::File(
                crate::gcode_file_sink::AtomicFileSink::in_directory(
                    directory.join(format!("plate_{plate}.gcode")),
                    Rc::clone(&sink_error),
                ),
            ),
            None => crate::gcode_file_sink::PlateSink::Discard,
        })
    };
    let on_complete =
        |plate, sink: crate::gcode_file_sink::PlateSink, report: &ares_core::SlicedPlate| {
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
            if let Err(error) = sink.commit() {
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
    let result =
        admitted.slice_plates_to_sink(request, metadata, context, (make_sink, on_complete));
    if let Err(error) = result {
        if publish_failure.is_none()
            && let Some(io_error) = sink_error.borrow_mut().take()
        {
            publish_failure = Some(CliFailure::slice_export_error(
                #[cfg(target_os = "linux")]
                &result_path,
                #[cfg(target_os = "linux")]
                error.plate,
                io_error,
            ));
        }
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

pub(super) fn sources_for_slice<'a>(
    datadir: Option<&Path>,
    files: impl Iterator<Item = LoadSettingsFile<'a>>,
    output: &Path,
    #[cfg(target_os = "linux")] plate: u32,
) -> Result<ares_core::LoadedPresetSources, CliFailure> {
    crate::preset_sources::load(datadir, files).map_err(|error| {
        match error.downcast::<SliceError>() {
            Ok(error) => slice_error_to_failure(
                *error,
                output,
                #[cfg(target_os = "linux")]
                plate,
            ),
            Err(error) => CliFailure::config_file_error(
                #[cfg(target_os = "linux")]
                output,
                format!("{error}\n"),
            ),
        }
    })
}
