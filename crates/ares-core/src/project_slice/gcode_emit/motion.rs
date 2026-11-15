mod acceleration;
mod arc;
pub(super) mod auto_lift;
pub(super) mod clip;
mod extruder_registry;
mod extruder_state;
mod extrusion;
mod fan;
mod features;
pub(in crate::project_slice::gcode_emit) mod format;
mod jerk;
mod layer;
mod loop_paths;
mod materialized;
mod motion_state;
mod options;
mod origin;
mod overhang;
mod path;

mod perimeter;
mod scan;
mod scarf;
mod shared_reversals;
mod skirt_brim;
mod state;
mod support;

pub(super) mod tool_selection;
pub(super) mod tower;
mod travel;
mod writer_state;

pub(super) use acceleration::{set_accel_and_jerk, set_layer_acceleration_and_jerk};
pub(in crate::project_slice) use arc::{simplify_linear_points, simplify_points};
pub(super) use extruder_registry::ExtruderRegistry;
pub(super) use layer::emit_layer;
pub(super) use origin::set_origin;
pub(in crate::project_slice) use overhang::{
    CurlEstimate, CurledLine, ExternalEntityPoints, estimate_object,
};
pub(super) use shared_reversals::{EmissionPhase, SharedLayerId};
pub(super) use skirt_brim::{SkirtLoopParameters, emit_brim_loop, emit_skirt_loop};
pub(super) use state::{
    AvoidCrossingGeometry, EmitState, LayerBegin, LayerGeometry, LiftMode, append_exclude_end,
    append_object_start, begin_layer, begin_path_travel, queue_exclude_end, queue_exclude_start,
    queue_object_start, queue_object_stop_label,
};
pub(super) use support::{LayerAreas, emit as emit_support};
pub(super) use travel::{
    flush_pending_retract_eager_forced, flush_pending_retract_lift, flush_pending_retract_wipe,
    retract_for_custom_toolchange, retract_for_print_end, retract_for_toolchange,
};

use features::PathProperties;
pub(in crate::project_slice::gcode_emit) use motion_state::invalidate_after_custom_gcode;
pub(in crate::project_slice::gcode_emit) use options::{MotionOptions, first_nullable_float};
pub(in crate::project_slice::gcode_emit) use writer_state::{
    apply_writer_state, refresh_writer_state,
};

use crate::{
    geometry::Point,
    project_slice::{
        fill_entities::FillExtrusionEntity, perimeters::classic::gap_extrusion::GapFillEntity,
    },
};

pub(super) fn prepare_traditional_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::retract_for_timelapse(output, state);
}

/// The wrapping-detection per-layer retract (`GCode.cpp:5282/5303`,
/// `retract(false, false, auto_lift_type, true)`): retract + wipe + eager
/// lift. Like the timelapse retract, it still performs an allowed eager
/// lift when the tracked retraction is already complete.
pub(super) fn retract_for_wrapping(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::retract_for_wrapping(output, state);
}

pub(super) fn defer_layer_retraction(state: &mut EmitState) {
    // GCode.cpp:8466: change_layer retracts whether or not an in-layer snapshot left the
    // writer position unclear.
    if state.options.retract_when_changing_layer {
        state.pending_layer_retract = true;
    }
}

/// Defer the layer-change hop: upstream's `change_layer` retract runs
/// `maybe_zlift` (`GCodeWriter.cpp:626-648`) and `m_to_lift` survives into
/// the new layer's first travel, which raises to layer+hop and descends.
pub(super) fn defer_layer_change_lift(state: &mut EmitState) {
    travel::defer_layer_change_lift(state);
}

pub(super) fn end_layer_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    if state.options.retract_when_changing_layer && state.positioned {
        if state.traditional_timelapse && !state.has_insert_timelapse_gcode {
            // Traditional timelapse prints (i3 structure or multi-nozzle)
            // retract immediately at the layer end (`GCode.cpp:5527-5546`
            // fires `retract()` before the labels and timelapse template) —
            // UNLESS the mid-layer insert already ran
            // (`!has_insert_timelapse_gcode`, `GCode.cpp:5534`): then the
            // layer-end block is skipped entirely and the retraction +
            // label flush move into the next layer's `change_layer`
            // (`GCode.cpp:5690-5699`, the deferred path below).
            travel::retract_for_timelapse(output, state);
        } else {
            // Core-xy BBL (`GCode.cpp:5693` change_layer retract) and
            // compatible flavors emit the retraction inside the next layer's
            // CHANGE_LAYER block; `flush_pending_retract_wipe`/`_lift` emit
            // it there.
            state.pending_layer_retract = true;
        }
    }
}

