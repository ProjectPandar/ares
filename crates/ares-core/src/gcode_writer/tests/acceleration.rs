use super::*;

#[test]
fn set_acceleration_formats_m204_and_updates_state() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(333.6), None),
        "M204 S334\n"
    );

    assert_eq!(writer.current_acceleration(), 334);
}

#[test]
fn set_acceleration_suppresses_none_zero_and_unchanged_values() {
    let mut writer = GCodeWriter::new();

    assert_eq!(writer.set_print_acceleration_with_comment(None, None), "");
    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(0.0), None),
        ""
    );
    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.0), None),
        "M204 S500\n"
    );
    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.4), None),
        ""
    );
}

#[test]
fn set_acceleration_appends_comment() {
    let mut writer = GCodeWriter::new();

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.0), Some("adjust acceleration")),
        "M204 S500 ; adjust acceleration\n"
    );
}

#[test]
fn klipper_acceleration_emits_velocity_limit_with_default_accel_to_decel() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Klipper);

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(333.6), None),
        "SET_VELOCITY_LIMIT ACCEL=334 ACCEL_TO_DECEL=167\n"
    );

    assert_eq!(writer.current_acceleration(), 334);
}

#[test]
fn klipper_acceleration_uses_integer_truncating_custom_accel_to_decel_factor() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Klipper);
    writer.set_accel_to_decel_config(crate::options::AccelToDecelConfig::new(true, 33.0));

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(333.6), None),
        "SET_VELOCITY_LIMIT ACCEL=334 ACCEL_TO_DECEL=110\n"
    );
}

#[test]
fn klipper_acceleration_omits_accel_to_decel_when_disabled() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Klipper);
    writer.set_accel_to_decel_config(crate::options::AccelToDecelConfig::new(false, 50.0));

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.0), None),
        "SET_VELOCITY_LIMIT ACCEL=500\n"
    );
}

#[test]
fn klipper_acceleration_truncates_after_applying_decimal_factor() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Klipper);
    writer.set_accel_to_decel_config(crate::options::AccelToDecelConfig::new(true, 33.5));

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(333.6), None),
        "SET_VELOCITY_LIMIT ACCEL=334 ACCEL_TO_DECEL=111\n"
    );
}

#[test]
fn klipper_acceleration_comment_applies_to_whole_velocity_limit_command() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Klipper);

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.0), Some("adjust acceleration")),
        "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 ; adjust acceleration\n"
    );
}

#[test]
fn marlin2_separates_print_and_travel_acceleration_commands() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MarlinFirmware);

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.0), None),
        "M204 P500\n"
    );
    assert_eq!(
        writer.set_travel_acceleration_with_comment(Some(900.0), None),
        "M204 T900\n"
    );
    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(500.0), None),
        ""
    );
    assert_eq!(
        writer.set_travel_acceleration_with_comment(Some(900.0), None),
        ""
    );
}

#[test]
fn reprap_firmware_uses_marlin2_style_separate_acceleration_commands() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::RepRapFirmware);

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(501.0), None),
        "M204 P501\n"
    );
    assert_eq!(
        writer.set_travel_acceleration_with_comment(Some(901.0), None),
        "M204 T901\n"
    );
}

#[test]
fn repetier_separates_print_and_travel_acceleration_commands() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Repetier);

    assert_eq!(
        writer.set_print_acceleration_with_comment(Some(502.0), None),
        "M201 X502 Y502\n"
    );
    assert_eq!(
        writer.set_travel_acceleration_with_comment(Some(902.0), None),
        "M202 X902 Y902\n"
    );
}

#[test]
fn separate_travel_acceleration_comments_apply_to_emitted_command() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MarlinFirmware);

    assert_eq!(
        writer.set_travel_acceleration_with_comment(Some(900.0), Some("adjust acceleration")),
        "M204 T900 ; adjust acceleration\n"
    );
}
