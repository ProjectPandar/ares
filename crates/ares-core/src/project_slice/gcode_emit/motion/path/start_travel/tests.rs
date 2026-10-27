//! `retract::decide` regression coverage — Hilbert suppression and the
//! already-retracted `restart_extra` assignment (`GCode.cpp:7667-7672`,
//! `Extruder.cpp:77`).

use super::retract::{RetractContext, decide};
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
fn tower_travel_uses_native_comment_without_extrusion_z_restatement() {
    use crate::{
        geometry::CoordinateScale,
        project_slice::gcode_emit::motion::{AvoidCrossingGeometry, LayerGeometry},
    };
    let mut state = EmitState {
        positioned: true,
        last_scaled_position: (0, 0),
        last_position_defined: true,
        writer_z: Some(0.0),
        layer_z: 0.2,
        layer_change_travel_pending: true,
        travel_feedrate: 1200.0,
        options: MotionOptions {
            gcode_comments: true,
            retraction_length: 0.0,
            retraction_minimum_travel: 1000.0,
            travel_feedrate: 1200.0,
            first_layer_travel_feedrate: 1200.0,
            default_acceleration: 0.0,
            default_jerk: 0.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let geometry = LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: &[],
        support_islands: None,
        scale: CoordinateScale::Normal,
        previous_curled: &[],
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            layer_slices: &[],
            perimeter_spacing: 0.4,
            external_perimeter_width: 0.4,
            top_surfaces: &[],
            chunk_slices: &[],
            chunk_perimeter_spacing: 0.4,
        },
    };
    let mut output = Vec::new();
    super::tower_travel(&mut output, &mut state, (10_000_000, 20_000_000), geometry).unwrap();
    assert_eq!(output, b"G1 X10 Y20 Z.2 F1200 ; Travel to a Wipe Tower\n");
    assert_eq!(state.last_scaled_position, (10_000_000, 20_000_000));
    assert!(state.last_position_defined);
    assert!(!state.layer_change_travel_pending);
}

#[test]
fn first_layer_hilbert_bottom_suppresses_writer_retract_and_extra() {
    let mut state = state();
    state.extruder.retracted = true;
    state.extruder.restart_extra = 0.5;
    let mut output = Vec::new();
    // A long, non-exempt already-retracted travel on the Hilbert first
    // layer: upstream `_retract` skips `m_writer.retract()` entirely, so
    // `m_restart_extra` keeps the template/stale value (`Extruder.cpp:77`
    // never runs).
    decide(
        &mut output,
        &mut state,
        RetractContext {
            retract: true,
            route_reaches_minimum: true,
            skip_retraction: false,
            destination_feature: "Internal infill",
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(state.extruder.restart_extra, 0.5);
    assert!(output.is_empty());
}

#[test]
fn hilbert_suppression_still_emits_reset_e_in_absolute_mode() {
    // `G92 E0` (`GCode.cpp:7674`) is outside the suppression — only
    // `m_writer.retract()` is gated.
    let mut state = state();
    state.options.use_relative_e_distances = false;
    // `reset_e` emits only when the E counter is nonzero
    // (`GCodeWriter.cpp:486-489`).
    state.extruder.e_position = 0.4;
    let mut output = Vec::new();
    decide(
        &mut output,
        &mut state,
        RetractContext {
            retract: true,
            route_reaches_minimum: true,
            skip_retraction: false,
            destination_feature: "Internal infill",
        },
        &mut |_| Ok(()),
    )
    .unwrap();
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
    state.extruder.retracted = true;
    state.extruder.restart_extra = 0.5;
    let mut output = Vec::new();
    decide(
        &mut output,
        &mut state,
        RetractContext {
            retract: true,
            route_reaches_minimum: true,
            skip_retraction: false,
            destination_feature: "Top surface",
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(state.extruder.restart_extra, 0.5);
    // A non-top destination on the same layer still assigns.
    decide(
        &mut output,
        &mut state,
        RetractContext {
            retract: false,
            route_reaches_minimum: true,
            skip_retraction: false,
            destination_feature: "Internal infill",
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(
        state.extruder.restart_extra,
        state.options.retract_restart_extra
    );
}

#[test]
fn non_hilbert_already_retracted_travel_assigns_restart_extra() {
    let mut state = EmitState {
        layer_index: 3,
        ..EmitState::default()
    };
    state.extruder.retracted = true;
    state.extruder.restart_extra = 0.5;
    state.options.retract_restart_extra = 0.0;
    let mut output = Vec::new();
    decide(
        &mut output,
        &mut state,
        RetractContext {
            retract: false,
            route_reaches_minimum: true,
            skip_retraction: false,
            destination_feature: "Internal infill",
        },
        &mut |_| Ok(()),
    )
    .unwrap();
    assert_eq!(state.extruder.restart_extra, 0.0);
}
