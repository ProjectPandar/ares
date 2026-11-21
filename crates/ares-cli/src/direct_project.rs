use std::{fs, io, path::Path};

use ares_core::{
    CliProjectSliceRequest, CliUpwardMachine, ConfigOverrides, GenerationMetadata,
    LoadSettingsFile, LoadSettingsSliceRequest, Placement, ProcessedGCode, SliceError, SlicedPlate,
    SlicerContext, UpwardMachineInput, UpwardMachineSource,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    preset_sources::read_preset_files,
    split_native_list,
};

pub(super) async fn run(mut args: DirectSliceArgs) -> Result<(), CliFailure> {
    let plate = u32::try_from(
        args.selection
            .expect("--slice is required without a subcommand"),
    )
    .ok()
    .ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "select a project plate with --slice <n>",
        )
    })?;
    let presets = match (
        args.load_settings.as_deref(),
        args.load_filaments.as_deref(),
    ) {
        (None, None) => None,
        (Some(settings), Some(filaments)) => Some((settings, filaments)),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "direct project preset replacement requires --load-settings and --load-filaments",
            )
            .into());
        }
    };
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
        return crate::slice_plates::run_project(
            crate::slice_plates::PlateArgs {
                input,
                directory: Some(directory),
                actions: &args.project_actions,
                settings: args.load_settings.as_deref(),
                filaments: args.load_filaments.as_deref(),
                catalog: None,
                datadir: None,
                resources: None,
                uptodate: false,
                orient: false,
            },
            0,
            &bytes,
            &ConfigOverrides::default(),
            args.project_actions.context(SlicerContext::new(0)),
        );
    }
    enum ProjectPresets<'a> {
        Replace(&'a str, &'a str),
        Saved(Option<UpwardMachineSource>, Option<Vec<u8>>),
    }
    let presets = match presets {
        Some((settings, filaments)) => ProjectPresets::Replace(settings, filaments),
        None => {
            let resources = crate::installed_resources::resolve(None)?;
            let (source, machine) = crate::compatible_machine::load(&bytes, &resources)?;
            ProjectPresets::Saved(source, machine)
        }
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
    let mut sliced_plate = SlicedPlate::default();
    let mut upward_machines = Vec::new();
    let processed = match presets {
        ProjectPresets::Replace(settings, filaments) => {
            let processed = slice_with_replacement(
                (input, &bytes, plate),
                (settings, filaments),
                context,
                &mut sliced_plate,
                args.export_statistics,
            )
            .await?;
            upward_machines = std::mem::take(&mut sliced_plate.upward_machines);
            processed
        }
        ProjectPresets::Saved(source, machine) => {
            ares_core::slice_cli_project_plate_validated(
                &bytes,
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
                GenerationMetadata::default(),
                context,
            )
            .await
        }
    }
    .map_err(|error| {
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
                .with_config_substitutions(&input.to_string_lossy(), crate::substitutions(&bytes))
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
    let project = Some((input_file.as_ref(), bytes.as_slice()));
    crate::failure::slicing::check_direct(
        (&mut args.diagnostics, args.no_check, args.strict),
        #[cfg(target_os = "linux")]
        &output,
        (plate, &sliced_plate),
        &upward_machines,
        project,
    )
    .map_err(|failure| failure.with_downward_report(downward_machines.clone()))?;
    crate::statistics::write(&output, processed, args.export_statistics, plate)?;
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
    .write(&input_file, crate::substitutions(&bytes));
    Ok(())
}

async fn slice_with_replacement(
    (input, bytes, plate): (&Path, &[u8], u32),
    (settings, filaments): (&str, &str),
    context: SlicerContext<'_>,
    sliced_plate: &mut SlicedPlate,
    record_moves: bool,
) -> Result<Result<ProcessedGCode, SliceError>, CliFailure> {
    if let Err(error) = context.validate_load_settings_input(bytes) {
        return Ok(Err(error));
    }
    let settings = read_preset_files(&split_native_list(settings))?;
    let filaments = read_preset_files(&split_native_list(filaments))?;
    let settings = settings
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect::<Vec<_>>();
    let filaments = filaments
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect::<Vec<_>>();
    let filename = input
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?
        .to_string_lossy();
    Ok(ares_core::slice_with_load_settings_processed(
        bytes,
        LoadSettingsSliceRequest {
            filename: &filename,
            catalog_json: None,
            loaded_sources: None,
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
    )
    .await)
}
