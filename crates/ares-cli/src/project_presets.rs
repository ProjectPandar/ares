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
    let context = ares_core::SlicerContext::new(0);
    let result = if args.dump_moves.is_some() {
        ares_core::slice_with_profiles_processed_validated(input, request, metadata, context)
            .await
            .map(|processed| (processed.gcode, Some(processed.result)))
    } else {
        ares_core::slice_with_profiles_validated(input, request, metadata, context)
            .await
            .map(|gcode| (gcode, None))
    };
    let (gcode, moves) = result.map_err(|error| {
        slice_error_to_failure(
            error,
            &args.output,
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        )
    })?;
    if let (Some(path), Some(moves)) = (&args.dump_moves, moves) {
        fs::write(path, crate::moves_csv::format(&moves))?;
    }
    fs::write(&args.output, gcode)?;
    Ok(())
}
