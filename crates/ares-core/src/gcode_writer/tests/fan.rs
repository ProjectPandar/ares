use super::super::*;

#[test]
fn set_fan_formats_marlin_percent_as_pwm() {
    let writer = GCodeWriter::new();

    assert_eq!(writer.set_fan(0), "M106 S0\n");
    assert_eq!(writer.set_fan(20), "M106 S51\n");
    assert_eq!(writer.set_fan(100), "M106 S255\n");
}

#[test]
fn set_fan_formats_mach3_and_machinekit_with_p_axis() {
    for flavor in [GCodeFlavor::Mach3, GCodeFlavor::Machinekit] {
        let mut writer = GCodeWriter::new();
        writer.set_gcode_flavor(flavor);

        assert_eq!(writer.set_fan(50), "M106 P127\n", "{flavor:?}");
        assert_eq!(writer.set_fan(0), "M106 S0\n", "{flavor:?}");
    }
}

#[test]
fn set_fan_formats_makerware_and_sailfish_commands() {
    for flavor in [GCodeFlavor::MakerWare, GCodeFlavor::Sailfish] {
        let mut writer = GCodeWriter::new();
        writer.set_gcode_flavor(flavor);

        assert_eq!(writer.set_fan(50), "M126\n", "{flavor:?}");
        assert_eq!(writer.set_fan(0), "M127\n", "{flavor:?}");
    }
}

#[test]
fn set_fan_preserves_zero_when_min_pwm_is_non_zero() {
    let mut writer = GCodeWriter::new();
    writer.set_part_cooling_fan_min_pwm(30);

    assert_eq!(writer.set_fan(0), "M106 S0\n");
}

#[test]
fn set_fan_clamps_non_zero_speed_below_min_pwm_before_pwm_conversion() {
    let mut writer = GCodeWriter::new();
    writer.set_part_cooling_fan_min_pwm(30);

    assert_eq!(writer.set_fan(20), "M106 S76\n");
    assert_eq!(writer.set_fan(30), "M106 S76\n");
    assert_eq!(writer.set_fan(60), "M106 S153\n");
}

#[test]
fn set_fan_clamps_mach3_pwm_but_preserves_off_command() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::Mach3);
    writer.set_part_cooling_fan_min_pwm(30);

    assert_eq!(writer.set_fan(20), "M106 P76\n");
    assert_eq!(writer.set_fan(0), "M106 S0\n");
}

#[test]
fn set_fan_makerware_and_sailfish_preserve_command_family_with_min_pwm() {
    for flavor in [GCodeFlavor::MakerWare, GCodeFlavor::Sailfish] {
        let mut writer = GCodeWriter::new();
        writer.set_gcode_flavor(flavor);
        writer.set_part_cooling_fan_min_pwm(30);

        assert_eq!(writer.set_fan(20), "M126\n", "{flavor:?}");
        assert_eq!(writer.set_fan(0), "M127\n", "{flavor:?}");
    }
}
