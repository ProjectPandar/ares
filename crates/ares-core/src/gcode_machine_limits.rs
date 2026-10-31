use crate::options::{GCodeFlavor, MachineLimits};

pub(crate) fn format_machine_envelope(flavor: GCodeFlavor, limits: MachineLimits) -> String {
    if !limits.emit_to_gcode
        || !matches!(
            flavor,
            GCodeFlavor::MarlinLegacy | GCodeFlavor::MarlinFirmware | GCodeFlavor::RepRapFirmware
        )
    {
        return String::new();
    }

    let factor = if flavor == GCodeFlavor::RepRapFirmware {
        60.0
    } else {
        1.0
    };
    let travel_acceleration = if flavor == GCodeFlavor::MarlinLegacy {
        round_machine_limit(limits.max_acceleration_extruding)
    } else {
        round_machine_limit(limits.max_acceleration_travel)
    };
    let mut gcode = String::new();

    gcode.push_str(&format!(
        "M201 X{} Y{} Z{} E{}\n",
        round_machine_limit(limits.max_acceleration[0]),
        round_machine_limit(limits.max_acceleration[1]),
        round_machine_limit(limits.max_acceleration[2]),
        round_machine_limit(limits.max_acceleration[3])
    ));
    gcode.push_str(&format!(
        "M203 X{} Y{} Z{} E{}\n",
        round_machine_limit(limits.max_speed[0] * factor),
        round_machine_limit(limits.max_speed[1] * factor),
        round_machine_limit(limits.max_speed[2] * factor),
        round_machine_limit(limits.max_speed[3] * factor)
    ));
    match flavor {
        GCodeFlavor::RepRapFirmware => gcode.push_str(&format!(
            "M204 P{} T{} ; sets acceleration (P, T), mm/sec^2\n",
            round_machine_limit(limits.max_acceleration_extruding),
            travel_acceleration
        )),
        GCodeFlavor::MarlinFirmware => gcode.push_str(&format!(
            "M204 P{} R{} T{} ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2\n",
            round_machine_limit(limits.max_acceleration_extruding),
            round_machine_limit(limits.max_acceleration_retracting),
            round_machine_limit(limits.max_acceleration_travel)
        )),
        GCodeFlavor::MarlinLegacy => gcode.push_str(&format!(
            "M204 P{} R{} T{}\n",
            round_machine_limit(limits.max_acceleration_extruding),
            round_machine_limit(limits.max_acceleration_retracting),
            travel_acceleration
        )),
        _ => unreachable!("unsupported flavor returned before M204 formatting"),
    }

    if flavor == GCodeFlavor::RepRapFirmware {
        gcode.push_str(&format!(
            "M566 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/min\n",
            limits.max_jerk[0] * factor,
            limits.max_jerk[1] * factor,
            limits.max_jerk[2] * factor,
            limits.max_jerk[3] * factor
        ));
    } else {
        gcode.push_str(&format!(
            "M205 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/sec\n",
            limits.max_jerk[0], limits.max_jerk[1], limits.max_jerk[2], limits.max_jerk[3]
        ));
    }

    if flavor == GCodeFlavor::MarlinFirmware && limits.max_junction_deviation > 0.0 {
        gcode.push_str(&format!("M205 J{:.3}\n", limits.max_junction_deviation));
    }

    gcode
}

fn round_machine_limit(value: f64) -> u32 {
    (value + 0.5).floor() as u32
}
