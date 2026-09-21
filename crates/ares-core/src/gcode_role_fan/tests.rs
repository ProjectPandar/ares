use super::*;
use crate::{SliceOptions, SpeedMoveKinematics};

#[test]
fn larger_upshift_while_pending_adds_to_remaining_kickstart_time() {
    let writer = GCodeWriter::new();
    let mut state = RoleFanGCodeState::new(None, 0.3, FanSpeedupControl::new(0.0, true));

    assert_eq!(state.layer_baseline_command(&writer, 20), "M106 S255\n");
    assert_eq!(state.after_move(&writer, &speed_move(0.0, 0.0, 10.0)), "");
    assert_eq!(state.after_move(&writer, &speed_move(0.1, 0.0, 10.0)), "");
    assert_eq!(state.layer_baseline_command(&writer, 90), "");
    assert_eq!(state.after_move(&writer, &speed_move(1.0, 0.0, 10.0)), "");
    assert_eq!(state.after_move(&writer, &speed_move(2.3, 0.0, 10.0)), "");
    assert_eq!(
        state.after_move(&writer, &speed_move(2.8, 0.0, 10.0)),
        "M106 S229\n"
    );
}

#[test]
fn initial_zero_baseline_is_suppressed_but_later_zero_turns_fan_off() {
    let writer = GCodeWriter::new();
    let mut state = RoleFanGCodeState::new(Some(0), 0.0, FanSpeedupControl::new(0.0, true));

    assert_eq!(state.layer_baseline_command(&writer, 0), "");
    assert_eq!(state.layer_baseline_command(&writer, 40), "M106 S102\n");
    assert_eq!(state.layer_baseline_command(&writer, 0), "M106 S0\n");
}

fn speed_move(x: f64, y: f64, speed_mm_s: f64) -> SpeedMove {
    SpeedMove::new(
        ToolpathMoveKind::Print,
        PrintPathRole::ExternalPerimeter,
        Point2::new(x, y),
        Some(0.0),
        SpeedMoveKinematics::new(speed_mm_s, None, None),
    )
}

#[test]
fn fan_speedup_marks_overhang_role_upshift_for_early_placement() {
    let writer = GCodeWriter::new();
    let options: SliceOptions = serde_json::from_value(serde_json::json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }))
    .unwrap();
    let role_fan_control = options.role_fan_control().unwrap().for_layer(
        options.part_cooling_fan_ramp().unwrap(),
        0,
        None,
    );
    let mut state = RoleFanGCodeState::new(
        Some(0),
        0.0,
        crate::options::FanSpeedupControl::new(0.2, true),
    );

    let command = state.before_move(RoleFanMoveCommand {
        writer: &writer,
        role_fan_control,
        baseline_speed: None,
        move_kind: ToolpathMoveKind::Print,
        role: PrintPathRole::Bridge,
    });

    assert_eq!(command.gcode, "M106 S191\n");
    assert!(command.speedup_eligible);
}

#[test]
fn fan_speedup_overhang_gate_does_not_mark_external_perimeter_override() {
    let writer = GCodeWriter::new();
    let options: SliceOptions = serde_json::from_value(serde_json::json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "overhang_fan_threshold": "0%"
    }))
    .unwrap();
    let role_fan_control = options.role_fan_control().unwrap().for_layer(
        options.part_cooling_fan_ramp().unwrap(),
        0,
        None,
    );
    let mut state = RoleFanGCodeState::new(
        Some(0),
        0.0,
        crate::options::FanSpeedupControl::new(0.2, true),
    );

    let command = state.before_move(RoleFanMoveCommand {
        writer: &writer,
        role_fan_control,
        baseline_speed: None,
        move_kind: ToolpathMoveKind::Print,
        role: PrintPathRole::ExternalPerimeter,
    });

    assert_eq!(command.gcode, "M106 S191\n");
    assert!(!command.speedup_eligible);
}
