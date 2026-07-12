use super::*;

#[test]
fn firmware_retract_unretract_use_g10_g11_by_default() {
    let mut writer = GCodeWriter::new();

    assert_eq!(writer.firmware_retract(), "G10 ; retract\n");
    assert_eq!(writer.firmware_unretract(), "G11 ; unretract\n");
    assert_eq!(writer.current_e(), 0.0);
}

#[test]
fn firmware_retract_unretract_use_machinekit_g22_g23() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Machinekit);

    assert_eq!(writer.firmware_retract(), "G22 ; retract\n");
    assert_eq!(writer.firmware_unretract(), "G23 ; unretract\n");
    assert_eq!(writer.current_e(), 0.0);
}

#[test]
fn extrude_to_xy_with_feedrate_formats_inline_feedrate_and_updates_state() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.extrude_to_xy_with_feedrate_and_comment(
            Point2::new(1.0, 0.0),
            -0.4,
            7200.0,
            Some("wipe and retract"),
        ),
        "G1 X1 Y0 E-0.4 F7200 ; wipe and retract\n"
    );
    assert_eq!(writer.current_position(), (1.0, 0.0, 0.0));
    assert_eq!(writer.current_feedrate(), 7200.0);
    assert_eq!(writer.current_e(), -0.4);
}

#[test]
fn zero_extrude_to_xy_with_feedrate_omits_e_but_keeps_feedrate() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.extrude_to_xy_with_feedrate_and_comment(
            Point2::new(0.5, 0.0),
            0.0,
            7200.0,
            Some("wipe and retract"),
        ),
        "G1 X0.5 Y0 F7200 ; wipe and retract\n"
    );
    assert_eq!(writer.current_position(), (0.5, 0.0, 0.0));
    assert_eq!(writer.current_feedrate(), 7200.0);
    assert_eq!(writer.current_e(), 0.0);
}

#[test]
fn absolute_extrude_to_xy_with_feedrate_emits_cumulative_e() {
    let mut writer = GCodeWriter::new();
    writer.set_extrusion_axis_mode(ExtrusionAxisMode::Absolute);

    assert_eq!(
        writer.extrude_to_xy_with_feedrate_and_comment(
            Point2::new(1.0, 0.0),
            0.25,
            7200.0,
            Some("wipe and retract"),
        ),
        "G1 X1 Y0 E0.25 F7200 ; wipe and retract\n"
    );
    assert_eq!(
        writer.extrude_to_xy_with_feedrate_and_comment(
            Point2::new(0.5, 0.0),
            -0.1,
            7200.0,
            Some("wipe and retract"),
        ),
        "G1 X0.5 Y0 E0.15 F7200 ; wipe and retract\n"
    );
    assert_eq!(writer.current_e(), 0.15);
}
