//! `retract::decide` regression coverage — Hilbert suppression and the
//! already-retracted `restart_extra` assignment (`GCode.cpp:7667-7672`,
//! `Extruder.cpp:77`).

use super::retract::decide;
use crate::project_slice::gcode_emit::motion::{EmitState, MotionOptions};

fn state() -> EmitState {
    EmitState {
        layer_index: 0,
        options: MotionOptions {
            bottom_surface_pattern_hilbert: true,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    }
}

#[test]
fn first_layer_hilbert_bottom_suppresses_writer_retract_and_extra() {
    let mut state = state();
    state.retracted = true;
    state.restart_extra = 0.5;
    let mut output = Vec::new();
    // A long, non-exempt already-retracted travel on the Hilbert first
    // layer: upstream `_retract` skips `m_writer.retract()` entirely, so
    // `m_restart_extra` keeps the template/stale value (`Extruder.cpp:77`
    // never runs).
    decide(
        &mut output,
        &mut state,
        true,
        10.0,
        false,
        "Internal infill",
        &mut |_| {},
    );
    assert_eq!(state.restart_extra, 0.5);
    assert!(output.is_empty());
}

#[test]
fn hilbert_suppression_still_emits_reset_e_in_absolute_mode() {
    // `G92 E0` (`GCode.cpp:7674`) is outside the suppression — only
    // `m_writer.retract()` is gated.
    let mut state = state();
    state.options.use_relative_e_distances = false;
    let mut output = Vec::new();
    decide(
        &mut output,
        &mut state,
        true,
        10.0,
        false,
        "Internal infill",
        &mut |_| {},
    );
    assert_eq!(output, b"G92 E0\n");
}

#[test]
fn hilbert_top_surface_suppresses_top_infill_destination_only() {
    let mut state = EmitState {
        layer_index: 3,
        options: MotionOptions {
            top_surface_pattern_hilbert: true,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    state.retracted = true;
    state.restart_extra = 0.5;
    let mut output = Vec::new();
    decide(
        &mut output,
        &mut state,
        true,
        10.0,
        false,
        "Top surface",
        &mut |_| {},
    );
    assert_eq!(state.restart_extra, 0.5);
    // A non-top destination on the same layer still assigns.
    decide(
        &mut output,
        &mut state,
        false,
        10.0,
        false,
        "Internal infill",
        &mut |_| {},
    );
    assert_eq!(state.restart_extra, state.options.retract_restart_extra);
}

#[test]
fn non_hilbert_already_retracted_travel_assigns_restart_extra() {
    let mut state = EmitState {
        layer_index: 3,
        ..EmitState::default()
    };
    state.retracted = true;
    state.restart_extra = 0.5;
    state.options.retract_restart_extra = 0.0;
    let mut output = Vec::new();
    decide(
        &mut output,
        &mut state,
        false,
        10.0,
        false,
        "Internal infill",
        &mut |_| {},
    );
    assert_eq!(state.restart_extra, 0.0);
}
