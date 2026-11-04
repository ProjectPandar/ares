// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:310-358, 7400-7448` and
// `GCodeWriter.cpp` retract, wipe, and spiral-lift motion.

mod lift;
mod retract_core;
mod unretract;
pub(super) use unretract::{unlift, unretract};

mod wipe_execute;
mod wipe_path;
mod wipe_sequence;
pub(super) use retract_core::plain_retract;
pub(super) use wipe_execute::without_retraction as wipe_without_retraction;
pub(in crate::project_slice::gcode_emit) use wipe_sequence::gcode_comment;
pub(in crate::project_slice::gcode_emit) use wipe_sequence::reset_e;
pub(super) use wipe_sequence::retract_and_wipe;
pub(in crate::project_slice::gcode_emit) use wipe_sequence::{
    retract_and_wipe_suppressed_tail, wipe_only_retract,
};
use wipe_sequence::{retract_and_wipe_with, retract_sequence_due, wipe_eligible};

use super::{EmitState, arc};
pub(in crate::project_slice::gcode_emit) use lift::schedule_retract_tail;
pub(super) use lift::{
    EPSILON, emit_pending as emit_pending_lift, is_allowed_at as lift_is_allowed_at,
    mode_for as lift_mode_for, schedule_normal, schedule_tower_approach,
    z_feedrate as lift_z_feedrate,
};

/// Firmware retraction substitutes a fixed `1.0` retract length
/// (`GCodeWriter.cpp:1027-1037` `_retract` firmware branch) — the
/// configured `retraction_length` is irrelevant to the
/// already-retracted top-up decision.
pub(in crate::project_slice::gcode_emit) fn effective_retraction_length(state: &EmitState) -> f64 {
    if state.options.use_firmware_retraction {
        1.0
    } else {
        state.options.retraction_length
    }
}

/// The sequential by-object transition retract: `m_enable_cooling_markers
/// = false` (`GCode.cpp:3308`) strips the `;_WIPE` marker from the wipe.
pub(in crate::project_slice::gcode_emit) fn transition_retract(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    wipe_sequence::retract_and_wipe_with(output, state, false);
    schedule_normal(state);
    state.extruder.retracted = true;
}

/// `append_tcr`: retract(toolchange, false, configured lift, true).
pub(in crate::project_slice::gcode_emit) fn retract_for_type1(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    toolchange: bool,
) {
    let suppressed_tail = state.on_first_layer && state.options.bottom_surface_pattern_hilbert;
    wipe_sequence::retract_and_wipe_impl(output, state, true, suppressed_tail, toolchange);
    state.extruder.retracted = state.extruder.retracted_amount > 0.0;
    if lift::enforce_allows_lift(state) {
        lift::append_eager(output, state, false);
    }
}

pub(super) fn retract_and_lift(output: &mut Vec<u8>, state: &mut EmitState) {
    retract_and_wipe(output, state);
    if state.spiral_vase {
        lift::append_spiral_vase(output, state);
    } else {
        lift::schedule(state, false);
    }
    state.extruder.retracted = true;
}

/// `GCode::set_extruder`'s `retract(true, false)` and unconditional
/// `m_wipe.reset_path()` (`GCode.cpp:7751–7755`).
pub(in crate::project_slice::gcode_emit) fn retract_for_toolchange(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if state.extruder_selected {
        let suppressed_tail = state.on_first_layer && state.options.bottom_surface_pattern_hilbert;
        wipe_sequence::retract_and_wipe_impl(output, state, true, suppressed_tail, true);
        state.extruder.retracted = state.extruder.retracted_amount > 0.0;
        lift::schedule_for_toolchange(state);
    }
    state.wipe_path.clear();
    state.wipe_start = None;
}

/// Mandatory `retract(false, false, SpiralLift, true)` before custom change
/// processing (`GCode.cpp:7957`), independent of the configured hop type.
pub(in crate::project_slice::gcode_emit) fn retract_for_custom_toolchange(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    if !state.extruder_selected {
        return;
    }
    let suppressed_tail = state.on_first_layer && state.options.bottom_surface_pattern_hilbert;
    wipe_sequence::retract_and_wipe_impl(output, state, false, suppressed_tail, false);
    state.extruder.retracted = state.extruder.retracted_amount > 0.0;
    if lift::enforce_allows_lift(state) {
        lift::append_eager(output, state, true);
    }
}

