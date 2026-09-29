//! `append_machine_envelope` — `print_machine_envelope` (`GCode.cpp:2629-2658`),
//! the Marlin-family M201/203/204/205 limit block after the first M73 line.

use super::first;
use crate::options::InputShaperType;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn append_machine_envelope(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
) {
    let machine = &traversal.resolved.views.full.printer.machine;
    let flavor = traversal.resolved.views.full.printer.gcode.gcode_flavor;
    let rrf = flavor == crate::GCodeFlavor::RepRapFirmware;
    let factor: f64 = if rrf { 60.0 } else { 1.0 };
    // Upstream converts the envelope integers with `int(value + 0.5)`
    // (`GCode.cpp:3945-3980`).
    let half_up = |value: f64| (value + 0.5).floor() as i64;
    output.extend_from_slice(
        format!(
            "M201 X{} Y{} Z{} E{}\n",
            half_up(first(&machine.machine_max_acceleration_x)),
            half_up(first(&machine.machine_max_acceleration_y)),
            half_up(first(&machine.machine_max_acceleration_z)),
            half_up(first(&machine.machine_max_acceleration_e)),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M203 X{} Y{} Z{} E{}\n",
            half_up(first(&machine.machine_max_speed_x) * factor),
            half_up(first(&machine.machine_max_speed_y) * factor),
            half_up(first(&machine.machine_max_speed_z) * factor),
            half_up(first(&machine.machine_max_speed_e) * factor),
        )
        .as_bytes(),
    );
    // Legacy Marlin exports travel acceleration equal to printing
    // acceleration; other flavors use the travel value.
    let travel_acceleration = if flavor == crate::GCodeFlavor::MarlinLegacy {
        first(&machine.machine_max_acceleration_extruding)
    } else {
        first(&machine.machine_max_acceleration_travel)
    };
    if rrf {
        output.extend_from_slice(
            format!(
                "M204 P{} T{} ; sets acceleration (P, T), mm/sec^2\n",
                half_up(first(&machine.machine_max_acceleration_extruding)),
                half_up(travel_acceleration),
            )
            .as_bytes(),
        );
    } else if flavor == crate::GCodeFlavor::MarlinFirmware {
        output.extend_from_slice(
            format!(
                "M204 P{} R{} T{} ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2\n",
                half_up(first(&machine.machine_max_acceleration_extruding)),
                half_up(first(&machine.machine_max_acceleration_retracting)),
                half_up(travel_acceleration),
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "M204 P{} R{} T{}\n",
                half_up(first(&machine.machine_max_acceleration_extruding)),
                half_up(first(&machine.machine_max_acceleration_retracting)),
                half_up(travel_acceleration),
            )
            .as_bytes(),
        );
    }
    if rrf {
        output.extend_from_slice(
            format!(
                "M566 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/min\n",
                first(&machine.machine_max_jerk_x) * factor,
                first(&machine.machine_max_jerk_y) * factor,
                first(&machine.machine_max_jerk_z) * factor,
                first(&machine.machine_max_jerk_e) * factor,
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "M205 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/sec\n",
                first(&machine.machine_max_jerk_x),
                first(&machine.machine_max_jerk_y),
                first(&machine.machine_max_jerk_z),
                first(&machine.machine_max_jerk_e),
            )
            .as_bytes(),
        );
    }
    let junction_deviation = first(&machine.machine_max_junction_deviation);
    if flavor == crate::GCodeFlavor::MarlinFirmware && junction_deviation > 0.0 {
        output.extend_from_slice(format!("M205 J{junction_deviation:.3}\n").as_bytes());
    }

    // `GCode.cpp:3991-3998` — input-shaping override lines, inside the
    // same `emit_machine_limits_to_gcode` gate as the M201-M205 block,
    // skipped only for legacy Marlin. The X call carries the shaper type;
    // the Y call is skipped for RepRapFirmware and when the type is
    // `Disable`.
    if machine.input_shaping_emit.0 && flavor != crate::GCodeFlavor::MarlinLegacy {
        let disable = machine.input_shaping_type == InputShaperType::Disable;
        output.extend_from_slice(
            set_input_shaping(
                flavor,
                'X',
                machine.input_shaping_damp_x.0,
                machine.input_shaping_freq_x.0,
                machine.input_shaping_type,
            )
            .as_bytes(),
        );
        if flavor != crate::GCodeFlavor::RepRapFirmware && !disable {
            // Upstream passes `type = ""` for the Y call — model that as
            // `None` so no `SHAPER_TYPE`/`P"..."` params are emitted.
            output.extend_from_slice(
                set_input_shaping_y(
                    flavor,
                    machine.input_shaping_damp_y.0,
                    machine.input_shaping_freq_y.0,
                )
                .as_bytes(),
            );
        }
    }
}

