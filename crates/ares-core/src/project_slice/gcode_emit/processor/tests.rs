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
