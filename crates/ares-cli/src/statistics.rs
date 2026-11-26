use std::{fs, path::Path};

use ares_core::{GCodeProcessorResult, ProcessedGCode};

use crate::failure::CliFailure;

pub(super) fn write(
    output: &Path,
    processed: ProcessedGCode,
    export: bool,
    plate: u32,
) -> Result<(), CliFailure> {
    fs::write(output, processed.gcode)?;
    if export {
        write_sidecar(output, &processed.result, plate)?;
    }
    Ok(())
}

pub(super) fn write_sidecar(
    output: &Path,
    result: &GCodeProcessorResult,
    plate: u32,
) -> Result<(), CliFailure> {
    let path = output.with_extension("statistics.json");
    let bytes = result.statistics().to_json_bytes()?;
    fs::write(&path, bytes).map_err(|_| {
        CliFailure::statistics_write(
            &path,
            plate,
            #[cfg(target_os = "linux")]
            output,
        )
    })?;
    Ok(())
}
