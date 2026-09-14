use super::super::limiter::{RateSlope, adjust_volumetric_rate, role_index};
use super::super::line::{LineParser, X};
use crate::ExtrusionRole;

const SLOPE: f32 = 100.0 * 60.0 * 60.0;

fn slopes() -> [RateSlope; 20] {
    [RateSlope {
        positive: SLOPE,
        negative: SLOPE,
    }; 20]
}

/// Three extrusion lines in one segment: a fast middle line between two
/// slow ones. The forward pass must lower the middle line's entry rate
/// so it does not accelerate faster than the configured slope allows.
#[test]
fn forward_pass_limits_entry_rate() {
    // A deliberately tiny slope forces the entry-rate limit; the
    // production slope (100 mm/s^2 = 360000 mm3/min^2) would allow the
    // acceleration within one 10mm segment and modify nothing.
    let slopes = [RateSlope {
        positive: 1.0,
        negative: 1.0,
    }; 20];
    let mut parser = LineParser::new(&[1.75], false);
    let mut marker = Default::default();
    parser.parse_line(";_EXTRUSION_ROLE:1", &mut marker); // Perimeter
    let mut lines = vec![];
    for code in [
        "G1 X10 Y0 E1 F300 ;_EXTRUDE_SET_SPEED",
        "G1 X20 Y0 E2 F1800",
        "G1 X30 Y0 E3 F1800",
        "G1 X40 Y0 E4 F300",
    ] {
        let mut line = Default::default();
        parser.parse_line(code, &mut line).expect("parses");
        lines.push(line);
    }
    // Indexes 1..=3 are extrusions inside the set-speed block.
    for line in &lines[1..=3] {
        assert!(line.extruding());
        assert!(line.adjustable_flow);
    }

    adjust_volumetric_rate(&mut lines, &slopes, false, 0, 3);

    // The last fast line's entry rate must have been limited (and marked).
    assert!(lines[2].modified);
    assert!(lines[2].volumetric_extrusion_rate_start < lines[2].volumetric_extrusion_rate);
}

/// Bridge infill lines never have their flow adjusted
/// (`flow_not_adjustable`, `PressureEqualizer.cpp:765-770`).
#[test]
fn bridge_infill_flow_is_not_adjustable() {
    let mut parser = LineParser::new(&[1.75], false);
    let mut marker = Default::default();
    parser.parse_line(";_EXTRUSION_ROLE:9", &mut marker);

    let mut lines = vec![];
    for code in [
        "G1 X10 Y0 E1 F300 ;_EXTRUDE_SET_SPEED",
        "G1 X20 Y0 E2 F1800",
        "G1 X30 Y0 E3 F1800",
    ] {
        let mut line = Default::default();
        parser.parse_line(code, &mut line).expect("parses");
        lines.push(line);
    }
    assert_eq!(lines[1].extrusion_role, ExtrusionRole::BridgeInfill);

    adjust_volumetric_rate(&mut lines, &slopes(), false, 0, 2);

    assert!(!lines[1].modified);
    assert!(!lines[2].modified);
}

/// `extrusion_rate_smoothing_external_perimeter_only` skips every role
/// except overhang/external perimeters (`PressureEqualizer.cpp:768-770`).
#[test]
fn external_only_mode_skips_other_roles() {
    let mut parser = LineParser::new(&[1.75], false);
    let mut lines = vec![];
    for code in [
        "G1 X10 Y0 E1 F300 ;_EXTRUDE_SET_SPEED",
        "G1 X20 Y0 E2 F1800",
        "G1 X30 Y0 E3 F1800",
    ] {
        let mut line = Default::default();
        parser.parse_line(code, &mut line).expect("parses");
        lines.push(line);
    }
    assert_eq!(
        role_index(lines[1].extrusion_role),
        role_index(ExtrusionRole::None)
    );

    adjust_volumetric_rate(&mut lines, &slopes(), true, 0, 2);

    assert!(!lines[1].modified);
    assert!(!lines[2].modified);
}

/// A window with fewer than 2 lines is a no-op (`cpp:735-737`).
#[test]
fn short_window_is_noop() {
    let mut parser = LineParser::new(&[1.75], false);
    let mut lines = vec![];
    let mut line = Default::default();
    parser
        .parse_line("G1 X10 Y0 E1 F1800", &mut line)
        .expect("parses");
    lines.push(line);

    adjust_volumetric_rate(&mut lines, &slopes(), false, 0, 0);

    assert!(!lines[0].modified);
    let _ = X;
}
