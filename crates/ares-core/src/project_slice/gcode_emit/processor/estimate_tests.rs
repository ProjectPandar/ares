use super::estimate::Estimate;
use super::motion::{MotionBlock, MotionState, planned_times};
use super::{ProcessorLimits, process};

// Synthetic timing tests isolate other behavior with nonbinding axis caps;
// zero is a real acceleration limit, not an unlimited sentinel.
fn nonbinding_axis_limits() -> ProcessorLimits {
    ProcessorLimits {
        max_acceleration: [f64::MAX; 4],
        ..ProcessorLimits::default()
    }
}

// The synthetic footer fixtures use the BBL placeholder set; the end-to-end
// suite covers the compatible set via Orca parity.
fn bbl_limits() -> ProcessorLimits {
    ProcessorLimits {
        bbl_printer: true,
        ..nonbinding_axis_limits()
    }
}
#[test]
fn collinear_cruise_time_is_not_zeroed_by_default_jerk() {
    let mut state = MotionState {
        max_acceleration: [f64::MAX; 4],
        ..MotionState::default()
    };
    state.motion("M204 S1000");
    let first = state.motion("G1 X600 F3600").unwrap();
    let second = state.motion("G1 X1200 F3600").unwrap();

    let times = planned_times(&[first, second]);

    assert!((times.iter().sum::<f64>() - 20.043_349_266_052_246).abs() < 1e-6);
}

#[test]
fn tracks_relative_e_only_moves() {
    let mut state = MotionState::default();
    state.motion("M83");
    let block = state.motion("G1 E-.4 F1800").unwrap();
    assert!((block.distance - 0.4).abs() < 1e-6, "{}", block.distance);
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
    // Two blocks: upstream drops a lone trailing block from the total.
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM203 E30\nM204 R1000\nM83\nG1 E30 F3600\nG1 E60 F3600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(
        process(
            output,
            false,
            0.0,
            bbl_limits().with_settings(&Default::default()),
        )
        .0
        .gcode,
    )
    .unwrap();

    assert!(output.contains("total estimated time: 3s"), "{output}");
}

#[test]
fn machine_max_acceleration_limits_motion_block() {
    let mut state = MotionState::default();
    state.motion("M201 E100");
    state.motion("M204 R1000");
    state.motion("M83");

    let block = state.motion("G1 E10 F3600").unwrap();

    assert_eq!(block.max_acceleration[3], 100.0);
}

#[test]
fn m204_updates_respect_machine_acceleration_envelopes() {
    let mut state = MotionState::with_acceleration_limits(20_000.0, 30_000.0, 9_000.0);

    state.motion("M204 P20000 R30000 T20000");

    assert_eq!(state.acceleration, 20_000.0);
    assert_eq!(state.retract_acceleration, 30_000.0);
    assert_eq!(state.travel_acceleration, 9_000.0);
}

#[test]
fn travel_blocks_retain_print_acceleration_for_centripetal_limits() {
    let mut state = MotionState {
        max_acceleration: [f64::MAX; 4],
        ..MotionState::default()
    };
    state.motion("M204 P500 T10000");

    let block = state.motion("G1 X10 F6000").unwrap();

    assert_eq!(block.acceleration, 10_000.0);
    assert_eq!(block.centripetal_acceleration, 500.0);
}

