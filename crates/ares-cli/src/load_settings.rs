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
    // `OrcaSlicer.cpp:1682-1690`: the 3MF load block admits the
    // project before `:1953-2024` reads the preset files — probe the
    // admission first so the embedded `-19` gate wins over the
    // adapter's missing-file `-5`.
    if let Err(error) = ares_core::validate_load_settings_input(input) {
        return Err(slice_error_to_failure(
            error,
            &args.output,
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        ));
    }
    let catalog = args
        .project_preset_catalog
        .as_ref()
        .map(fs::read_to_string)
        .transpose()?;
    let settings = read_preset_files(load_settings)?;
    let filaments = read_preset_files(load_filaments)?;
    let settings = settings
        .iter()
        .map(
            |(label, json, source_identity)| ares_core::LoadSettingsFile {
                label,
                json,
                source_identity: source_identity.as_ref(),
            },
        )
        .collect::<Vec<_>>();
    let filaments = filaments
        .iter()
        .map(
            |(label, json, source_identity)| ares_core::LoadSettingsFile {
                label,
                json,
                source_identity: source_identity.as_ref(),
            },
        )
        .collect::<Vec<_>>();
    let loaded_sources = crate::preset_sources::load(
        args.datadir.as_deref(),
        settings.iter().chain(&filaments).copied(),
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
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let gcode = match ares_core::slice_with_load_settings(
        input,
        ares_core::LoadSettingsSliceRequest {
            orient: args.orient == 1,
            filename,
            catalog_json: catalog.as_deref(),
            loaded_sources: Some(&loaded_sources),
            uptodate: args.uptodate,
            settings: &settings,
            filaments: &filaments,
            plate: args.plate,
            overrides,
            placement: ares_core::Placement::None,
            sliced_plate: &mut sliced_plate,
        },
        ares_core::GenerationMetadata::default(),
        context,
    )
    .await
    {
        Ok(gcode) => gcode,
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
            downward_machines: &[],
            sliced_plate,
        },
    );
    Ok(())
}
