use super::{MotionState, process};

#[test]
fn inserts_progress_and_rewrites_time_fields() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n".to_vec();
    let output = String::from_utf8(process(output)).unwrap();
    assert!(output.contains("total estimated time: 1m 40s"), "{output}");
    assert!(output.contains("M73 P0 R"));
    assert!(output.contains("; model printing time:"));
    assert!(!output.contains("total estimated time: 0s"));
}

#[test]
fn tracks_relative_e_only_moves() {
    let mut state = MotionState::default();
    state.motion("M83");
    let block = state.motion("G1 E-.4 F1800").unwrap();
    assert!((block.distance - 0.4).abs() < 1e-9);
}

#[test]
fn spiral_arc_p_one_is_one_turn_at_same_endpoint() {
    let mut state = MotionState::default();
    let block = state.motion("G3 Z.6 I1 J0 P1 F600").unwrap();
    assert!((block.distance - (2.0 * std::f64::consts::PI).hypot(0.6)).abs() < 1e-9);
}

#[test]
fn arc_p_word_adds_full_turns() {
    let mut state = MotionState::default();
    let block = state.motion("G3 X0 Y2 I0 J1 P1 F600").unwrap();
    assert!((block.distance - 3.0 * std::f64::consts::PI).abs() < 1e-9);
}
