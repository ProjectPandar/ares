use super::{MotionBlock, MotionState, planned_times, process};

#[test]
fn inserts_progress_and_rewrites_time_fields() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n".to_vec();
    let output = String::from_utf8(process(output, true)).unwrap();
    assert!(output.contains("total estimated time: 1m 40s"), "{output}");
    assert!(output.contains("M73 P0 R"));
    assert!(output.contains("; model printing time:"));
    assert!(!output.contains("total estimated time: 0s"));
}

#[test]
fn disable_m73_suppresses_synthetic_progress_lines() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(process(output, false)).unwrap();

    assert!(!output.lines().any(|line| line.starts_with("M73 P")));
    assert!(output.contains("total estimated time: 1m 40s"));
}
#[test]
fn first_layer_time_ends_at_first_change_layer() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X600 F3600\n; CHANGE_LAYER\nG1 X600 F3600\n; CHANGE_LAYER\nM73 P100 R0\n".to_vec();

    let output = String::from_utf8(process(output, false)).unwrap();

    assert!(
        output.contains("estimated first layer printing time (normal mode) = 10s"),
        "{output}"
    );
}

#[test]
fn collinear_cruise_time_is_not_zeroed_by_default_jerk() {
    let mut state = MotionState::default();
    state.motion("M204 S1000");
    let first = state.motion("G1 X600 F3600").unwrap();
    let second = state.motion("G1 X1200 F3600").unwrap();

    let times = planned_times(&[first, second]);

    assert!((times.iter().sum::<f64>() - 20.06).abs() < 0.01);
}

#[test]
fn tracks_relative_e_only_moves() {
    let mut state = MotionState::default();
    state.motion("M83");
    let block = state.motion("G1 E-.4 F1800").unwrap();
    assert!((block.distance - 0.4).abs() < 1e-9);
}

#[test]
fn legacy_m204_s_sets_travel_and_t_as_retract() {
    let mut state = MotionState::default();
    state.motion("M204 S500 T125");
    assert_eq!(state.acceleration, 500.0);
    assert_eq!(state.travel_acceleration, 500.0);
    assert_eq!(state.retract_acceleration, 125.0);
}

#[test]
fn machine_max_feedrate_limits_extrusion_time() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM203 E30\nM204 R1000\nM83\nG1 E60 F3600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(process(output, false)).unwrap();

    assert!(output.contains("total estimated time: 2s"), "{output}");
}

#[test]
fn machine_max_acceleration_limits_motion_block() {
    let mut state = MotionState::default();
    state.motion("M201 E100");
    state.motion("M204 R1000");
    state.motion("M83");

    let block = state.motion("G1 E10 F3600").unwrap();

    assert_eq!(block.acceleration, 100.0);
}

#[test]
fn collinear_blocks_keep_speed_at_the_shared_junction() {
    let block = || MotionBlock {
        index: 0,
        distance: 10.0,
        speed: 10.0,
        acceleration: 100.0,
        jerk: [10.0; 4],
        direction: [1.0, 0.0, 0.0, 0.0],
    };

    let elapsed = planned_times(&[block(), block()]).into_iter().sum::<f64>();

    assert!((elapsed - 2.1).abs() < 1e-9, "{elapsed}");
}
#[test]
fn spiral_arc_p_one_is_one_turn_at_same_endpoint() {
    let mut state = MotionState::default();
    let block = state.motion("G3 Z.6 I1 J0 P1 F600").unwrap();
    assert!((block.distance - (2.0 * std::f64::consts::PI).hypot(0.6)).abs() < 1e-9);
}

#[test]
fn homing_command_emits_motion_to_requested_axes() {
    let mut state = MotionState::default();
    state.motion("G1 X10 Y20 Z3 F600");

    let block = state.motion("G28 X").unwrap();

    assert!((block.distance - 10.0).abs() < 1e-9);
    assert_eq!(state.position, [0.0, 20.0, 3.0]);
}

#[test]
fn unsupported_commands_do_not_change_motion_feedrate() {
    let mut state = MotionState::default();
    state.motion("G1 X1 F600");
    state.motion("G130 F4.36536");

    let block = state.motion("G1 X2").unwrap();

    assert_eq!(block.speed, 10.0);
}
#[test]
fn arc_p_word_adds_full_turns() {
    let mut state = MotionState::default();
    let block = state.motion("G3 X0 Y2 I0 J1 P1 F600").unwrap();
    assert!((block.distance - 3.0 * std::f64::consts::PI).abs() < 1e-9);
}

#[test]
fn bare_g92_resets_all_logical_axes() {
    let mut state = MotionState {
        position: [10.0, 20.0, 30.0],
        e_position: 40.0,
        ..MotionState::default()
    };

    assert!(state.motion("G92").is_none());

    assert_eq!(state.position, [0.0; 3]);
    assert_eq!(state.e_position, 0.0);
}
