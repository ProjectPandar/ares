use std::{fs, io, path::Path};

use ares_core::{CustomGCodeInfo, SlicerContext};

use crate::failure::CliFailure;

pub(super) fn load<'a>(
    context: SlicerContext<'a>,
    path: Option<&Path>,
    output: &Path,
    plate_index: i32,
) -> Result<SlicerContext<'a>, CliFailure> {
    let Some(path) = path.filter(|path| !path.as_os_str().is_empty()) else {
        return Ok(context);
    };
    let malformed = |detail: &str| {
        CliFailure::invalid_preset_file(output).with_stderr(format!(
            "run:Loading custom-gcode file \"{}\" failed: {detail}\n",
            path.display()
        ))
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                path,
                #[cfg(target_os = "linux")]
                output,
            )
            .with_stderr(format!(
                "run: can not find custom_gcode file: {}\n",
                path.display()
            )));
        }
        Err(error) => return Err(malformed(&error.to_string())),
    };
    let info = CustomGCodeInfo::from_json_bytes(&bytes).map_err(|detail| malformed(&detail))?;
    Ok(context.with_custom_gcodes(plate_index, info))
}