/// Configured retraction at the start of the first layer — matches the
/// GCodeWriter `retract()` formatting (`GCodeWriter.cpp`).
pub(super) fn retract_before_layer(output: &mut Vec<u8>, state: &mut EmitState) {
    travel::plain_retract(output, state);
    if travel::effective_retraction_length(state) > 0.0 {
        state.extruder.retracted = true;
        travel::defer_layer_change_lift(state);
    }
    travel::reset_e(output, state, false);
}

/// The second-layer first-layer-inspection scan (`GCode.cpp:4780-4790`):
/// BBL printers with `scan_first_layer` retract around
/// `M976 S1 P1` + `M400 P100`. `retract()` emits nothing when the
/// extruder is already fully retracted (`Extruder::retract`,
/// `Extruder.cpp:77`); `unretract()` restores the tracked retracted
/// amount plus `retract_restart_extra` at the deretraction feedrate
/// (`Extruder::unretract`, `GCodeWriter.cpp:1063` zero-dE guard).
pub(super) fn scan_first_layer_block(output: &mut Vec<u8>, state: &mut EmitState) {
    scan::scan_first_layer_block(output, state);
}

#[derive(Clone, Copy)]
pub(super) struct SkirtLoopFlow {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) mm3_per_mm: f64,
}

fn emit_fill_entity(
    output: &mut Vec<u8>,
    entity: &FillExtrusionEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => path::emit(
            output,
            path.polyline
                .points()
                .iter()
                .map(|point| (point.x(), point.y())),
            PathProperties {
                contour: path.z_contoured.then_some(path.polyline.points.as_slice()),
                mm3_per_mm: path.mm3_per_mm,
                width: path.width,
                height: path.height,
                feature: features::for_fill(path.role),
                is_perimeter: matches!(
                    path.role,
                    crate::ExtrusionRole::Perimeter
                        | crate::ExtrusionRole::ExternalPerimeter
                        | crate::ExtrusionRole::OverhangPerimeter
                ),
                end_clip: 0.0,
                fitting: &path.polyline.fitting,
                slope: None,
            },
            geometry,
            state,
        ),
        FillExtrusionEntity::VariableWidth(entity) => {
            emit_variable_width_entity(output, entity, geometry, state)
        }
    }
}

fn emit_variable_width_entity(
    output: &mut Vec<u8>,
    entity: &GapFillEntity,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
) -> Result<(), crate::SliceError> {
    match entity {
        GapFillEntity::Path(path) => materialized::emit(
            output,
            state,
            materialized::Emission {
                path,
                end_clip: 0.0,
                slope: None,
                geometry,
            },
        ),
        GapFillEntity::Loop(paths) => {
            // `variable_width` (VariableWidth.cpp:224-227) closes up loop
            // entities; `GCode::extrude_loop` then splits them at the point
            // nearest to the last position (`GCode.cpp:5771`, the
            // non-perimeter `loop.split_at(last_pos, false)` branch).
            let mut loop_ = crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoop {
                paths: paths.clone(),
                role: crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoopRole::Internal,
                print_after_infill: false,
            };
            crate::project_slice::seam_placement::place_nearest_projection(
                &mut loop_,
                crate::project_slice::perimeters::classic::materialize::Point3 {
                    x: local_cursor(state, geometry).x(),
                    y: local_cursor(state, geometry).y(),
                    z: 0,
                },
                geometry.scale,
            );
            loop_paths::emit(output, (&loop_, None, None), geometry, state, &[])
        }
    }
}

pub(super) fn local_cursor(state: &EmitState, _geometry: LayerGeometry<'_>) -> Point {
    // `GCode.hpp:224`: numeric m_last_pos exists even while undefined for travel.
    Point::new(state.last_scaled_position.0, state.last_scaled_position.1)
}
