use super::motion::MotionState;

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
