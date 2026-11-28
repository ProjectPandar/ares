use std::{fs, io, path::Path};

use ares_core::{
    CliPresetProject, CliProjectSliceRequest, CliUpwardMachine, ConfigOverrides,
    ConfigSubstitution, GenerationMetadata, LoadSettingsSliceRequest, Placement, ProcessedGCode,
    SliceError, SlicedPlate, SlicerContext, UpwardMachineInput, UpwardMachineSource,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    preset_sources::{as_load_settings_files, read_preset_list},
    slice_plates::{self, PlateOptions},
};

pub(super) async fn run(mut args: DirectSliceArgs) -> Result<(), CliFailure> {
    let datadir = args
        .datadir
        .as_deref()
        .filter(|path| !path.as_os_str().is_empty());
    if let Some(datadir) = datadir
        && !datadir.exists()
        && fs::create_dir_all(datadir).is_err()
    {
        eprintln!(
            "Could not create data directory: {}\nsetup params error",
            datadir.display()
        );
        std::process::exit(-2);
    }
    let selection = args
        .selection
        .expect("--slice is required without a subcommand");
    let plate = u32::try_from(selection).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "select a project plate with --slice <n>",
        )
    })?;
    if plate == 0
        && let Some((_, flag)) = [
            (args.strict, "--strict"),
            (args.export_statistics, "--export-statistics"),
        ]
        .into_iter()
        .find(|(present, _)| *present)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{flag} is not supported with --slice 0 for a saved project"),
        )
        .into());
    }
    let files = (
        args.load_settings.as_deref(),
        args.load_filaments.as_deref(),
    );
    let input = &args.inputs[0];
    let directory = args
        .outputdir
        .as_deref()
        .expect("--outputdir is required without a subcommand");
    let output = directory.join(format!("plate_{}.gcode", plate.max(1)));
    let bytes = match fs::read(input) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                input,
                #[cfg(target_os = "linux")]
                &output,
            ));
        }
        Err(error) => return Err(error.into()),
    };
    if plate == 0 {
        return slice_plates::run(
            PlateOptions {
                input,
                directory: Some(directory),
                actions: &args.project_actions,
                datadir,
                load_settings: args.load_settings.as_deref(),
                load_filaments: args.load_filaments.as_deref(),
                project_preset_catalog: None,
                resources_dir: None,
                uptodate: false,
                uptodate_settings: None,
                uptodate_filaments: None,
                orient: false,
                allow_newer_file: args.allow_newer_file,
                #[cfg(target_os = "linux")]
                report_plate: 1,
            },
            selection,
            &bytes,
            &ConfigOverrides::default(),
            args.project_actions
                .context(SlicerContext::new(0))
                .with_no_check(args.no_check)
                .with_enable_timelapse(args.enable_timelapse),
        );
    }
    let mut project = CliPresetProject::open(&bytes, args.allow_newer_file).map_err(|error| {
        slice_error_to_failure(
            error,
            &output,
            #[cfg(target_os = "linux")]
            plate,
        )
    })?;
    enum ProjectPresets<'a> {
        Replace(Option<&'a str>, Option<&'a str>),
        Saved {
            source: Option<UpwardMachineSource>,
            machine: Option<Vec<u8>>,
            catalog: Option<Box<ares_core::PresetCatalogHandle>>,
        },
    }
    let presets = match files {
        (None, None) => {
            let resources = crate::installed_resources::resolve(None)?;
            let source = project
                .take_upward_source()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
            let machine =
                crate::compatible_machine::read_machine_full(source.as_ref(), &resources)?;
            let catalog = crate::project_presets::retained_printer_catalog(
                &resources,
                source.as_ref(),
                false,
            )?;
            ProjectPresets::Saved {
                source,
                machine,
                catalog: catalog.map(Box::new),
            }
        }
        (settings, filaments) => ProjectPresets::Replace(settings, filaments),
    };
    let mut downward_machines = Vec::new();
    let targets = crate::downward::load(&args.project_actions, &output)?;
    let context = crate::custom_gcodes::load(
        args.project_actions
            .context(SlicerContext::new(0))
            .with_no_check(args.no_check)
            .with_cli_path_conflict_check()
            .with_enable_timelapse(args.enable_timelapse)
            .with_downward_check(&targets, &mut downward_machines),
        args.project_actions.load_custom_gcodes.as_deref(),
        &output,
        plate as i32 - 1,
    )?;
    enum DirectOutcome {
        Buffered(ProcessedGCode),
        Streamed(
            ares_core::StreamedGCode,
            crate::gcode_file_sink::AtomicFileSink,
        ),
    }
    let mut sliced_plate = SlicedPlate::default();
    let mut upward_machines = Vec::new();
    let (outcome, substitutions) = match presets {
        ProjectPresets::Replace(settings, filaments) => {
            let (processed, substitutions) = slice_with_replacement(
                (input, project, plate, &output),
                (settings, filaments, datadir),
                context,
                &mut sliced_plate,
                args.export_statistics,
            )?;
            upward_machines = std::mem::take(&mut sliced_plate.upward_machines);
            (processed.map(DirectOutcome::Buffered), substitutions)
        }
        ProjectPresets::Saved {
            source,
            machine,
            catalog,
        } => {
            let substitutions = project.substitutions().to_vec();
            let mut sink = crate::gcode_file_sink::AtomicFileSink::new(output.clone());
            let streamed = project.slice_project_to_sink(
                CliProjectSliceRequest {
                    plate: Some(plate),
                    overrides: &ConfigOverrides::default(),
                    record_moves: args.export_statistics,
                    orient: false,
                    upward: CliUpwardMachine {
                        source,
                        input: UpwardMachineInput::FetchOriginal(machine.as_deref()),
                        report: &mut upward_machines,
                    },
                    sliced_plate: &mut sliced_plate,
                },
                (GenerationMetadata::default(), (catalog.as_deref(), None)),
                context,
                &mut sink,
            );
            if streamed.is_err()
                && let Some(io_error) = sink.take_error()
            {
                return Err(io_error.into());
            }
            (
                streamed.map(|streamed| DirectOutcome::Streamed(streamed, sink)),
                substitutions,
            )
        }
    };
    let outcome = outcome.map_err(|error| {
        if let Some(failure) = crate::failure::slicing::path_conflict(
            &error,
            &mut args.diagnostics,
            #[cfg(target_os = "linux")]
            &output,
            plate,
            #[cfg(target_os = "linux")]
            &sliced_plate,
        ) {
            return failure
                .with_config_substitutions(&input.to_string_lossy(), substitutions.clone())
                .with_upward_report(upward_machines.clone())
                .with_downward_report(downward_machines.clone());
        }
        #[cfg(target_os = "linux")]
        let detail = match &error {
            ares_core::SliceError::Validate { message, .. } if args.no_check => {
                Some(message.clone())
            }
            _ => None,
        };
        let failure = slice_error_to_failure(
            error,
            &output,
            #[cfg(target_os = "linux")]
            plate,
        )
        .with_upward_report(upward_machines.clone())
        .with_downward_report(downward_machines.clone());
        #[cfg(target_os = "linux")]
        let failure = failure.with_validation_detail(detail);
        failure
    })?;
    let input_file = input.to_string_lossy();
    let project = Some((input_file.as_ref(), substitutions.as_slice()));
    crate::failure::slicing::check_direct(
        (&mut args.diagnostics, args.no_check, args.strict),
        #[cfg(target_os = "linux")]
        &output,
        (plate, &sliced_plate),
        &upward_machines,
        project,
    )
    .map_err(|failure| failure.with_downward_report(downward_machines.clone()))?;
    match outcome {
        DirectOutcome::Buffered(processed) => {
            crate::statistics::write(&output, processed, args.export_statistics, plate)?;
        }
        DirectOutcome::Streamed(streamed, sink) => {
            sink.commit()?;
            if args.export_statistics {
                crate::statistics::write_sidecar(&output, &streamed.result, plate)?;
            }
        }
    }
    #[cfg(target_os = "linux")]
    crate::result_json::Success {
        strict_mode: args.strict,
        output: &output,
        plate,
        plate_index: plate,
        sliced_plate,
        upward_machines: &upward_machines,
        downward_machines: &downward_machines,
    }
    .write(&input_file, substitutions);
    Ok(())
}

