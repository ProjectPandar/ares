use std::{fs, io};

use ares_core::{
    CliProjectSliceRequest, CliUpwardMachine, ConfigOverrides, GenerationMetadata,
    LoadSettingsFile, LoadSettingsSliceRequest, Placement, SlicedPlate, SlicerContext,
    UpwardMachineInput,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    read_preset_files, split_native_list,
};

pub(super) async fn run(mut args: DirectSliceArgs) -> Result<(), CliFailure> {
    let plate = u32::try_from(
        args.selection
            .expect("--slice is required without a subcommand"),
    )
    .ok()
    .filter(|&plate| plate > 0)
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
    let output = args
        .outputdir
        .expect("--outputdir is required without a subcommand")
        .join(format!("plate_{plate}.gcode"));
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
    let context = crate::custom_gcodes::load(
        SlicerContext::new(0)
            .with_no_check(args.no_check)
            .with_enable_timelapse(args.enable_timelapse),
        args.load_custom_gcodes.as_deref(),
        &output,
        plate as i32 - 1,
    )?;
    let mut sliced_plate = SlicedPlate::default();
    let mut upward_machines = Vec::new();
    let processed = if let Some((settings, filaments)) = presets {
        if let Err(error) = ares_core::validate_load_settings_input(&bytes) {
            Err(error)
        } else {
            let settings = read_preset_files(&split_native_list(settings))?;
            let filaments = read_preset_files(&split_native_list(filaments))?;
            let settings = settings
                .iter()
                .map(|(label, json)| LoadSettingsFile { label, json })
                .collect::<Vec<_>>();
            let filaments = filaments
                .iter()
                .map(|(label, json)| LoadSettingsFile { label, json })
                .collect::<Vec<_>>();
            let processed = ares_core::slice_with_load_settings_processed(
                &bytes,
                LoadSettingsSliceRequest {
                    filename: &input.file_name().expect("3MF filename").to_string_lossy(),
                    catalog_json: None,
                    uptodate: false,
                    settings: &settings,
                    filaments: &filaments,
                    plate: Some(plate),
                    overrides: &ConfigOverrides::default(),
                    placement: Placement::None,
                    orient: false,
                    sliced_plate: &mut sliced_plate,
                },
                GenerationMetadata::default(),
                context,
                args.export_statistics,
            )
            .await;
            upward_machines = std::mem::take(&mut sliced_plate.upward_machines);
            processed
        }
    } else {
        let (source, machine) = crate::compatible_machine::load(&bytes, None)?;
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
    .map_err(|error| {
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
        .with_upward_report(upward_machines.clone());
        #[cfg(target_os = "linux")]
        let failure = failure.with_validation_detail(detail);
        failure
    })?;
    let input_file = input.to_string_lossy();
    let project = Some((input_file.as_ref(), bytes.as_slice()));
    if args.strict {
        crate::failure::slicing::check_strict(
            #[cfg(target_os = "linux")]
            &output,
            plate,
            &sliced_plate,
            &upward_machines,
            project,
        )?;
    }
    crate::failure::slicing::check_direct(
        (&mut args.diagnostics, args.no_check),
        #[cfg(target_os = "linux")]
        &output,
        (plate, &sliced_plate),
        &upward_machines,
        project,
    )?;
    crate::statistics::write(&output, processed, args.export_statistics, plate)?;
    #[cfg(target_os = "linux")]
    crate::result_json::Success {
        strict_mode: args.strict,
        output: &output,
        plate,
        plate_index: plate,
        sliced_plate,
        upward_machines: &upward_machines,
    }
    .write(&input_file, crate::substitutions(&bytes));
    Ok(())
}
