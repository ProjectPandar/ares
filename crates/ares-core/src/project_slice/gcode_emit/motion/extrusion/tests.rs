use super::{EmitState, PathProperties, arc, linear_segment};

#[test]
fn zero_flow_move_omits_extrusion_axis() {
    let mut state = EmitState {
        options: super::super::MotionOptions {
            filament_flow_ratio: 1.0,
            print_flow_ratio: 1.0,
            filament_area: 2.4,
            use_relative_e_distances: true,
            ..super::super::MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    linear_segment(
        &mut output,
        arc::Point { x: 1.0, y: 2.0 },
        3.0,
        PathProperties {
            mm3_per_mm: 0.0,
            width: 0.45,
            height: 0.2,
            feature: "Bottom surface",
            is_perimeter: false,
            end_clip: 0.0,
            fitting: &[],
        },
        &mut state,
    );

    assert_eq!(output, b"G1 X1 Y2\n");
    assert_eq!(state.e_position, 0.0);
}
