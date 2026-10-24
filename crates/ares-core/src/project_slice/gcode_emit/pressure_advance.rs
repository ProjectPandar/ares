//! Fixed PA command consumer (`GCode.cpp:8048–8053`, `GCodeWriter.cpp:370–391`).

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn append(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
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
        state.adaptive_pa_resets.push((state.adaptive_pa_layer, pa));
        output
            .extend_from_slice(command(pa, settings.printer.gcode.gcode_flavor, is_bbl).as_bytes());
    }
}

pub(super) fn command(pa: f64, flavor: crate::GCodeFlavor, is_bbl: bool) -> String {
    if pa < 0.0 {
        return String::new();
    }
    let value = format_pa(pa);
    match flavor {
        _ if is_bbl => {
            format!("M900 K{value} L1000 M10 ; Override pressure advance value\n")
        }
        crate::GCodeFlavor::Klipper => {
            format!("SET_PRESSURE_ADVANCE ADVANCE={value}; Override pressure advance value\n")
        }
        crate::GCodeFlavor::RepRapFirmware => {
            format!("M572 D0 S{value}; Override pressure advance value\n")
        }
        crate::GCodeFlavor::Repetier => {
            format!("M233 X{value} Y{value} ; Override pressure advance value\n")
        }
        _ => format!("M900 K{value}; Override pressure advance value\n"),
    }
}

fn format_pa(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.to_owned();
    }
    if !value.is_finite() {
        return value.to_string().to_ascii_lowercase();
    }
    // defaultfloat chooses notation using the exponent after rounding.
    let scientific = format!("{value:.3e}");
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific notation");
    let exponent: i32 = exponent.parse().expect("formatted exponent");
    if !(-4..4).contains(&exponent) {
        return format!(
            "{}e{exponent:+03}",
            mantissa.trim_end_matches('0').trim_end_matches('.')
        );
    }
    let decimals = (3 - exponent) as usize;
    let formatted = format!("{value:.decimals$}");
    if decimals == 0 {
        formatted
    } else {
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    }
}