fn set_input_shaping_y(flavor: crate::GCodeFlavor, damp: f64, freq: f64) -> String {
    let mut params = String::new();
    match flavor {
        crate::GCodeFlavor::Klipper => {
            if freq > 0.0 {
                params.push_str(&format!(" SHAPER_FREQ_Y={freq:.2}"));
            }
            if damp > 0.0 {
                params.push_str(&format!(" DAMPING_RATIO_Y={damp:.3}"));
            }
            if params.is_empty() {
                String::new()
            } else {
                format!("SET_INPUT_SHAPER{params}\n")
            }
        }
        crate::GCodeFlavor::MarlinFirmware => {
            params.push_str(" Y");
            if freq > 0.0 {
                params.push_str(&format!(" F{freq:.2}"));
            }
            if damp > 0.0 {
                params.push_str(&format!(" D{damp:.3}"));
            }
            format!("M593{params}\n")
        }
        _ => String::new(),
    }
}

/// `GCodeWriter::set_input_shaping` (`GCodeWriter.cpp:395-481`): one
/// `M593`/`SET_INPUT_SHAPER` line per flavor. The `full_gcode_comment`
/// suffix is only emitted when G-code comments are enabled
/// (`GCodeWriter.cpp:473-477`), which the smoke fixture leaves off.
/// `GCodeWriter::set_input_shaping` (`GCodeWriter.cpp:395-481`) for the
/// X axis / disable call: `Disable` zeroes freq/damp, switches to the
/// all-axis `A`, and resets the type — but `disable` stays set so the
/// `|| disable` guards still emit the zeroed params. `full_gcode_comment`
/// suffix (`GCodeWriter.cpp:473-477`) is omitted — the smoke fixture
/// leaves comments off.
fn set_input_shaping(
    flavor: crate::GCodeFlavor,
    axis: char,
    damp: f64,
    freq: f64,
    shaper_type: InputShaperType,
) -> String {
    let disable = shaper_type == InputShaperType::Disable;
    let (axis, damp, freq, shaper_type) = if disable {
        ('A', 0.0, 0.0, InputShaperType::Default)
    } else {
        (axis, damp, freq, shaper_type)
    };
    let mut params = String::new();
    match flavor {
        crate::GCodeFlavor::Klipper => {
            if shaper_type != InputShaperType::Default {
                params.push_str(&format!(" SHAPER_TYPE={}", shaper_type.as_str()));
            }
            if axis != 'A' {
                if freq > 0.0 {
                    params.push_str(&format!(" SHAPER_FREQ_{axis}={freq:.2}"));
                }
                if damp > 0.0 {
                    params.push_str(&format!(" DAMPING_RATIO_{axis}={damp:.3}"));
                }
            } else {
                if freq > 0.0 || disable {
                    params.push_str(&format!(" SHAPER_FREQ_X={freq:.2} SHAPER_FREQ_Y={freq:.2}"));
                }
                if damp > 0.0 || disable {
                    params.push_str(&format!(
                        " DAMPING_RATIO_X={damp:.3} DAMPING_RATIO_Y={damp:.3}"
                    ));
                }
            }
            if params.is_empty() {
                String::new()
            } else {
                format!("SET_INPUT_SHAPER{params}\n")
            }
        }
        crate::GCodeFlavor::RepRapFirmware => {
            if shaper_type != InputShaperType::Default && shaper_type != InputShaperType::Daa {
                params.push_str(&format!(" P\"{}\"", shaper_type.as_str()));
            }
            if freq > 0.0 || disable {
                params.push_str(&format!(" F{freq:.2}"));
            }
            if damp > 0.0 || disable {
                params.push_str(&format!(" S{damp:.3}"));
            }
            if params.is_empty() {
                String::new()
            } else {
                format!("M593{params}\n")
            }
        }
        crate::GCodeFlavor::MarlinFirmware => {
            if axis != 'A' {
                params.push_str(&format!(" {axis}"));
            }
            if freq > 0.0 || disable {
                params.push_str(&format!(" F{freq:.2}"));
            }
            if damp > 0.0 || disable {
                params.push_str(&format!(" D{damp:.3}"));
            }
            if params.is_empty() {
                String::new()
            } else {
                format!("M593{params}\n")
            }
        }
        _ => String::new(),
    }
}
