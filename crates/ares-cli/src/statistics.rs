use std::{fs, path::Path};

use ares_core::ProcessedGCode;

use crate::failure::CliFailure;

pub(super) fn write(
    output: &Path,
    processed: ProcessedGCode,
    export: bool,
    plate: u32,
) -> Result<(), CliFailure> {
    fs::write(output, processed.gcode)?;
    if export {
        let path = output.with_extension("statistics.json");
        let bytes = processed.result.statistics().to_json_bytes()?;
        fs::write(&path, bytes).map_err(|_| {
            CliFailure::statistics_write(
                &path,
                plate,
                #[cfg(target_os = "linux")]
                output,
            )
        })?;
    }
    Ok(())
}
