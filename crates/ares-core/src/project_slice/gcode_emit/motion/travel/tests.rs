use super::super::{LiftMode, MotionOptions};
use super::wipe_path::wipe_moves;
use super::{
    EmitState, emit_pending_lift, flush_pending_retract_lift, inside_internal_surfaces,
    lift_is_allowed_at, retract_and_lift, retract_for_print_end,
};
use crate::{
    geometry::{ExPolygon, Point, Polygon, Polyline},
    project_slice::{gcode_emit::motion::arc, region_slices::RegionSurface},
};

fn internal_square() -> RegionSurface {
    RegionSurface::internal(ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 10_000_000),
            Point::new(0, 10_000_000),
        ]),
        Vec::new(),
    ))
}

#[test]
fn tower_writer_unlift_then_unretract_preserves_exact_commands() {
    let mut state = EmitState {
        writer_z: Some(0.8),
        layer_z: 0.6,
        lifted: true,
        lifted_amount: 0.2,
        pending_lift: Some(LiftMode::Normal),
        options: MotionOptions {
            gcode_comments: true,
            use_relative_e_distances: true,
            z_travel_feedrate: 600.0,
            deretraction_feedrate: 1800.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    state.extruder.retracted = true;
    state.extruder.retracted_amount = 0.5;
    state.extruder.restart_extra = 0.1;
    let mut output = Vec::new();
    super::unlift(&mut output, &mut state);
    super::unretract(&mut output, &mut state);
    assert_eq!(
        output,
        b"G1 Z.6 F600 ; restore layer Z\nG1 E.6 F1800 ;  ; unretract\n"
    );
    assert_eq!(state.extruder.retracted_amount, 0.0);
    assert_eq!(state.extruder.restart_extra, 0.0);
    assert_eq!(state.lifted_amount, 0.0);
    assert!(state.pending_lift.is_none());
}

#[test]
fn bottom_only_lift_does_not_lift_later_layers() {
    let mut state = EmitState {
        layer_chunk_index: 1,
        layer_index: 1,
        options: MotionOptions {
            retraction_length: 1.0,
            retraction_feedrate: 3_600.0,
            z_hop: 0.4,
            retract_lift_enforce: crate::RetractLiftEnforce::BottomOnly,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    retract_and_lift(&mut output, &mut state);

    assert_eq!(output, b"G1 E-1 F3600\n");
    assert!(state.extruder.retracted);
    assert_eq!(state.pending_lift, None);
}

#[test]
fn top_only_lift_requires_a_top_or_ironing_feature() {
    let mut state = EmitState {
        layer_z: 0.2,
        options: MotionOptions {
            z_hop: 0.4,
            retract_lift_enforce: crate::RetractLiftEnforce::TopOnly,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    assert!(!lift_is_allowed_at(&state, state.layer_z));
    state.last_notgapfill_feature = Some("Top surface");
    assert!(lift_is_allowed_at(&state, state.layer_z));
}

#[test]
fn deferred_bottom_only_lift_uses_the_target_layer_index() {
    let mut state = EmitState {
        layer_chunk_index: 1,
        layer_index: 1,
        pending_layer_retract: true,
        options: MotionOptions {
            z_hop: 0.4,
            retraction_length: 1.0,
            retract_lift_enforce: crate::RetractLiftEnforce::BottomOnly,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    let writer_z = state.layer_z;
    flush_pending_retract_lift(&mut Vec::new(), &mut state, writer_z, true);

    assert_eq!(state.pending_lift, None);
    assert!(state.extruder.retracted);
}

#[test]
fn layer_change_top_and_bottom_enforce_uses_the_new_layer_index() {
    // `change_layer` increments `m_layer_index` before the change-layer
    // retract (`GCode.cpp:5690-5696`; `m_layer_index` starts at -1 so the
    // first change-layer sees index 0). `retract_lift_enforce =
    // Top and Bottom` therefore only defers a hop when the NEW layer
    // index satisfies the gate (`GCode.cpp:7678-7705`): never via the
    // bottom clause after the first layer, only via a top/ironing last
    // role. LH Stinger (z_hop == layer step, Top and Bottom) relies on
    // the 0 -> 1 transition deferring nothing, so the layer travel is a
    // plain combined xyz move.
    let mut state = EmitState {
        layer_z: 0.4,
        writer_z: Some(0.2),
        // The state already advanced to the layer the change enters.
        layer_chunk_index: 1,
        layer_index: 1,
        options: MotionOptions {
            retraction_length: 1.0,
            z_hop: 0.2,
            z_hop_type: crate::ZHopType::Auto,
            retract_lift_enforce: crate::RetractLiftEnforce::TopAndBottom,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    super::defer_layer_change_lift(&mut state);

    assert_eq!(state.pending_lift, None);

    // The print's first change-layer (index 0) still defers the hop, and
    // a top-surface ending defers it on any layer.
    state.layer_chunk_index = 0;
    state.layer_index = 0;
    super::defer_layer_change_lift(&mut state);
    assert_eq!(state.pending_lift, Some(LiftMode::Spiral));

    state.layer_chunk_index = 7;
    state.layer_index = 7;
    state.last_notgapfill_feature = Some("Top surface");
    state.pending_lift = None;
    super::defer_layer_change_lift(&mut state);
    assert_eq!(state.pending_lift, Some(LiftMode::Spiral));
}

#[test]
fn slope_type_schedules_a_slope_lift() {
    let mut state = EmitState {
        layer_z: 0.4,
        options: MotionOptions {
            // `maybe_zlift` requires `needs_lift` (retraction_length > 0,
            // GCode.cpp:7678-7681).
            retraction_length: 1.0,
            z_hop: 0.4,
            z_hop_type: crate::ZHopType::Slope,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };

    retract_and_lift(&mut Vec::new(), &mut state);

    assert_eq!(state.pending_lift, Some(LiftMode::Slope));
}

#[test]
fn lift_above_gate_keeps_retraction_without_lifting() {
    let mut state = EmitState {
        layer_z: 0.4,
        options: MotionOptions {
            retraction_length: 1.0,
            retraction_feedrate: 3_600.0,
            z_hop: 0.4,
            retract_lift_above: 0.5,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    retract_and_lift(&mut output, &mut state);

    assert_eq!(output, b"G1 E-1 F3600\n");
    assert_eq!(state.pending_lift, None);
}

#[test]
fn degenerate_slope_uses_the_raised_target_without_a_preliminary_move() {
    let mut state = EmitState {
        layer_z: 0.4,
        pending_lift: Some(LiftMode::Slope),
        options: MotionOptions {
            z_hop: 0.4,
            travel_slope_radians: 0.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    // A degenerate slope fit emits no preliminary move but the lifted
    // state + raised destination still apply (`GCodeWriter.cpp:705-710`)
    // — the caller's final XYZ carries the raised target.
    let raised = emit_pending_lift(
        &mut output,
        arc::Point { x: 1.0, y: 1.0 },
        state.layer_z,
        &mut state,
        "",
    );
    assert_eq!(raised, Some(state.layer_z + state.options.z_hop));
    assert!(output.is_empty());
    assert!(state.lifted);
}

#[test]
fn print_end_retracts_without_a_layer_change_lift() {
    let mut state = EmitState {
        options: MotionOptions {
            retraction_length: 1.0,
            retraction_feedrate: 3_600.0,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    retract_for_print_end(&mut output, &mut state);

    assert_eq!(output, b"G1 E-1 F3600\n");
    assert!(state.extruder.retracted);
    assert!(!state.lifted);
}

#[test]
fn travel_fully_inside_internal_surface_avoids_retraction() {
    assert!(inside_internal_surfaces(
        &[internal_square()],
        &Polyline::new(vec![
            Point::new(1_000_000, 1_000_000),
            Point::new(9_000_000, 9_000_000),
        ]),
    ));
}

#[test]
fn travel_leaving_internal_surface_requires_retraction() {
    assert!(!inside_internal_surfaces(
        &[internal_square()],
        &Polyline::new(vec![
            Point::new(1_000_000, 1_000_000),
            Point::new(11_000_000, 9_000_000),
        ]),
    ));
}

#[test]
fn pending_spiral_lift_uses_resolution_based_linear_segments() {
    let mut state = EmitState {
        x: 102.379,
        y: 108.302,
        layer_z: 0.4,
        travel_feedrate: 12_000.0,
        pending_lift: Some(LiftMode::Spiral),
        options: MotionOptions {
            z_hop: 0.4,
            travel_slope_radians: 3.0_f64.to_radians(),
            arc_fitting_tolerance: 0.012,
            travel_feedrate: 12_000.0,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    assert!(
        emit_pending_lift(
            &mut output,
            arc::Point {
                x: 108.991,
                y: 108.991,
            },
            state.layer_z,
            &mut state,
            "",
        )
        .is_some()
    );

    assert_eq!(
        output,
        b"G1 F12000\n\
G1 X103.278 Y108.856 Z0.457143\n\
G1 X103.405 Y109.905 Z0.514286\n\
G1 X102.664 Y110.658 Z0.571429\n\
G1 X101.614 Y110.548 Z0.628571\n\
G1 X101.045 Y109.659 Z0.685714\n\
G1 X101.385 Y108.659 Z0.742857\n\
G1 X102.379 Y108.302 Z0.8\n"
    );
}

#[test]
fn wipe_distribution_uses_configured_distance_when_clipping_rounds_long() {
    let state = EmitState {
        scale_factor: 0.000_001,
        options: MotionOptions {
            wipe: true,
            wipe_distance: 1.0,
            retraction_during_wipe_distance: 1.0,
            ..MotionOptions::default()
        },
        wipe_start: Some(arc::Point { x: 0.0, y: 0.0 }),
        wipe_path: vec![
            arc::Point { x: 0.0, y: 0.0 },
            arc::Point { x: 0.6, y: 0.2 },
            arc::Point { x: -1.0, y: 0.4 },
        ],
        ..EmitState::default()
    };

    let path = wipe_moves(&state);
    let clipped_distance = path.segments.iter().map(|(_, length)| length).sum::<f64>();

    assert!(clipped_distance > 1_000_000.0);
    assert_eq!(path.retraction_distance, 1.0);
    assert_eq!(path.distribution_distance, 1_000_000.0);
}

#[test]
fn wipe_clip_end_empties_the_path_without_panicking() {
    // A configured wipe distance shorter than every segment consumes all
    // points; `Polyline::clip_end` (`Polyline.cpp:56-59`) returns an empty
    // polyline rather than popping the final point.
    let state = EmitState {
        scale_factor: 0.000_001,
        options: MotionOptions {
            wipe: true,
            wipe_distance: 0.0005,
            retraction_during_wipe_distance: 0.0005,
            ..MotionOptions::default()
        },
        wipe_start: Some(arc::Point { x: 0.0, y: 0.0 }),
        wipe_path: vec![
            arc::Point { x: 0.0, y: 0.0 },
            arc::Point { x: 0.2, y: 0.0 },
            arc::Point { x: 0.5, y: 0.0 },
        ],
        ..EmitState::default()
    };

    let path = wipe_moves(&state);

    assert!(path.segments.iter().map(|(_, len)| *len).sum::<f64>() <= 500.0 + 1.0e-6);
}
