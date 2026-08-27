use super::*;
use crate::{Point2, gcode_writer::SpiralLiftCommand};

mod acceleration;
mod fan;
mod junction_deviation;
mod retraction;

#[test]
fn travel_to_z_formats_exact_line_and_updates_state() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.travel_to_z_with_comment(0.2, 7200.0, None),
        "G1 Z0.2 F7200\n"
    );

    assert_eq!(writer.current_position(), (0.0, 0.0, 0.2));
    assert_eq!(writer.current_feedrate(), 7200.0);
}

#[test]
fn travel_to_xy_formats_exact_line() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.travel_to_xy_with_comment(Point2::new(0.5, -0.5), 7200.0, None),
        "G1 X0.5 Y-0.5 F7200\n"
    );

    assert_eq!(writer.current_position(), (0.5, -0.5, 0.0));
    assert_eq!(writer.current_feedrate(), 7200.0);
}

#[test]
fn spiral_lift_closes_at_start_xy_for_diagonal_travel() {
    let mut writer = GCodeWriter::new();
    writer.travel_to_xyz_with_comment(Point2::new(1.0, 1.0), 0.2, 7200.0, None);

    let gcode = writer.spiral_lift_with_comment(SpiralLiftCommand {
        start: Point2::new(1.0, 1.0),
        z_start: 0.2,
        z: 0.6,
        slope_radians: 45_f64.to_radians(),
        resolution: 0.01,
        target: Point2::new(3.0, 2.0),
        feedrate: 7200.0,
        comment: Some("spiral lift Z"),
    });

    let final_segment = gcode
        .lines()
        .rfind(|line| line.starts_with("G1 X") && line.contains(" Z"))
        .unwrap();
    assert_eq!(final_segment, "G1 X1 Y1 Z0.6");
    assert_eq!(writer.current_position(), (1.0, 1.0, 0.6));
}

#[test]
fn set_speed_formats_exact_line_and_updates_feedrate() {
    let mut writer = GCodeWriter::new();

    assert_eq!(writer.set_speed_with_comment(3600.0, None), "G1 F3600\n");

    assert_eq!(writer.current_feedrate(), 3600.0);
}

#[test]
fn set_jerk_xy_formats_m205_and_updates_state() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.set_jerk_xy_with_comment(Some(9.125), None),
        "M205 X9.125 Y9.125\n"
    );

    assert_eq!(writer.current_jerk(), 9.125);
}

#[test]
fn set_jerk_xy_suppresses_none_small_and_unchanged_values() {
    let mut writer = GCodeWriter::new();

    assert_eq!(writer.set_jerk_xy_with_comment(None, None), "");
    assert_eq!(writer.set_jerk_xy_with_comment(Some(0.009), None), "");
    assert_eq!(
        writer.set_jerk_xy_with_comment(Some(8.0), None),
        "M205 X8 Y8\n"
    );
    assert_eq!(writer.set_jerk_xy_with_comment(Some(8.0), None), "");
}

#[test]
fn set_jerk_xy_appends_comment() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.set_jerk_xy_with_comment(Some(8.0), Some("adjust jerk")),
        "M205 X8 Y8 ; adjust jerk\n"
    );
}

#[test]
fn command_comments_are_appended_only_when_enabled() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.travel_to_xy_with_comment(Point2::new(1.0, 0.0), 7200.0, Some("travel")),
        "G1 X1 Y0 F7200 ; travel\n"
    );
    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(2.0, 0.0), 0.25, Some("extrude")),
        "G1 X2 Y0 E0.25 ; extrude\n"
    );
    assert_eq!(writer.set_speed_with_comment(3600.0, None), "G1 F3600\n");
}

#[test]
fn preamble_uses_relative_e_by_default() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.preamble(),
        "G90\nG21\nM83 ; use relative distances for extrusion\n"
    );
}

#[test]
fn preamble_uses_absolute_e_and_resets_e_when_configured() {
    let mut writer = GCodeWriter::new();
    writer.set_extrusion_axis_mode(ExtrusionAxisMode::Absolute);

    assert_eq!(
        writer.preamble(),
        "G90\nG21\nM82 ; use absolute distances for extrusion\nG92 E0\n"
    );
}

#[test]
fn active_gcode_flavors_emit_position_units_and_relative_e_mode() {
    for flavor in [
        GCodeFlavor::MarlinLegacy,
        GCodeFlavor::Klipper,
        GCodeFlavor::RepRapFirmware,
        GCodeFlavor::Repetier,
        GCodeFlavor::MarlinFirmware,
    ] {
        let mut writer = GCodeWriter::new();
        writer.set_gcode_flavor(flavor);

        assert_eq!(
            writer.preamble(),
            "G90\nG21\nM83 ; use relative distances for extrusion\n",
            "{flavor:?}"
        );
    }
}

#[test]
fn set_nozzle_temperature_formats_marlin_non_wait_command() {
    let writer = GCodeWriter::new();

    assert_eq!(
        writer.set_nozzle_temperature(215, false, None),
        "M104 S215 ; set nozzle temperature\n"
    );
}

#[test]
fn set_nozzle_temperature_formats_reprap_firmware_non_wait_command() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::RepRapFirmware);

    assert_eq!(
        writer.set_nozzle_temperature(216, false, None),
        "G10 S216 ; set nozzle temperature\n"
    );
}

