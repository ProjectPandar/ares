#[cfg(target_os = "linux")]
use crate::result_json;
use std::{fs, io};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    read_preset_files, report_success, substitutions,
};

pub(super) async fn slice(
    args: &SliceArgs,
    input: &[u8],
    overrides: &ares_core::ConfigOverrides,
    (load_settings, load_filaments): (&[&str], &[&str]),
    context: ares_core::SlicerContext<'_>,
) -> Result<(), CliFailure> {
    let input_file = args.input.to_string_lossy().into_owned();
    let filename = args
        .input
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let gcode = match ares_core::slice_cli_preset_project(
        input,
        ares_core::CliPresetSliceRequest {
            plate: args.plate,
            overrides,
            orient: args.orient == 1,
            uptodate: args.uptodate,
            record_moves: false,
            sliced_plate: &mut sliced_plate,
        },
        context,
        |error| {
            slice_error_to_failure(
                error,
                &args.output,
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
        },
        || {
            let catalog = args
                .project_preset_catalog
                .as_ref()
                .map(fs::read_to_string)
                .transpose()?;
            let settings = read_preset_files(load_settings)?;
            let filaments = read_preset_files(load_filaments)?;
            let loaded_sources = crate::preset_sources::load(
                args.datadir.as_deref(),
                settings
                    .iter()
                    .chain(&filaments)
                    .map(|(label, json, identity)| ares_core::LoadSettingsFile {
                        label,
                        json,
                        source_identity: identity.as_ref(),
                    }),
            )
            .map_err(|error| match error.downcast::<ares_core::SliceError>() {
                Ok(error) => slice_error_to_failure(
                    *error,
                    &args.output,
                    #[cfg(target_os = "linux")]
                    args.plate.unwrap_or(1),
                ),
                Err(error) => CliFailure::config_file_error(
                    #[cfg(target_os = "linux")]
                    &args.output,
                    format!("{error}\n"),
                ),
            })?;
            Ok(ares_core::CliPresetFiles::LoadSettings {
                filename,
                catalog,
                settings,
                filaments,
                loaded_sources: Box::new(loaded_sources),
            })
        },
    )? {
        Ok(processed) => processed.gcode,
        Err(error) => {
            crate::failure::slicing::check(
                args,
                &sliced_plate,
                &input_file,
                input,
                &sliced_plate.upward_machines,
            )?;
            let failure = slice_error_to_failure(
                error,
                &args.output,
                #[cfg(target_os = "linux")]
                // The request above selects args.plate; CLI :5681-5690
                // selects that same one-based plate (default first), and
                // :5723/:433 records index+1 for the -50 result.
                args.plate.unwrap_or(1),
            )
            .with_config_substitutions(&input_file, substitutions(input));
            #[cfg(target_os = "linux")]
            let failure = failure
                .with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings)
                .with_upward_report(sliced_plate.upward_machines);
            return Err(failure);
        }
    };
    crate::failure::slicing::check(
        args,
        &sliced_plate,
        &input_file,
        input,
        &sliced_plate.upward_machines,
    )?;
    fs::write(&args.output, gcode)?;
    report_success(
        &input_file,
        substitutions(input),
        #[cfg(target_os = "linux")]
        result_json::Success {
            output: &args.output,
            plate: args.plate.unwrap_or(1),
            plate_index: args.plate.unwrap_or(0),
            upward_machines: &std::mem::take(&mut sliced_plate.upward_machines),
            sliced_plate,
        },
    );
    Ok(())
}