#[test]
fn collinear_blocks_keep_speed_at_the_shared_junction() {
    let block = || MotionBlock {
        distance: 10.0,
        delta: [10.0, 0.0, 0.0, 0.0],
        speed: 10.0,
        acceleration: 100.0,
        centripetal_acceleration: 100.0,
        max_feedrate: [0.0; 4],
        max_acceleration: [f64::INFINITY; 4],
        jerk: [10.0; 4],
        extrude_factor: 1.0,
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let elapsed = planned_times(&[block(), block()]).into_iter().sum::<f64>();

    assert!((elapsed - 2.0).abs() < 1e-9, "{elapsed}");
}

#[test]
fn tool_change_block_resets_the_following_junction() {
    let tool_change = MotionBlock {
        distance: 0.0,
        delta: [0.0; 4],
        speed: 0.0,
        acceleration: 0.0,
        centripetal_acceleration: 0.0,
        max_feedrate: [0.0; 4],
        max_acceleration: [f64::INFINITY; 4],
        jerk: [0.0; 4],
        extrude_factor: 1.0,
        kind: super::motion::MotionKind::ToolChange,
        e_only: false,
    };
    let retract = MotionBlock {
        distance: 3.0,
        delta: [0.0, 0.0, 0.0, -3.0],
        speed: 30.0,
        acceleration: 30_000.0,
        centripetal_acceleration: 10_000.0,
        max_feedrate: [0.0; 4],
        max_acceleration: [f64::INFINITY; 4],
        jerk: [9.0, 9.0, 3.0, 2.5],
        extrude_factor: 1.0,
        kind: super::motion::MotionKind::Regular,
        e_only: true,
    };

    let times = planned_times(&[tool_change, retract]);

    assert_eq!(times[0], 0.0);
    assert!((times[1] - 0.100920171).abs() < 1e-7, "{}", times[1]);
}

#[test]
fn isolated_block_uses_firmware_safe_entry_speed() {
    let block = MotionBlock {
        distance: 10.0,
        delta: [10.0, 0.0, 0.0, 0.0],
        speed: 10.0,
        acceleration: 100.0,
        centripetal_acceleration: 100.0,
        max_feedrate: [0.0; 4],
        max_acceleration: [f64::INFINITY; 4],
        jerk: [9.0, 9.0, 3.0, 2.5],
        extrude_factor: 1.0,
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let elapsed = planned_times(&[block])[0];

    assert!((elapsed - 1.001).abs() < 1e-6, "{elapsed}");
}

#[test]
fn single_block_synchronization_waits_for_next_motion() {
    // Upstream `calculate_time` keeps a lone queued block deferred until a
    // later block arrives (`blocks.size() < 2` early return); the trailing
    // `G1 X1200` pairs with it, so both count toward the total.
    let lines = [
        "M204 S1000",
        "G1 X600 F3600",
        "M1",
        "G1 X1200 F3600",
        "G1 X1800 F3600",
    ]
    .map(str::to_owned);

    let estimate = Estimate::record(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
        None,
    );

    assert!(
        (estimate.total - 30.043_349_266_052_246).abs() < 1e-6,
        "{}",
        estimate.total
    );
}

#[test]
fn final_pass_drains_a_lone_block_with_buffered_delay() {
    // `calculate_time(is_final)` (`GCodeProcessor.cpp:427`) processes a
    // lone queued block when extra time is still buffered: the `G4 S5`
    // found fewer than two blocks, so the block and the delay both count.
    let lines = ["M204 S1000", "G1 X600 F3600", "G4 S5"].map(str::to_owned);

    let estimate = Estimate::record(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
        None,
    );

    assert!(estimate.total > 15.0, "{}", estimate.total);
}

#[test]
fn initial_tool_selection_adds_machine_load_time_once() {
    let lines = ["T0 H-1", "T0 H-1"].map(str::to_owned);

    let estimate = Estimate::record(
        &lines,
        29.0,
        &ProcessorLimits::default().with_settings(&Default::default()),
        None,
    );

    assert_eq!(estimate.total, 29.0);
}
#[test]
fn spiral_arc_p_one_is_one_turn_at_same_endpoint() {
    let mut state = MotionState::default();
    let block = state.motion("G3 Z.6 I1 J0 P1 F600").unwrap();
    assert!(
        (block.distance - (2.0 * std::f64::consts::PI).hypot(0.6)).abs() < 1e-6,
        "{}",
        block.distance
    );
}

#[test]
fn marlin_arc_is_discretized_into_firmware_segments() {
    let mut state = MotionState::default();

    let blocks = state.motions("G3 X0 Y2 I0 J1 F600");

    assert_eq!(blocks.len(), 10);
    let distance = blocks.iter().map(|block| block.distance).sum::<f64>();
    assert!((distance - 3.123189).abs() < 1e-6, "{distance}");
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
fn homing_motion_contributes_to_total_estimate() {
    let lines = ["G1 X10 F600", "G28 X"].map(str::to_owned);

    let estimate = Estimate::record(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
        None,
    );

    assert!(
        (estimate.total - 2.000_159_978_866_577).abs() < 1e-9,
        "{}",
        estimate.total
    );
}

#[test]
fn unsupported_commands_do_not_change_motion_feedrate() {
    let mut state = MotionState::default();
    state.motion("G1 X1 F600");
    state.motion("G130 F4.36536");

    let block = state.motion("G1 X2").unwrap();

    // F600 through the f32 reciprocal lands at 10.000001 mm/s
    // (`GCodeProcessor.cpp:41`).
    assert_eq!(block.speed, 10.000_000_953_674_316);
}
#[test]
fn arc_p_word_adds_full_turns() {
    let mut state = MotionState::default();
    let block = state.motion("G3 X0 Y2 I0 J1 P1 F600").unwrap();
    assert!((block.distance - 3.0 * std::f64::consts::PI).abs() < 1e-9);
}

#[test]
fn bare_g92_sets_origin_without_moving_physical_axes() {
    let mut state = MotionState {
        position: [10.0, 20.0, 30.0],
        e_position: 40.0,
        ..MotionState::default()
    };

    assert!(state.motion("G92").is_none());

    assert_eq!(state.position, [10.0, 20.0, 30.0]);
    assert_eq!(state.e_position, 40.0);
    assert_eq!(state.origin, [10.0, 20.0, 30.0, 40.0]);
    assert_eq!(
        std::array::from_fn::<_, 3, _>(|axis| state.position[axis] - state.origin[axis]),
        [0.0; 3]
    );
    assert_eq!(state.e_position - state.origin[3], 0.0);
}
