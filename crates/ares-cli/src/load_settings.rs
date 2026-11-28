#[cfg(target_os = "linux")]
use crate::result_json;
use std::{fs, io};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    preset_sources::{as_load_settings_files, read_preset_files},
};

pub(super) async fn slice(
    args: &SliceArgs,
    admitted: ares_core::CliPresetProject<'_>,
    overrides: &ares_core::ConfigOverrides,
    (load_settings, load_filaments): (&[&str], &[&str]),
    (context, _warnings): (ares_core::SlicerContext<'_>, &mut crate::PositionalWarnings),
) -> Result<(), CliFailure> {
    let filename = args
        .input()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
    let catalog = args
        .project_preset_catalog
        .as_ref()
        .map(fs::read_to_string)
        .transpose()?;
    let settings_files = read_preset_files(load_settings)?;
    let filament_files = read_preset_files(load_filaments)?;
    let settings = as_load_settings_files(&settings_files);
    let filaments = as_load_settings_files(&filament_files);
    let loaded_sources = crate::preset_sources::load(
        args.datadir.as_deref(),
        settings.iter().chain(&filaments).copied(),
    )
    .map_err(|error| match error.downcast::<ares_core::SliceError>() {
        Ok(error) => slice_error_to_failure(
            *error,
            args.output(),
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        ),
        Err(error) => CliFailure::config_file_error(
            #[cfg(target_os = "linux")]
            args.output(),
            format!("{error}\n"),
        ),
    })?;
    let refresh = crate::native_refresh::load_admitted(args, &admitted, load_filaments)?;
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let gcode = match admitted.slice_with_load_settings(
        ares_core::LoadSettingsSliceRequest {
            orient: args.orient == Some(1),
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
        refresh.as_ref(),
        ares_core::GenerationMetadata::default(),
        context,
    ) {
        Ok(gcode) => gcode,
        Err(error) => {
            crate::failure::slicing::check(args, &sliced_plate, &sliced_plate.upward_machines)?;
            let failure = slice_error_to_failure(
                error,
                args.output(),
                #[cfg(target_os = "linux")]
                // The request above selects args.plate; CLI :5681-5690
                // selects that same one-based plate (default first), and
                // :5723/:433 records index+1 for the -50 result.
                args.plate.unwrap_or(1),
            );
            #[cfg(target_os = "linux")]
            let failure = failure
                .with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings)
                .with_upward_report(sliced_plate.upward_machines);
            return Err(failure);
        }
    };
    crate::failure::slicing::check(args, &sliced_plate, &sliced_plate.upward_machines)?;
    fs::write(args.output(), gcode)?;
    #[cfg(target_os = "linux")]
    result_json::Success {
        strict_mode: false,
        output: args.output(),
        plate: args.plate.unwrap_or(1),
        plate_index: args.plate.unwrap_or(0),
        upward_machines: &std::mem::take(&mut sliced_plate.upward_machines),
        downward_machines: &[],
        sliced_plate,
    }
    .write(_warnings.take());
    Ok(())
}
