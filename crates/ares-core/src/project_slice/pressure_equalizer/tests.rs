use super::line::{E, X};
use super::*;
use crate::ExtrusionRole;

fn parser() -> LineParser {
    LineParser::new(&[1.75], false)
}

#[test]
fn role_marker_updates_role_and_is_filtered() {
    let mut parser = parser();
    let mut line = GCodeLine::default();

    let kept = parser.parse_line(";_EXTRUSION_ROLE:2", &mut line);

    assert!(kept.is_none());
    assert_eq!(parser.current_role, ExtrusionRole::ExternalPerimeter);
}

#[test]
fn extrude_line_classified_with_volumetric_rate() {
    let mut parser = parser();
    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X10 Y0 F1200", &mut line)
        .expect("travel parses");
    assert_eq!(line.line_type, GCodeLineType::Move);

    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X10 Y10 E1 F1200", &mut line)
        .expect("extrusion parses");
    assert_eq!(line.line_type, GCodeLineType::Extrude);
    assert!(line.extruding());
    // 1.75²π/4 ≈ 2.405 mm²; 1200 mm/min × 1mm E / 10mm XY.
    let expected = 2.405_281_8_f32 * 1200.0 * (1.0_f32 / 10.0);
    assert!((line.volumetric_extrusion_rate - expected).abs() < 1.0);
    assert_eq!(line.feedrate(), 1200.0);
}

#[test]
fn retract_and_unretract_classification() {
    let mut parser = parser();
    let mut line = GCodeLine::default();
    parser.parse_line("G1 E5 F1800", &mut line).expect("parses");
    assert_eq!(line.line_type, GCodeLineType::Unretract);

    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 E-1 F1800", &mut line)
        .expect("parses");
    assert_eq!(line.line_type, GCodeLineType::Retract);
    assert!(parser.retracted);
}

#[test]
fn g92_resets_logical_position() {
    let mut parser = parser();
    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X10 Y0 E1 F600", &mut line)
        .expect("parses");

    let mut line = GCodeLine::default();
    parser.parse_line("G92 E0", &mut line).expect("parses");
    assert_eq!(parser.current_pos[E], 0.0);
    assert_eq!(parser.current_pos[X], 10.0);

    // The next extrusion computes against the reset origin.
    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X20 Y0 E1 F600", &mut line)
        .expect("parses");
    assert_eq!(line.pos_start[E], 0.0);
    assert_eq!(line.pos_end[E], 1.0);
}

#[test]
fn set_speed_markers_open_and_close_the_block() {
    let mut parser = parser();
    let mut line = GCodeLine::default();

    parser
        .parse_line("G1 F1800 ;_EXTRUDE_SET_SPEED", &mut line)
        .expect("parses");
    assert!(line.extrude_set_speed_tag);
    assert!(parser.opened_extrude_set_speed_block);
    // The marker line itself parses with the block already open.
    assert!(line.adjustable_flow);

    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X5 Y0 E1 ;_EXTRUDE_END", &mut line)
        .expect("parses");
    // The end marker closes the block before the G1 body reads it.
    assert!(!line.adjustable_flow);
    assert!(line.extrude_end_tag);
    assert!(!parser.opened_extrude_set_speed_block);
}

#[test]
fn relative_e_distances_accumulate() {
    let mut parser = LineParser::new(&[1.75], true);
    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X10 Y0 E1 F600", &mut line)
        .expect("parses");
    assert_eq!(line.pos_end[E], 1.0);

    let mut line = GCodeLine::default();
    parser
        .parse_line("G1 X20 Y0 E1 F600", &mut line)
        .expect("parses");
    assert_eq!(line.pos_start[E], 1.0);
    assert_eq!(line.pos_end[E], 2.0);
}

#[test]
fn tool_change_switches_extruder_and_retracts() {
    let mut parser = parser();
    let mut line = GCodeLine::default();
    parser.parse_line("T1", &mut line).expect("parses");
    assert_eq!(line.line_type, GCodeLineType::ToolChange);
    assert_eq!(parser.current_extruder, 1);
    assert!(parser.retracted);

    let mut line = GCodeLine::default();
    parser.parse_line("T1", &mut line).expect("parses");
    assert_eq!(line.line_type, GCodeLineType::Noop);
}

#[test]
fn firmware_retract_codes_map() {
    let mut parser = parser();
    let mut line = GCodeLine::default();
    parser.parse_line("G10", &mut line).expect("parses");
    assert_eq!(line.line_type, GCodeLineType::Retract);

    let mut line = GCodeLine::default();
    parser.parse_line("G11", &mut line).expect("parses");
    assert_eq!(line.line_type, GCodeLineType::Unretract);
}