#[test]
fn set_nozzle_temperature_formats_wait_commands_by_flavor() {
    let mut marlin = GCodeWriter::new();
    marlin.set_gcode_flavor(GCodeFlavor::MarlinLegacy);
    assert_eq!(
        marlin.set_nozzle_temperature(217, true, None),
        "M109 S217 ; set nozzle temperature and wait for it to be reached\n"
    );

    let mut reprap = GCodeWriter::new();
    reprap.set_gcode_flavor(GCodeFlavor::RepRapFirmware);
    assert_eq!(
        reprap.set_nozzle_temperature(218, true, None),
        "G10 S218 ; set nozzle temperature\nM116 ; wait for temperature to be reached\n"
    );
}

#[test]
fn set_nozzle_temperature_skips_wait_for_makerware_and_sailfish() {
    for flavor in [GCodeFlavor::MakerWare, GCodeFlavor::Sailfish] {
        let mut writer = GCodeWriter::new();
        writer.set_gcode_flavor(flavor);

        assert_eq!(writer.set_nozzle_temperature(219, true, None), "");
    }
}

#[test]
fn set_bed_temperature_formats_non_wait_command() {
    let writer = GCodeWriter::new();

    assert_eq!(
        writer.set_bed_temperature(35, false),
        "M140 S35 ; set bed temperature\n"
    );
}

#[test]
fn set_bed_temperature_formats_wait_command() {
    let writer = GCodeWriter::new();

    assert_eq!(
        writer.set_bed_temperature(60, true),
        "M190 S60 ; set bed temperature and wait for it to be reached\n"
    );
}

#[test]
fn set_chamber_temperature_formats_wait_command() {
    let writer = GCodeWriter::new();

    assert_eq!(
        writer.set_chamber_temperature(45, true),
        "M191 S45 ;set chamber_temperature and wait for it to be reached\n"
    );
}

#[test]
fn set_chamber_temperature_formats_non_wait_command() {
    let writer = GCodeWriter::new();

    assert_eq!(
        writer.set_chamber_temperature(0, false),
        "M141 S0;set chamber_temperature\n"
    );
}

#[test]
fn set_exhaust_fan_formats_p3_percent_as_pwm() {
    let writer = GCodeWriter::new();

    assert_eq!(writer.set_exhaust_fan(0), "M106 P3 S0\n");
    assert_eq!(writer.set_exhaust_fan(60), "M106 P3 S153\n");
    assert_eq!(writer.set_exhaust_fan(80), "M106 P3 S204\n");
    assert_eq!(writer.set_exhaust_fan(100), "M106 P3 S255\n");
}

#[test]
fn set_additional_fan_formats_p2_percent_as_floor_pwm() {
    let writer = GCodeWriter::new();

    assert_eq!(writer.set_additional_fan(0), "M106 P2 S0\n");
    assert_eq!(writer.set_additional_fan(1), "M106 P2 S2\n");
    assert_eq!(writer.set_additional_fan(70), "M106 P2 S178\n");
    assert_eq!(writer.set_additional_fan(100), "M106 P2 S255\n");
}

#[test]
fn makerware_preamble_omits_position_units_and_extrusion_mode() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MakerWare);
    writer.set_extrusion_axis_mode(ExtrusionAxisMode::Absolute);

    assert_eq!(writer.preamble(), "");
}

#[test]
fn sailfish_absolute_e_preamble_omits_reset() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Sailfish);
    writer.set_extrusion_axis_mode(ExtrusionAxisMode::Absolute);

    assert_eq!(writer.preamble(), "G90\nG21\n");
}

#[test]
fn marlin_absolute_e_preamble_keeps_reset() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MarlinLegacy);
    writer.set_extrusion_axis_mode(ExtrusionAxisMode::Absolute);

    assert_eq!(
        writer.preamble(),
        "G90\nG21\nM82 ; use absolute distances for extrusion\nG92 E0\n"
    );
}

#[test]
fn relative_e_mode_emits_delta_e_values_and_tracks_total_e() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(1.0, 0.0), 0.123456, None),
        "G1 X1 Y0 E0.12346\n"
    );
    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(2.0, 0.0), 1.0, None),
        "G1 X2 Y0 E1\n"
    );

    assert_eq!(writer.current_e(), 1.123456);
}

#[test]
fn absolute_e_mode_emits_cumulative_e_values() {
    let mut writer = GCodeWriter::new();
    writer.set_extrusion_axis_mode(ExtrusionAxisMode::Absolute);

    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(1.0, 0.0), 0.123456, None),
        "G1 X1 Y0 E0.12346\n"
    );
    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(2.0, 0.0), 1.0, None),
        "G1 X2 Y0 E1.12346\n"
    );

    assert_eq!(writer.current_e(), 1.123456);
}

#[test]
fn zero_delta_extrusion_omits_e_axis() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(1.0, 0.0), 0.0, None),
        "G1 X1 Y0\n"
    );

    assert_eq!(writer.current_e(), 0.0);
}

#[test]
fn effectively_zero_delta_extrusion_omits_e_axis_and_preserves_e() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(1.0, 0.0), f64::EPSILON, None),
        "G1 X1 Y0\n"
    );

    assert_eq!(writer.current_e(), 0.0);
}

#[test]
fn reset_e_is_internal_state_only() {
    let mut writer = GCodeWriter::new();
    writer.extrude_to_xy_with_comment(Point2::new(1.0, 0.0), 0.5, None);

    writer.reset_e();

    assert_eq!(writer.current_e(), 0.0);
    assert_eq!(
        writer.extrude_to_xy_with_comment(Point2::new(2.0, 0.0), 0.25, None),
        "G1 X2 Y0 E0.25\n"
    );
}
