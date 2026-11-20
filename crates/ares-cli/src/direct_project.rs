use std::{fs, io};

use ares_core::{
    CliProjectSliceRequest, CliUpwardMachine, ConfigOverrides, GenerationMetadata, SlicedPlate,
    SlicerContext, UpwardMachineInput,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
};

pub(super) async fn run(args: DirectSliceArgs) -> Result<(), CliFailure> {
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
    if args.load_settings.is_some() || args.load_filaments.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "direct project slicing requires saved presets",
        )
        .into());
    }
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
    let (source, machine) = crate::compatible_machine::load(&bytes, None)?;
    let mut sliced_plate = SlicedPlate::default();
    let mut upward_machines = Vec::new();
    let processed = ares_core::slice_cli_project_plate_validated(
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
        SlicerContext::new(0).with_no_check(args.no_check),
    )
    .await
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
    if args.strict {
        crate::failure::slicing::check_strict(
            #[cfg(target_os = "linux")]
            &output,
            plate,
            &sliced_plate,
            &upward_machines,
            Some((&input.to_string_lossy(), &bytes)),
        )?;
    }
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
    .write(&input.to_string_lossy(), crate::substitutions(&bytes));
    Ok(())
}
