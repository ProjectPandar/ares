use super::*;

#[test]
fn pressure_reset_preserves_selected_reprap_tool() {
    for flavor in [GCodeFlavor::RepRapSprinter, GCodeFlavor::RepRapFirmware] {
        let mut writer = Writer::new(0.2, 0.5, flavor, 2.0);
        writer.set_tool(2, 0.5);
        writer.disable_linear_advance();
        assert!(writer.gcode.ends_with("M572 D2 S0\n"));
        writer.set_tool(1, 0.5);
        writer.disable_linear_advance();
        assert!(writer.gcode.ends_with("M572 D2 S0\nM572 D1 S0\n"));
    }
}

#[test]
fn source_writer_limits_selected_tool_and_keeps_preview_area_owner() {
    let mut writer = Writer::new(0.2, 0.5, GCodeFlavor::MarlinLegacy, 2.0);
    writer.set_initial_position([0.0, 0.0], [0.0; 2], 0.0);
    writer.set_tool(2, 0.5);
    writer.set_flow(0.1);
    writer.extrude([10.0, 0.0], 600.0);
    assert!(writer.gcode.ends_with("G1 X10.000 E1.0000 F300\n"));
    assert_eq!(writer.used_filament, 1.0);
    assert_eq!(writer.extrusions.len(), 2);
    assert_eq!(
        writer.extrusions[1].width,
        1.0 + 0.2 * (1.0 - std::f64::consts::FRAC_PI_4) as f32
    );
    writer.suppress_preview = true;
    writer.load_move_x(20.0, 4.0, 2.0, 50.0);
    assert_eq!(writer.extrusions.len(), 2);
    assert_eq!(writer.take_used_filament(), 1.0);
    assert_eq!(writer.take_used_filament(), 0.0);
}

#[test]
fn source_writer_rotation_and_y_shift_preserve_local_position() {
    let mut writer = Writer::new(0.2, 0.5, GCodeFlavor::Klipper, 2.0);
    writer.set_initial_position([0.0, 0.0], [40.0, 10.0], 180.0);
    writer.set_y_shift(3.0);
    assert_eq!(writer.position, [0.0, -3.0]);
    let position = writer.rotated(writer.position);
    assert!((position[0] - 40.0).abs() < 1e-4);
    assert!((position[1] - 10.0).abs() < 1e-4);
    writer.disable_linear_advance();
    assert!(writer.gcode.ends_with("SET_PRESSURE_ADVANCE ADVANCE=0\n"));
    let length = writer.gcode.len();
    writer.speed_override_backup();
    writer.speed_override_restore();
    writer.trimpot(750);
    assert_eq!(writer.gcode.len(), length);
}