fn slice_with_replacement(
    (input, admitted, plate, output): (&Path, CliPresetProject<'_>, u32, &Path),
    (settings, filaments, datadir): (Option<&str>, Option<&str>, Option<&Path>),
    context: SlicerContext<'_>,
    sliced_plate: &mut SlicedPlate,
    record_moves: bool,
) -> Result<(Result<ProcessedGCode, SliceError>, Vec<ConfigSubstitution>), CliFailure> {
    if let Err(error) = admitted.check_post_process(&context) {
        return Ok((Err(error), Vec::new()));
    }
    let substitutions = admitted.substitutions().to_vec();
    let settings_files = read_preset_list(settings)?;
    let filament_files = read_preset_list(filaments)?;
    let settings = as_load_settings_files(&settings_files);
    let filaments = as_load_settings_files(&filament_files);
    let loaded_sources = datadir
        .map(|datadir| {
            slice_plates::sources_for_slice(
                Some(datadir),
                settings.iter().chain(&filaments).copied(),
                output,
                #[cfg(target_os = "linux")]
                plate,
            )
        })
        .transpose()?;
    let filename = input
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?
        .to_string_lossy();
    let result = admitted.slice_with_load_settings_processed(
        LoadSettingsSliceRequest {
            filename: &filename,
            catalog_json: None,
            loaded_sources: loaded_sources.as_ref(),
            uptodate: false,
            settings: &settings,
            filaments: &filaments,
            plate: Some(plate),
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            orient: false,
            sliced_plate,
        },
        GenerationMetadata::default(),
        context,
        record_moves,
    );
    Ok((result, substitutions))
}
