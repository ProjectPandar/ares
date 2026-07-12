use crate::{SliceError, SliceOptions, options::GCodeFlavor};

pub(crate) fn first_progress_line(options: &SliceOptions) -> Result<String, SliceError> {
    progress_line(options, 0)
}

pub(crate) fn last_progress_line(options: &SliceOptions) -> Result<String, SliceError> {
    progress_line(options, 100)
}

fn progress_line(options: &SliceOptions, percent: u8) -> Result<String, SliceError> {
    if options.disable_m73()? {
        return Ok(String::new());
    }

    let mut line = format!("M73 P{percent} R0\n");
    if options.silent_mode()? && supports_stealth_m73(options.gcode_flavor()?) {
        line.push_str(&format!("M73 Q{percent} S0\n"));
    }
    Ok(line)
}

fn supports_stealth_m73(flavor: GCodeFlavor) -> bool {
    matches!(
        flavor,
        GCodeFlavor::MarlinLegacy | GCodeFlavor::MarlinFirmware
    )
}
