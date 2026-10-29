#[test]
fn third_layer_does_not_reassert_default_kinematics() {
    let mut state = super::EmitState {
        options: super::MotionOptions {
            gcode_flavor: crate::GCodeFlavor::Klipper,
            default_acceleration: 8_000.0,
            default_jerk: 9.0,
            max_acceleration: 12_000,
            max_jerk_x: 12.0,
            max_jerk_y: 12.0,
            ..super::MotionOptions::default()
        },
        last_acceleration: Some(8_000),
        last_jerk: Some(12.0),
        ..super::EmitState::default()
    };
    let mut output = Vec::new();

    super::begin_layer(
        &mut output,
        &mut state,
        super::LayerBegin {
            layer_z: 0.6,
            layer_height: 0.2,
            first_layer: false,
            second_layer_reset: false,
        },
    );

    assert!(output.is_empty());
}

#[test]
fn internal_travel_retraction_matches_source_role_rules() {
    assert!(super::path::can_skip_retraction(
        true,
        true,
        Some("Sparse infill"),
        false,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        true,
        Some("Outer wall"),
        false,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        true,
        Some("Overhang wall"),
        false,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        true,
        Some("Sparse infill"),
        true,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        false,
        Some("Sparse infill"),
        false,
        true,
    ));
}
