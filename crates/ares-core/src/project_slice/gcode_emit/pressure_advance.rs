//! Fixed PA command consumer (`GCode.cpp:8048–8053`, `GCodeWriter.cpp:370–391`).

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn append(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    filament: usize,
) {
    let settings = &traversal.resolved.views.full;
    let is_bbl = super::tags::Tags::of(traversal).is_bbl();
    let enabled = settings
        .filament
        .gcode
        .enable_pressure_advance
        .0
        .get(filament)
        .or_else(|| settings.filament.gcode.enable_pressure_advance.0.first())
        .is_some_and(|value| value.0);
    if enabled {
        let pa = settings
            .filament
            .gcode
            .pressure_advance
            .0
            .get(filament)
            .or_else(|| settings.filament.gcode.pressure_advance.0.first())
            .map_or(0.0, |value| value.0);
        if pa >= 0.0 {
            let value = format_pa(pa);
            let line = match settings.printer.gcode.gcode_flavor {
                crate::GCodeFlavor::Klipper => {
                    format!(
                        "SET_PRESSURE_ADVANCE ADVANCE={value}; Override pressure advance value\n"
                    )
                }
                crate::GCodeFlavor::RepRapFirmware => {
                    format!("M572 D0 S{value}; Override pressure advance value\n")
                }
                crate::GCodeFlavor::Repetier => {
                    format!("M233 X{value} Y{value} ; Override pressure advance value\n")
                }
                _ if is_bbl => {
                    format!("M900 K{value} L1000 M10 ; Override pressure advance value\n")
                }
                _ => format!("M900 K{value}; Override pressure advance value\n"),
            };
            output.extend_from_slice(line.as_bytes());
        }
    }
}

fn format_pa(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let decimals = (3 - magnitude).max(0) as usize;
    let formatted = format!("{value:.decimals$}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
