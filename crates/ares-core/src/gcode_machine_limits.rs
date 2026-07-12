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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marlin_legacy_formats_machine_envelope_with_legacy_travel_acceleration() {
        let limits = custom_limits();

        assert_eq!(
            format_machine_envelope(GCodeFlavor::MarlinLegacy, limits),
            concat!(
                "M201 X111 Y223 Z334 E444\n",
                "M203 X55 Y67 Z8 E88\n",
                "M204 P901 R803 T901\n",
                "M205 X9.10 Y8.20 Z0.33 E4.40 ; sets the jerk limits, mm/sec\n",
            )
        );
    }

    #[test]
    fn marlin_firmware_formats_machine_envelope_and_junction_deviation() {
        let limits = custom_limits();

        assert_eq!(
            format_machine_envelope(GCodeFlavor::MarlinFirmware, limits),
            concat!(
                "M201 X111 Y223 Z334 E444\n",
                "M203 X55 Y67 Z8 E88\n",
                "M204 P901 R803 T704 ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2\n",
                "M205 X9.10 Y8.20 Z0.33 E4.40 ; sets the jerk limits, mm/sec\n",
                "M205 J0.025\n",
            )
        );
    }

    #[test]
    fn reprap_firmware_formats_machine_envelope_with_minute_speed_units() {
        let limits = custom_limits();

        assert_eq!(
            format_machine_envelope(GCodeFlavor::RepRapFirmware, limits),
            concat!(
                "M201 X111 Y223 Z334 E444\n",
                "M203 X3324 Y3990 Z456 E5304\n",
                "M204 P901 T704 ; sets acceleration (P, T), mm/sec^2\n",
                "M566 X546.00 Y492.00 Z19.80 E264.00 ; sets the jerk limits, mm/min\n",
            )
        );
    }

    #[test]
    fn unsupported_or_disabled_machine_limits_emit_nothing() {
        let mut limits = custom_limits();

        assert_eq!(format_machine_envelope(GCodeFlavor::Klipper, limits), "");
        assert_eq!(format_machine_envelope(GCodeFlavor::Repetier, limits), "");

        limits.emit_to_gcode = false;
        assert_eq!(
            format_machine_envelope(GCodeFlavor::MarlinLegacy, limits),
            ""
        );
    }

    #[test]
    fn marlin_firmware_suppresses_zero_junction_deviation() {
        let mut limits = custom_limits();
        limits.max_junction_deviation = 0.0;

        assert!(!format_machine_envelope(GCodeFlavor::MarlinFirmware, limits).contains("M205 J"));
    }

    fn custom_limits() -> MachineLimits {
        MachineLimits {
            emit_to_gcode: true,
            max_acceleration: [111.4, 222.5, 333.6, 444.4],
            max_speed: [55.4, 66.5, 7.6, 88.4],
            max_acceleration_extruding: 901.2,
            max_acceleration_retracting: 802.5,
            max_acceleration_travel: 703.6,
            max_jerk: [9.1, 8.2, 0.33, 4.4],
            max_junction_deviation: 0.025,
        }
    }
}
