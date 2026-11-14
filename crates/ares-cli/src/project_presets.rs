use std::{fs, io, path::Path};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
};

pub(super) async fn slice(
    args: &SliceArgs,
    catalog_path: &Path,
    input: &[u8],
    overrides: &ares_core::ConfigOverrides,
    context: ares_core::SlicerContext<'_>,
) -> Result<(), CliFailure> {
    if args
        .input
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("3mf")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--project-preset-catalog requires 3MF project input",
        )
        .into());
    }
    // CLI:1682–1690 admits the embedded project before reading supplied presets.
    ares_core::validate_load_settings_input(input).map_err(|error| {
        slice_error_to_failure(
            error,
            &args.output,
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        )
    })?;
    let catalog = fs::read_to_string(catalog_path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "cannot read project preset catalog {}: {error}",
                catalog_path.display()
            ),
        )
    })?;
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
    let selection = if args.uptodate {
        r#"{"from_project":true,"uptodate":true}"#
    } else {
        r#"{"from_project":true,"uptodate":false}"#
    };
    let request = ares_core::ProfileSliceRequest {
        filename,
        catalog_json: &catalog,
        selection_json: selection,
        plate: args.plate,
        overrides,
        placement: ares_core::Placement::None,
    };
    let metadata = ares_core::GenerationMetadata::default();
    let input_file = args.input.to_string_lossy();
    let source = ares_core::UpwardMachineSource::from_project(input)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let mut upward_report = Vec::new();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let processed = match ares_core::slice_cli_with_profiles_validated(
        input,
        ares_core::CliProfileSliceRequest {
            profile: request,
            record_moves: args.dump_moves.is_some(),
            sliced_plate: &mut sliced_plate,
            upward: ares_core::CliUpwardMachine {
                source,
                input: ares_core::UpwardMachineInput::Embedded,
                report: &mut upward_report,
            },
        },
        metadata,
        context,
    )
    .await
    {
        Ok(processed) => processed,
        Err(error) => {
            crate::failure::slicing::check(
                args,
                &sliced_plate,
                &input_file,
                input,
                &upward_report,
            )?;
            let failure = slice_error_to_failure(
                error,
                &args.output,
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
            .with_upward_report(upward_report)
            .with_config_substitutions(&input_file, crate::substitutions(input));
            #[cfg(target_os = "linux")]
            let failure =
                failure.with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings);
            return Err(failure);
        }
    };
    crate::failure::slicing::check(args, &sliced_plate, &input_file, input, &upward_report)?;
    if let Some(path) = &args.dump_moves {
        fs::write(path, crate::moves_csv::format(&processed.result))?;
    }
    fs::write(&args.output, processed.gcode)?;
    crate::report_success(
        &input_file,
        crate::substitutions(input),
        #[cfg(target_os = "linux")]
        crate::result_json::Success {
            output: &args.output,
            plate: args.plate.unwrap_or(1),
            plate_index: args.plate.unwrap_or(0),
            sliced_plate,
            upward_machines: &upward_report,
        },
    );
    Ok(())
}
