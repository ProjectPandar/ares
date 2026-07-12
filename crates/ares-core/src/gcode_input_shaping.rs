// Source: OrcaSlicer/src/libslic3r/GCode.cpp
// Source: OrcaSlicer/src/libslic3r/GCodeWriter.cpp

use crate::options::{GCodeFlavor, InputShapingConfig};

pub(crate) fn format_input_shaping(
    flavor: GCodeFlavor,
    emit_machine_limits_to_gcode: bool,
    config: InputShapingConfig,
) -> String {
    if !emit_machine_limits_to_gcode || !config.emit || matches!(flavor, GCodeFlavor::MarlinLegacy)
    {
        return String::new();
    }

    match flavor {
        GCodeFlavor::MarlinFirmware => format_marlin(config),
        GCodeFlavor::RepRapFirmware => format_reprap_firmware(config),
        _ => String::new(),
    }
}

fn format_marlin(config: InputShapingConfig) -> String {
    if config.shaper_type.disables_input_shaping() {
        return "M593 F0.00 D0.000 ; Override input shaping\n".to_owned();
    }

    let mut gcode = set_marlin_axis('X', config.freq_x, config.damp_x);
    gcode.push_str(&set_marlin_axis('Y', config.freq_y, config.damp_y));
    gcode
}

fn set_marlin_axis(axis: char, freq: f64, damp: f64) -> String {
    let mut gcode = format!("M593 {axis}");
    if freq > 0.0 {
        gcode.push_str(&format!(" F{freq:.2}"));
    }
    if damp > 0.0 {
        gcode.push_str(&format!(" D{damp:.3}"));
    }
    gcode.push_str(" ; Override input shaping\n");
    gcode
}

fn format_reprap_firmware(config: InputShapingConfig) -> String {
    if config.shaper_type.disables_input_shaping() {
        return "M593 F0.00 S0.000 ; Override input shaping\n".to_owned();
    }

    let mut params = String::new();
    let shaper_type = config.shaper_type.as_gcode_value();
    if shaper_type != "Default" && shaper_type != "DAA" {
        params.push_str(&format!(" P\"{shaper_type}\""));
    }
    if config.freq_x > 0.0 {
        params.push_str(&format!(" F{:.2}", config.freq_x));
    }
    if config.damp_x > 0.0 {
        params.push_str(&format!(" S{:.3}", config.damp_x));
    }

    if params.is_empty() {
        String::new()
    } else {
        format!("M593{params} ; Override input shaping\n")
    }
}