pub(in crate::project_slice::gcode_emit) fn retract_for_print_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    // Upstream `m_writer.retract()` runs regardless of the retracted flag
    // (`GCode.cpp:7672`), so `m_restart_extra` is (re)assigned even on a
    // no-op retract (`Extruder.cpp:77`).
    state.extruder.restart_extra = state.options.retract_restart_extra;
    if retract_sequence_due(state) {
        retract_and_wipe_with(output, state, false);
        state.extruder.retracted = true;
    }
    // `reset_e` runs unconditionally after the retract/wipe block
    // (`GCode.cpp:7674`), independent of whether anything retracted.
    reset_e(output, state, false);
    if state.spiral_vase {
        lift::append_spiral_vase(output, state);
    }
    state.pending_layer_retract = false;
}

/// `GCode.cpp:470-475`: the last wipe of the print emits no `;_WIPE` cooling
/// marker — no later cooling pass runs to clean it (print-end retraction
/// happens after the final layer flushes).
pub(super) fn retract_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    state.extruder.restart_extra = state.options.retract_restart_extra;
    if retract_sequence_due(state) {
        retract_and_wipe(output, state);
        state.extruder.retracted = true;
    }
    reset_e(output, state, false);
    // GCode.cpp:7674-7704 lifts even when retraction is already complete.
    let needs_lift = state.options.retraction_length > 0.0 || state.options.use_firmware_retraction;
    if needs_lift && lift::is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z)) {
        lift::append_eager(output, state, false);
    }
}

/// `retract(false, false, auto_lift_type, true)` for the wrapping-detection
/// insert (`GCode.cpp:5282/5303`): upstream `retract()` re-emits the wipe
/// only while a wipe path is stored and `Extruder::retract` adds just the
/// missing length (`std::max(0., length - m_retracted)`,
/// `Extruder.cpp:77-84`) — when the layer-change retract already consumed
/// both, only the eager lift is emitted. The lift itself always runs
/// (`needs_lift && can_lift` → `eager_lift`, `GCode.cpp:7693-7696`).
pub(super) fn retract_for_wrapping(output: &mut Vec<u8>, state: &mut EmitState) {
    state.extruder.restart_extra = state.options.retract_restart_extra;
    if retract_sequence_due(state) {
        retract_and_wipe(output, state);
        state.extruder.retracted = true;
    }
    reset_e(output, state, false);
    // The `needs_lift && can_lift` guards of the parent retract
    // (`GCode.cpp:7680-7703`) still apply to the wrap insert's eager lift:
    // `needs_lift` requires a nonzero retraction length or firmware
    // retraction, and `can_lift` honors `retract_lift_enforce`.
    let needs_lift = state.options.retraction_length > 0.0 || state.options.use_firmware_retraction;
    if needs_lift && lift::is_allowed_at(state, state.writer_z.unwrap_or(state.layer_z)) {
        lift::append_eager(output, state, false);
    }
}

/// Non-BBL printers defer the layer-change retraction to after the next
/// layer marker block (`GCode.cpp` change_layer: the wipe lands between
/// the rendered `before_layer_change_gcode` and the cooling fan marker,
/// the lift after it).
/// `GCode.cpp:5692-5697`: `change_layer` retracts straight from the
/// config predicate (`retract_when_changing_layer && will_move_z`) —
/// there is no pending state to lose, and the stored wipe path survives
/// the layer change (`:5713-5715`), so the boundary flush decides from
/// config state, not a flag the tool transition clears
/// (`tool_transition/change.rs` resets `positioned`, disarming the
/// pending-flag arming under wiping overrides).
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_wipe(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    will_move_z: bool,
) {
    let change_layer_retract = state.options.retract_when_changing_layer && will_move_z;
    // Upstream `retract()` assigns `m_restart_extra` even when the extruder
    // is already retracted (`Extruder.cpp:77`).
    if change_layer_retract {
        state.extruder.restart_extra = state.options.retract_restart_extra;
    }
    if change_layer_retract && retract_sequence_due(state) {
        retract_and_wipe(output, state);
        state.extruder.retracted = true;
    }
    if change_layer_retract {
        reset_e(output, state, false);
    }
}

pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_lift(
    _output: &mut Vec<u8>,
    state: &mut EmitState,
    writer_z: f64,
    will_move_z: bool,
) {
    // Same `change_layer` gate as the wipe flush (`GCode.cpp:5693`).
    if state.pending_layer_retract && will_move_z {
        state.extruder.restart_extra = state.options.retract_restart_extra;
    }
    if state.pending_layer_retract
        && will_move_z
        && state.extruder.retracted_amount < effective_retraction_length(state)
    {
        lift::schedule_at(state, true, writer_z);
        state.extruder.retracted = true;
    }
    state.pending_layer_retract = false;
}

