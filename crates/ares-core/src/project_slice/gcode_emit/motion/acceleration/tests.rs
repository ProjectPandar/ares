use super::set_layer_acceleration_and_jerk;
use crate::project_slice::gcode_emit::motion::{EmitState, MotionOptions};

#[test]
fn klipper_layer_transition_emits_separate_acceleration_and_jerk() {
    let mut state = EmitState {
        options: MotionOptions {
            gcode_flavor: crate::GCodeFlavor::Klipper,
            accel_to_decel_enable: true,
            accel_to_decel_factor: 50.0,
            max_acceleration: 5_000,
            max_jerk_x: 20.0,
            max_jerk_y: 20.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    set_layer_acceleration_and_jerk(&mut output, &mut state, 1_000, 7.0);

    // `set_jerk_xy` appends the BBL ` Z{} E{}` tail outside the flavor
    // branches (GCodeWriter.cpp:296-301, whose comment admits it should be
    // in the else but is not), so with the default BBL tag set the Klipper
    // `SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY` line carries the tail too.
    assert_eq!(
        output,
        b"SET_VELOCITY_LIMIT ACCEL=1000 ACCEL_TO_DECEL=500\n\
SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY=7 Z0 E0\n"
    );
}
