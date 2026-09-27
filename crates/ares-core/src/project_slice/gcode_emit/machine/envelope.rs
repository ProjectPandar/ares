//! `append_machine_envelope` — `print_machine_envelope` (`GCode.cpp:2629-2658`),
//! the Marlin-family M201/203/204/205 limit block after the first M73 line.

use super::first;
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
}