/// Mirrors `GCode.cpp:5205-5210`: the BBL non-traditional layer-start
/// fires `retract()` at EVERY change regardless of the pending flag
/// (labeled layers never arm it); `Extruder::retract` still emits
/// nothing once fully retracted.
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_eager_forced(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    flush_pending_retract_eager_with(output, state, true);
}

fn flush_pending_retract_eager_with(output: &mut Vec<u8>, state: &mut EmitState, force: bool) {
    // Upstream calls `retract()` regardless of the retracted flag — the
    // restart-extra assignment happens even on a no-op
    // (`Extruder.cpp:77`); only a nozzle already at FULL retraction
    // skips the E move.
    if force || state.pending_layer_retract {
        state.extruder.restart_extra = state.options.retract_restart_extra;
    }
    if force || state.pending_layer_retract {
        // The forced `retract()` runs its wipe whenever a stored path is
        // eligible — upstream keeps the previous layer's wipe path alive
        // across `change_layer` (`GCode.cpp:5713-5715`) even when the
        // remaining E distance is zero (`GCode.cpp:7659-7674`).
        if force {
            if wipe_eligible(state) {
                retract_and_wipe(output, state);
            } else if state.extruder.retracted_amount < effective_retraction_length(state) {
                // No stored wipe path (`GCode.cpp:5205-5210`); the plain
                // retraction emits the remaining length.
                plain_retract(output, state);
            }
            state.extruder.retracted = true;
        } else if retract_sequence_due(state) {
            retract_and_wipe(output, state);
            state.extruder.retracted = true;
        }
    }
    if force || state.pending_layer_retract {
        // `reset_e` runs unconditionally after every `retract()` call
        // (`GCode.cpp:7674`) — a fully-retracted nozzle still clears the
        // counter the template assignments may have left behind.
        reset_e(output, state, false);
    }
    state.pending_layer_retract = false;
    // The eager lift supersedes any lift that the deferred path scheduled
    // for the layer's first travel.
    state.pending_lift = None;
    if state.extruder.retracted && lift::enforce_allows_lift(state) {
        lift::append_eager(output, state, false);
    }
}

/// `travel_inside_internal_regions`
/// (`RetractWhenCrossingPerimeters.cpp:9-51`): the travel polyline stays in the
/// scaled-int domain — upstream never leaves coord_t for this check. Passing
/// millimeters back through `checked_scale` (truncating) loses a unit to the
/// unscale/offset roundtrip, which lands a boundary-collinear travel one unit
/// inside the island and flips `diff_pl` empty.
///
/// The polyline is whatever `needs_retraction` is about to examine: the
/// straight `[m_last_pos, destination]` pair on the first call
/// (`GCode.cpp:7359`), and the avoid-crossing detour once routing has replaced
/// `travel` (`GCode.cpp:7421-7425`).
pub(super) fn inside_internal_surfaces<'a>(
    surfaces: impl IntoIterator<Item = &'a crate::project_slice::region_slices::RegionSurface>,
    travel: &crate::geometry::Polyline,
) -> bool {
    use crate::project_slice::region_slices::RegionSurfaceKind;
    surfaces.into_iter().any(|surface| {
        let (kind, region, ..) = surface.as_parts();
        matches!(
            kind,
            RegionSurfaceKind::Internal
                | RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::InternalBridge
                | RegionSurfaceKind::InternalVoid
        ) && crate::geometry::open_polyline_inside_expolygon(travel, region).unwrap()
    })
}

/// Defer the layer-change hop: upstream's `change_layer` retract runs
/// `maybe_zlift` (`GCodeWriter.cpp:626-648`) and `m_to_lift` survives into
/// the new layer's first travel, which raises to layer+hop and descends.
pub(in crate::project_slice::gcode_emit) fn defer_layer_change_lift(state: &mut EmitState) {
    // Upstream `change_layer` retracts while the writer still sits at the
    // previous layer's z (`GCode.cpp:5693-5706` before the z move), so the
    // `retract_lift_above/below` gate evaluates at that z, not the new
    // layer's (`GCodeWriter.cpp:633-639`). `schedule_at` itself applies
    // the parent's `needs_lift` guard (`GCode.cpp:7678`).
    let writer_z = state.writer_z.map_or(state.layer_z, f64::from);
    lift::schedule_at(state, true, writer_z);
}
