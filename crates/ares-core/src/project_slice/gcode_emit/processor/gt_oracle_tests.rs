use super::ProcessorLimits;
use super::estimate::Estimate;

// Synthetic timing tests isolate other behavior with nonbinding axis caps;
// zero is a real acceleration limit, not an unlimited sentinel.
fn nonbinding_axis_limits() -> ProcessorLimits {
    ProcessorLimits {
        max_acceleration: [f64::MAX; 4],
        ..ProcessorLimits::default()
    }
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
    let estimate = Estimate::record(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
        None,
    );
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
        retraction_length: 0.0,
        retract_restart_extra: 0.0,
        retraction_feedrate: 0.0,
        deretraction_feedrate: 0.0,
        ..ProcessorLimits::default()
    };
    let estimate = Estimate::record(
        &lines,
        0.0,
        &gt_defaults.with_settings(&Default::default()),
        None,
    );
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
        retraction_length: 0.0,
        retract_restart_extra: 0.0,
        retraction_feedrate: 0.0,
        deretraction_feedrate: 0.0,
        ..ProcessorLimits::default()
    };
    let estimate = Estimate::record(
        &lines,
        0.0,
        &limits.with_settings(&Default::default()),
        None,
    );
    let expected = 20.524_401;
    assert!(
        (estimate.total - expected).abs() < 0.001,
        "wall-ramp total {total} vs GT {expected}",
        total = estimate.total
    );
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
        retraction_length: 0.0,
        retract_restart_extra: 0.0,
        retraction_feedrate: 0.0,
        deretraction_feedrate: 0.0,
        ..ProcessorLimits::default()
    };
    let estimate = Estimate::record(
        &lines,
        0.0,
        &limits.with_settings(&Default::default()),
        None,
    );
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
