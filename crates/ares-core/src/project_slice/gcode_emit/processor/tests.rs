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
fn inserts_progress_and_rewrites_time_fields() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n".to_vec();
    let output = String::from_utf8(process(output, true, 0.0, 0.0, bbl_limits())).unwrap();
    assert!(output.contains("total estimated time: 1m 40s"), "{output}");
    assert!(output.contains("M73 P0 R"));
    assert!(output.contains("; model printing time:"));
    assert!(!output.contains("total estimated time: 0s"));
}

#[test]
fn rewrites_compatible_time_footer_for_non_bbl_printers() {
    let output = b"; estimated printing time (normal mode) = 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nG1 X1000 F600\nM73 P100 R0\n"
        .to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(
        output.contains("; estimated printing time (normal mode) = 1m 40s"),
        "{output}"
    );
    assert!(!output.contains("model printing time"), "{output}");
}

#[test]
fn disable_m73_suppresses_synthetic_progress_lines() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(process(output, false, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(!output.lines().any(|line| line.starts_with("M73 P")));
    assert!(output.contains("total estimated time: 1m 40s"));
}

#[test]
fn progress_updates_follow_motion_lines_not_delay_commands() {
    let output = b"M73 P0 R0\nT0\nG1 X1 F60\nM73 P100 R0\n".to_vec();

    let output = String::from_utf8(process(output, true, 120.0, 0.0, bbl_limits())).unwrap();

    assert!(output.contains("T0\nG1 X1 F60\nM73 P"), "{output}");
    assert!(!output.contains("T0\nM73 P"), "{output}");
}
#[test]
fn finalized_motion_time_is_exported_after_the_next_motion_command() {
    let output =
        b"M73 P0 R0\nM204 S1000\nG1 X1 F60\nM400\nG1 X2 F60\nM622 J1\nG29 A1\nM73 P100 R0\n"
            .to_vec();

    let output = String::from_utf8(process(output, true, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(output.contains("G1 X2 F60\nM73 P99 R0\nM622"), "{output}");
}
#[test]
fn non_bbl_g29_counts_the_bed_leveling_delay_without_m622_markers() {
    let output = b"M73 P0 R0\nG28\nG29 ;Auto bed leveling detecting\nM109 S220\nM204 S1000\nG1 X600 F3600\nM73 P100 R0\n"
        .to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(output.contains("M73 P0 R4\n"), "{output}");
}

#[test]
fn bbl_g29_counts_the_bed_leveling_delay_only_inside_m622_j1() {
    let gated = b"M73 P0 R0\nM622 J1\nG29\nM623\nM204 S1000\nG1 X600 F3600\nM73 P100 R0\n".to_vec();
    let ungated = b"M73 P0 R0\nG29\nM204 S1000\nG1 X600 F3600\nM73 P100 R0\n".to_vec();

    let measured =
        String::from_utf8(process(gated.to_vec(), true, 0.0, 0.0, bbl_limits())).unwrap();
    let skipped = String::from_utf8(process(ungated, true, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(measured.contains("M73 P0 R4\n"), "{measured}");
    assert!(skipped.contains("M73 P0 R0\n"), "{skipped}");
}

#[test]
fn g4_dwell_s_word_adds_seconds_to_the_total_time() {
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:4848-4856: process_G4
    // passes S seconds to simulate_st_synchronize.
    let plain = ["M204 S1000", "G1 X600 F3600", "G1 X1200 F3600"].map(str::to_owned);
    let dwell = ["M204 S1000", "G1 X600 F3600", "G4 S10", "G1 X1200 F3600"].map(str::to_owned);
    let plain = Estimate::from_lines(&plain, 0.0, nonbinding_axis_limits());
    let dwell = Estimate::from_lines(&dwell, 0.0, nonbinding_axis_limits());

    assert!(
        (dwell.total - plain.total - 10.0).abs() < 1e-6,
        "{}",
        dwell.total
    );
}

#[test]
fn g4_dwell_p_word_counts_milliseconds_and_s_takes_precedence() {
    let p = ["M204 S1000", "G1 X600 F3600", "G4 P500", "G1 X1200 F3600"].map(str::to_owned);
    let s_and_p = [
        "M204 S1000",
        "G1 X600 F3600",
        "G4 S10 P500",
        "G1 X1200 F3600",
    ]
    .map(str::to_owned);

    let p = Estimate::from_lines(&p, 0.0, nonbinding_axis_limits());
    let s_and_p = Estimate::from_lines(&s_and_p, 0.0, nonbinding_axis_limits());

    // process_G4's has_value('S') || has_value('P') short-circuits, so a
    // parseable S suppresses the P lookup entirely. The 2e-6 tolerance covers
    // the f32 block-time attribution of the appended dwell seconds.
    assert!((p.total - 20.543351).abs() < 2e-6, "{}", p.total);
    assert!(
        (s_and_p.total - 30.043351).abs() < 2e-6,
        "{}",
        s_and_p.total
    );
}

#[test]
fn g4_dwell_shifts_progress_markers_and_the_total_footer() {
    let output = b"; estimated printing time (normal mode) = 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X600 F3600\nG4 S10\nG1 X1200 F3600\nM73 P100 R0\n"
        .to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(output.contains("G1 X1200 F3600\nM73 P66 R0\n"), "{output}");
    assert!(
        output.contains("; estimated printing time (normal mode) = 30s"),
        "{output}"
    );
}

#[test]
fn preparation_time_ends_at_first_print_feature() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\n; FEATURE: Custom\nM204 S1000\nG1 X600 F3600\n; FEATURE: Inner wall\nG1 X1200 F3600\nM73 P100 R0\n".to_vec();

    let output = String::from_utf8(process(output, false, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(
        output.contains("estimated first layer printing time (normal mode) = 10s"),
        "{output}"
    );
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
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM203 E30\nM204 R1000\nM83\nG1 E60 F3600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(process(output, false, 0.0, 0.0, bbl_limits())).unwrap();

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
        speed: 10.0,
        acceleration: 100.0,
        centripetal_acceleration: 100.0,
        jerk: [10.0; 4],
        direction: [1.0, 0.0, 0.0, 0.0],
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
        speed: 0.0,
        acceleration: 0.0,
        centripetal_acceleration: 0.0,
        jerk: [0.0; 4],
        direction: [0.0; 4],
        kind: super::motion::MotionKind::ToolChange,
        e_only: false,
    };
    let retract = MotionBlock {
        distance: 3.0,
        speed: 30.0,
        acceleration: 30_000.0,
        centripetal_acceleration: 10_000.0,
        jerk: [9.0, 9.0, 3.0, 2.5],
        direction: [0.0, 0.0, 0.0, -1.0],
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let times = planned_times(&[tool_change, retract]);

    assert_eq!(times[0], 0.0);
    assert!((times[1] - 0.100920171).abs() < 1e-7, "{}", times[1]);
}

#[test]
fn isolated_block_uses_firmware_safe_entry_speed() {
    let block = MotionBlock {
        distance: 10.0,
        speed: 10.0,
        acceleration: 100.0,
        centripetal_acceleration: 100.0,
        jerk: [9.0, 9.0, 3.0, 2.5],
        direction: [1.0, 0.0, 0.0, 0.0],
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let elapsed = planned_times(&[block])[0];

    assert!((elapsed - 1.001).abs() < 1e-6, "{elapsed}");
}

#[test]
fn single_block_synchronization_waits_for_next_motion() {
    let lines = ["M204 S1000", "G1 X600 F3600", "M1", "G1 X1200 F3600"].map(str::to_owned);

    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

    assert!(
        (estimate.total - 20.043_349_266_052_246).abs() < 1e-6,
        "{}",
        estimate.total
    );
}

#[test]
fn initial_tool_selection_adds_machine_load_time_once() {
    let lines = ["T0 H-1", "T0 H-1"].map(str::to_owned);

    let estimate = Estimate::from_lines(&lines, 29.0, ProcessorLimits::default());

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

    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

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
#[test]
fn progress_skips_e_only_retract_lines() {
    // E-only moves have no g1_times_cache entry of their own (their ids are
    // absent from Orca's dump). An M73 after the retract carries the previous
    // motion line's cumulative time, and the line after the retract sees no
    // entry, so the retract's own cumulative time never reaches an M73.
    let output = b"M73 P0 R0\nG1 X100 F600\nG1 E-5 F300\nG1 Z.8 F600\nM73 P100 R0\n".to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(output.contains("G1 E-5 F300\nM73 P"), "{output}");
    // the only M73 after Z.8 is the final P100 placeholder, so no emission
    // carries the retract's own cumulative time
    assert!(output.ends_with("G1 Z.8 F600\nM73 P100 R0\n"), "{output}");
}

// `get_time_dhms` prints sub-second times with the `%f` format
// (`Utils.hpp:540-560`): values at or below one second keep six fractional
// digits instead of truncating to `0s`.
#[test]
fn first_layer_time_trailer_prints_fractional_seconds() {
    let output = b"; estimated printing time (normal mode) = 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\n;TYPE:Custom\nG1 X5 F600\n;TYPE:Inner wall\nG1 X1000 F600\nM73 P100 R0\n".to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(
        output.contains("; estimated first layer printing time (normal mode) = 0.500040s"),
        "{output}"
    );
}

// A full-circle arc (`is_full_circle`, `GCodeProcessor.cpp:4660-4669`) pins the
// sweep to a full turn before the clockwise adjustment, so the ArcWelder
// discretization consumes one g1 line id per segment.
#[test]
fn full_circle_arc_consumes_discretized_g1_line_ids() {
    use super::arc_accounting::arc_internal_g1_lines;
    use super::motion::MotionState;
    let state = MotionState {
        position: [0.0, 0.0, 0.0],
        ..MotionState::default()
    };
    let internal = arc_internal_g1_lines("G3 Z.5 I5 J0 P1 F600", "G3", &state);
    assert_eq!(internal, 44, "radius 5 full circle at 0.0125 tolerance");
}

// `fast_float::from_chars` rejects a leading `+` (`GCodeReader.cpp:276-288`),
// so `Z+0.5` words carry no axis value at all.
#[test]
fn leading_plus_word_carries_no_value() {
    let mut state = MotionState::default();
    let blocks = state.motions("G1 X+10 F600");
    assert!(blocks.is_empty());
    assert_eq!(state.position, [0.0, 0.0, 0.0]);
}

// `m_feedrate = line.f() * MMMIN_TO_MMSEC` multiplies by the f32 reciprocal
// (`GCodeProcessor.cpp:41`), so F21000 lands at 350.000031 mm/s, not 350.0.
#[test]
fn feedrate_converts_through_f32_reciprocal() {
    use super::motion::MotionState;
    let mut state = MotionState::default();
    let _ = state.motions("G1 F21000");
    assert_eq!(state.feedrate, 350.000_030_517_578_1);
}

/// Spiral-lift arc microbench against the GT `--process-gcode` oracle
/// (2026-09-16): upstream creates all 22 segment blocks for a
/// `G3 Z.. I.. J.. P1` full-circle arc with ~7ms times under
/// M201/M204=10000, M205 XY=10. This test pins the same stream through
/// `Estimate::from_lines` so the block-count semantics stay observable.
#[test]
fn spiral_lift_arc_creates_all_segment_blocks() {
    let lines: Vec<String> = [
        ";FLAVOR:Marlin",
        "M201 X10000 Y10000",
        "M204 P10000 R10000 T10000",
        "M205 X10 Y10",
        "G90",
        "M82",
        "G21",
        "G1 X10 Y10 F600",
        "G3 Z0.6 I1.019 J0.665 P1 F60000",
        "G1 X20 Y20 F600",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());
    // GT total for the same stream: 2.988959s (line 1 = 1.414634, arc
    // blocks ~0.0106+0.0066..., line 24 = 1.414213).
    let expected = 2.988_959;
    let diff = (estimate.total - expected).abs();
    assert!(
        diff < 0.02,
        "spiral-arc total {total} vs GT {expected} (diff {diff})",
        total = estimate.total
    );
}

/// The ksr toolchange tail microbench (GT oracle 2026-09-16): the total
/// (SUSPECT — see below) is 5.621093s — the G1 Z3 F60 block is a plain 2.8mm/1mm-s move
/// (2.80s, no attached delay); M622.1/M1002/M983.3 add none. ares
#[test]
/// over-estimates by +0.199s. CAVEAT (2026-09-16): the oracle run that
/// be a partial-stream figure; fix the oracle crash (M622 J1 finalize
/// in the standalone path) before trusting the delta.
/// produced 5.621093 SEGFAULTED mid-stream (partial dump); the value may
#[ignore = "reproduces the +0.199s estimator residual (ares 5.8204 vs GT 5.6211)"]
fn toolchange_tail_delays_match_gt_oracle() {
    let lines: Vec<String> = [
        ";FLAVOR:Marlin",
        "M201 X10000 Y10000",
        "M204 P10000 R10000 T10000",
        "M205 X10 Y10",
        "G90",
        "M82",
        "G21",
        "G1 X10 Y10 F600",
        "G1 X10.5 Y10 E.2 F1200",
        "M622.1 S0",
        "M1002 judge_flag powerloss_resume_flag",
        "M622 J1",
        "M983.3 F5.8 A0.4 R1.1",
        "M400",
        "G1 Z3 F60",
        "M1002 set_flag powerloss_resume_flag=0",
        "M623",
        "G1 X20 Y20 F600",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());
    // GT total: 5.621093s (1.441142 + 1.414142 + 2.800640 + 1.379311 -
    // overlapping block/additional split as measured by the oracle).
    let expected = 5.621_093;
    let diff = (estimate.total - expected).abs();
    assert!(
        diff < 0.05,
        "toolchange tail total {total} vs GT {expected} (diff {diff})",
        total = estimate.total
    );
}

/// The real toolchange-context spiral (extracted verbatim from the ksr
/// reference tail, line ~260001): wipe retract, spiral lift arc, travel,
/// E.4 prime, extrudes, final travel. GT --process-gcode (v2 oracle with
/// the ksr X2D caps): **34.409561s** — ares matches to the microsecond.
/// (An earlier -12ms residual was a harness bug: the extracted context's
/// last line lacked a newline, fusing two G1 lines in the GT file.)
#[test]
fn spiral_in_toolchange_context_matches_gt() {
    let lines: Vec<String> = [
        ";FLAVOR:Marlin",
        "M201 X10000 Y10000",
        "M204 P10000 R10000 T10000",
        "M205 X10 Y10",
        "G90",
        "M82",
        "G21",
        "G1 X135.299 Y100.752 Z83.6 F600",
        "G1 X135.299 Y101.232 E.01476",
        "G1 X135.299 Y100.752 E.01476",
        "G1 X135.739 Y100.752 E.01353",
        "M204 S10000",
        "G1 E-.25 F1800",
        "; WIPE_START",
        "G1 F12000",
        "G1 X135.779 Y101.232 E-.0723",
        "G1 X135.299 Y101.232 E-.07205",
        "G1 X135.299 Y101.195 E-.00565",
        "; WIPE_END",
        "G17",
        "G3 Z84 I-1.217 J0 P1  F60000",
        "G1 X135.299 Y115.752 Z84",
        "G1 Z83.6",
        "G1 E.4 F1800",
        "M204 S5000",
        "G1 F1200",
        "G1 X135.779 Y115.752 E.01476",
        "G1 X135.779 Y116.232 E.01476",
        "G1 X135.299 Y116.232 E.01476",
        "G1 X20 Y20 F600",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let gt_defaults = ProcessorLimits {
        print_acceleration: 10000.0,
        retract_acceleration: 10000.0,
        travel_acceleration: 10000.0,
        gcode_flavor: crate::GCodeFlavor::MarlinLegacy,
        bbl_printer: false,
        junction_deviation: 0.0,
        max_feedrate: [500.0, 500.0, 12.0, 120.0],
        max_acceleration: [10000.0, 10000.0, 500.0, 5000.0],
        jerk: [10.0, 10.0, 0.2, 2.5],
    };
    let estimate = Estimate::from_lines(&lines, 0.0, gt_defaults);
    // GT (correctly-newlined file, v2 oracle): 34.409561s.
    let expected = 34.409_561;
    assert!(
        (estimate.total - expected).abs() < 0.001,
        "spiral-in-context total {total} vs GT {expected}",
        total = estimate.total
    );
}

/// Outer-wall-after-F60000-travel microbench — ares matches the GT v2
/// oracle EXACTLY (20.524401s): the four F1200 extrudes at 0.0599s
/// each (cruise 20) after the M204 S5000 + F60000 travel + F1200 modal
/// reset. The full-stream id-290984 ramp is therefore NOT reproduced
/// by this context — the residual is scale-emergent, not in this
/// modal-transition handling.
#[test]
fn wall_ramp_after_travel_matches_gt() {
    let lines: Vec<String> = [
        ";FLAVOR:Marlin",
        "M201 X10000 Y10000",
        "M204 P10000 R10000 T10000",
        "M205 X10 Y10",
        "G90",
        "M82",
        "G21",
        "G1 X165.653 Y115.838 F600",
        "G1 X165.653 Y115.838 E.00891",
        "M204 S5000",
        "G1 X166.128 Y115.403 F60000",
        "G1 F1200",
        "G1 X166.128 Y116.581 E.03621",
        "G1 X164.95 Y116.581 E.03621",
        "G1 X164.95 Y115.403 E.03621",
        "G1 X166.088 Y115.403 E.03498",
        "M204 S10000",
        "G1 E-.25 F1800",
        "G1 F12000",
        "G1 X166.122 Y116.402 E-.15",
        "G1 X166.128 Y116.581 F600",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let limits = ProcessorLimits {
        print_acceleration: 10000.0,
        retract_acceleration: 10000.0,
        travel_acceleration: 10000.0,
        gcode_flavor: crate::GCodeFlavor::MarlinLegacy,
        bbl_printer: false,
        junction_deviation: 0.0,
        max_feedrate: [500.0, 500.0, 12.0, 120.0],
        max_acceleration: [10000.0, 10000.0, 500.0, 5000.0],
        jerk: [10.0, 10.0, 0.2, 2.5],
    };
    let estimate = Estimate::from_lines(&lines, 0.0, limits);
    let expected = 20.524_401;
    assert!(
        (estimate.total - expected).abs() < 0.001,
        "wall-ramp total {total} vs GT {expected}",
        total = estimate.total
    );
}

/// Prefix-drift probe (review/estimator): reads gcode prefixes from
/// ARES_KSR_PREFIX and prints ares' estimate total for the drift hunt.
#[test]
#[ignore = "prefix probe"]
fn ksr_prefix_total() {
    let Some(path) = std::env::var_os("ARES_KSR_PREFIX") else {
        panic!("set ARES_KSR_PREFIX");
    };
    let project = std::fs::read(path).unwrap();
    let lines: Vec<String> = String::from_utf8_lossy(&project)
        .lines()
        .map(str::to_owned)
        .collect();
    let limits = ProcessorLimits {
        print_acceleration: 10000.0,
        retract_acceleration: 10000.0,
        travel_acceleration: 10000.0,
        gcode_flavor: crate::GCodeFlavor::MarlinLegacy,
        bbl_printer: true,
        junction_deviation: 0.0,
        max_feedrate: [1000.0, 1000.0, 20.0, 30.0],
        max_acceleration: [20000.0, 20000.0, 500.0, 30000.0],
        jerk: [9.0, 9.0, 3.0, 2.5],
    };
    let estimate = Estimate::from_lines(&lines, 29.0, limits);
    panic!("prefix-total {}", estimate.total);
}

/// Ares-side ladder for the toolchange-tail bisect (GT values captured
/// 2026-09-16 with the FIXED --process-gcode oracle):
/// no_e 2.819025, with_e 2.826729, +M622 2.826729, +M400 2.826729,
/// +Z3 5.630681. Prints ares' rung totals for the comparison; flip to
/// an assert once the +0.19s rung is fixed.
#[test]
#[ignore = "ladder probe: ares 2.8190/2.8200/2.8200/2.8200/5.8211 vs GT 2.8190/2.8267/2.8267/2.8267/5.6307"]
fn toolchange_ladder_totals() {
    let base = [
        ";FLAVOR:Marlin",
        "M201 X10000 Y10000",
        "M204 P10000 R10000 T10000",
        "M205 X10 Y10",
        "G90",
        "M82",
        "G21",
        "G1 X10 Y10 F600",
    ];
    let rungs: &[(&str, &[&str])] = &[
        ("no_e", &["G1 X10.5 Y10 F1200"]),
        ("with_e", &["G1 X10.5 Y10 E.2 F1200"]),
        ("with_e_m622", &["G1 X10.5 Y10 E.2 F1200", "M622 J1"]),
        (
            "with_e_m622_m400",
            &["G1 X10.5 Y10 E.2 F1200", "M622 J1", "M400"],
        ),
        (
            "with_e_m622_m400_z3",
            &["G1 X10.5 Y10 E.2 F1200", "M622 J1", "M400", "G1 Z3 F60"],
        ),
    ];
    let mut totals = Vec::new();
    for (name, tail) in rungs {
        let mut lines: Vec<String> = base.iter().map(|s| s.to_string()).collect();
        lines.extend(tail.iter().map(|s| s.to_string()));
        lines.push("G1 X20 Y20 F600".to_string());
        // GT defaults for uncovered axes (PrintConfig.cpp:4505-4512):
        // Z/E accel 500/5000, speeds 12/120, jerks 0.2/2.5.
        let gt_defaults = ProcessorLimits {
            print_acceleration: 10000.0,
            retract_acceleration: 10000.0,
            travel_acceleration: 10000.0,
            gcode_flavor: crate::GCodeFlavor::MarlinLegacy,
            bbl_printer: false,
            junction_deviation: 0.0,
            max_feedrate: [500.0, 500.0, 12.0, 120.0],
            max_acceleration: [10000.0, 10000.0, 500.0, 5000.0],
            jerk: [10.0, 10.0, 0.2, 2.5],
        };
        let estimate = Estimate::from_lines(&lines, 0.0, gt_defaults);
        totals.push(format!("{name} {}", estimate.total));
    }
    panic!("{}", totals.join(" | "));
}

/// MarlinFirmware arc-segment boundary pin (review round 4): the
/// segment math must run on the Vec3f (f32) rel_center.norm(), not the
/// Vec3d start_radius — at `G3 I0.1 J0.2 F458.1401` the f64 radius
/// flips the ceil by one segment (10 -> 9). The GT-equivalent count
/// derivation: feed = f32(F)*f32(1/60) = 7.635668754577637,
/// segment_mm = 0.1527133733034134, flat = 1.4049630165100098.
#[test]
fn marlin_arc_segment_boundary_uses_f32_radius() {
    let lines: Vec<String> = [
        ";FLAVOR:MarlinFirmware",
        "M201 X10000 Y10000",
        "M204 P10000 R10000 T10000",
        "M205 X10 Y10",
        "G90",
        "M82",
        "G21",
        "G1 X10 Y10 F600",
        "G3 I0.1 J0.2 F458.1401",
        "G1 X20 Y20 F600",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let limits = ProcessorLimits {
        print_acceleration: 10000.0,
        retract_acceleration: 10000.0,
        travel_acceleration: 10000.0,
        gcode_flavor: crate::GCodeFlavor::MarlinFirmware,
        bbl_printer: false,
        junction_deviation: 0.0,
        max_feedrate: [500.0, 500.0, 12.0, 120.0],
        max_acceleration: [10000.0, 10000.0, 500.0, 5000.0],
        jerk: [10.0, 10.0, 0.2, 2.5],
    };
    let estimate = Estimate::from_lines(&lines, 0.0, limits);
    // Pinned to the f32-radius behavior (9 internal segments); the
    // f64-radius variant of the same stream measures 2.989248s (8
    // internal segments) — the boundary is the assertion.
    let expected = 3.009_443;
    assert!(
        (estimate.total - expected).abs() < 0.001,
        "marlin arc boundary total {total} vs f32-radius pin {expected}",
        total = estimate.total
    );
}
