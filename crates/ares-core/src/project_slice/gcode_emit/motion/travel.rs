// Source boundary: OrcaSlicer v2.4.2 `GCode.cpp:310-358, 7400-7448` and
// `GCodeWriter.cpp` retract, wipe, and spiral-lift motion.

mod lift;
mod retract_core;
use retract_core::{firmware_retract, plain_retract};
#[cfg(test)]
mod tests;
mod wipe_path;

use super::{
    EmitState, arc, extrusion,
    format::{axis as format_axis, extrusion as format_extrusion},
};
pub(super) use lift::{
    emit_pending as emit_pending_lift, is_allowed_at as lift_is_allowed_at,
    mode_for as lift_mode_for, z_feedrate as lift_z_feedrate,
};
pub(in crate::project_slice::gcode_emit::motion) use wipe_path::scaled_position;
use wipe_path::{WipePath, wipe_moves};

pub(super) fn retract_and_lift(output: &mut Vec<u8>, state: &mut EmitState) {
    retract_and_wipe(output, state);
    if state.spiral_vase {
        lift::append_spiral_vase(output, state);
    } else {
        lift::schedule(state, false);
    }
    state.retracted = true;
}

pub(in crate::project_slice::gcode_emit) fn retract_for_print_end(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    // Upstream `m_writer.retract()` runs regardless of the retracted flag
    // (`GCode.cpp:7672`), so `m_restart_extra` is (re)assigned even on a
    // no-op retract (`Extruder.cpp:77`).
    state.restart_extra = state.options.retract_restart_extra;
    if !state.retracted {
        retract_and_wipe_with(output, state, false);
        state.retracted = true;
    }
    if state.spiral_vase {
        lift::append_spiral_vase(output, state);
    }
    state.pending_layer_retract = false;
}

/// `GCode.cpp:470-475`: the last wipe of the print emits no `;_WIPE` cooling
/// marker — no later cooling pass runs to clean it (print-end retraction
/// happens after the final layer flushes).
pub(super) fn retract_for_timelapse(output: &mut Vec<u8>, state: &mut EmitState) {
    state.restart_extra = state.options.retract_restart_extra;
    if state.retracted {
        return;
    }
    retract_and_wipe(output, state);
    lift::append_eager(output, state);
    state.retracted = true;
}

/// `retract(false, false, auto_lift_type, true)` for the wrapping-detection
/// insert (`GCode.cpp:5282/5303`): upstream `retract()` re-emits the wipe
/// only while a wipe path is stored and `Extruder::retract` adds just the
/// missing length (`std::max(0., length - m_retracted)`,
/// `Extruder.cpp:77-84`) — when the layer-change retract already consumed
/// both, only the eager lift is emitted. The lift itself always runs
/// (`needs_lift && can_lift` → `eager_lift`, `GCode.cpp:7693-7696`).
pub(super) fn retract_for_wrapping(output: &mut Vec<u8>, state: &mut EmitState) {
    state.restart_extra = state.options.retract_restart_extra;
    if !state.retracted {
        retract_and_wipe(output, state);
        state.retracted = true;
    }
    // The `needs_lift && can_lift` guards of the parent retract
    // (`GCode.cpp:7680-7703`) still apply to the wrap insert's eager lift:
    // `needs_lift` requires a nonzero retraction length or firmware
    // retraction, and `can_lift` honors `retract_lift_enforce`.
    let needs_lift = state.options.retraction_length > 0.0 || state.options.use_firmware_retraction;
    if needs_lift && lift::is_allowed_at(state, state.layer_z) {
        lift::append_eager(output, state);
    }
}

/// Non-BBL printers defer the layer-change retraction to after the next
/// layer marker block (`GCode.cpp` change_layer: the wipe lands between
/// the rendered `before_layer_change_gcode` and the cooling fan marker,
/// the lift after it).
/// `will_move_z` mirrors the upstream `change_layer` retract gate
/// (`GCode.cpp:5693`): when false the upstream retract call never runs,
/// so neither the E move nor the `m_restart_extra` assignment happens.
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_wipe(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    will_move_z: bool,
) {
    // Upstream `retract()` assigns `m_restart_extra` even when the extruder
    // is already retracted (`Extruder.cpp:77`).
    if state.pending_layer_retract && will_move_z {
        state.restart_extra = state.options.retract_restart_extra;
    }
    if state.pending_layer_retract && will_move_z && !state.retracted {
        retract_and_wipe(output, state);
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
        state.restart_extra = state.options.retract_restart_extra;
    }
    if state.pending_layer_retract && will_move_z && !state.retracted {
        lift::schedule_at(state, true, writer_z);
        state.retracted = true;
    }
    state.pending_layer_retract = false;
}

/// Core-xy BBL change-layer retraction: the wipe lands inside the new
/// layer's CHANGE_LAYER block and the lift is emitted eagerly right after
/// (`GCode.cpp:5693` change_layer retract, `GCode.cpp:5205-5210` second
/// retract whose E half is a no-op once retracted).
pub(in crate::project_slice::gcode_emit) fn flush_pending_retract_eager(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    flush_pending_retract_eager_with(output, state, false);
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
    if force || state.pending_layer_retract {
        state.restart_extra = state.options.retract_restart_extra;
    }
    if (force || state.pending_layer_retract) && !state.retracted {
        if force {
            // The BBL layer-start `retract()` finds no stored wipe path
            // there (`GCode.cpp:5205-5210`); the plain retraction emits
            // the full remaining length at the retraction feedrate.
            plain_retract(output, state);
        } else {
            retract_and_wipe(output, state);
        }
        state.retracted = true;
    }
    state.pending_layer_retract = false;
    // The eager lift supersedes any lift that the deferred path scheduled
    // for the layer's first travel.
    state.pending_lift = None;
    if state.retracted {
        lift::append_eager(output, state);
    }
}

fn retract_and_wipe(output: &mut Vec<u8>, state: &mut EmitState) {
    retract_and_wipe_with(output, state, true);
}

/// The Hilbert-suppressed tail (`GCode.cpp:7667-7672`) skips the final
/// `m_writer.retract()` — `m_restart_extra` keeps the before-wipe
/// fraction assigned by the surviving call
/// (`GCodeWriter.cpp:1006-1011`: `retract_before_wipe *
/// retract_restart_extra`).
pub(in crate::project_slice::gcode_emit) fn retract_and_wipe_suppressed_tail(
    output: &mut Vec<u8>,
    state: &mut EmitState,
) {
    retract_and_wipe_impl(output, state, true, true);
}

fn retract_and_wipe_with(output: &mut Vec<u8>, state: &mut EmitState, cooling_marker: bool) {
    retract_and_wipe_impl(output, state, cooling_marker, false);
}

fn retract_and_wipe_impl(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    cooling_marker: bool,
    suppressed_tail: bool,
) {
    // `Extruder::retract` assigns `m_restart_extra` on every call
    // (`Extruder.cpp:66,77`): the before-wipe call assigns
    // `retract_before_wipe * extra`, the post-wipe `retract()` call
    // reassigns the full `extra` (`GCodeWriter.cpp:1004-1011`).
    state.restart_extra = if suppressed_tail {
        state.options.retract_before_wipe * state.options.retract_restart_extra
    } else {
        state.options.retract_restart_extra
    };
    let WipePath {
        segments,
        retraction_distance,
        distribution_distance,
    } = wipe_moves(state);
    let requested_before = state.options.retraction_length * state.options.retract_before_wipe;
    let remaining = state.options.retraction_length - requested_before;
    let wipe_speed = if state.options.role_based_wipe_speed {
        state.extrusion_feedrate / 60.0
    } else {
        state.options.wipe_speed
    }
    .max(10.0);
    // `GCode.cpp:350-358`: the during-wipe amount is
    // `retraction_speed * unscale_(wipe_path_length) / wipe_speed` (the
    // effective, unrounded mm/s). Upstream caps at `remaining` only when
    // `during - remaining > EPSILON`; within the EPSILON window the
    // larger `during` value is emitted as-is.
    let during_max = state.options.retraction_during_wipe_speed * retraction_distance / wipe_speed;
    // Upstream EPSILON is `1e-4` (`libslic3r.h:52`), not machine eps:
    // `GCode.cpp:352-357` keeps `during` > remaining when within 1e-4.
    let during = if during_max - remaining > 1e-4 {
        remaining
    } else {
        during_max
    };
    // `before` = `retraction_before_wipe + (remaining - during)` =
    // `retraction_length - during` (`GCode.cpp:358` adds the leftover
    // difference into the pre-wipe amount). Without an eligible wipe
    // path upstream never calls `retract(true, beforeWipe)` — the single
    // `retract()` emits only the REMAINING distance
    // (`Extruder.cpp:75-84`), so a partially-retracted nozzle (e.g. a
    // template `e_retracted` assignment) tops up rather than restarting
    // the full length.
    let before = if segments.is_empty() {
        (state.options.retraction_length - state.retracted_amount).max(0.0)
    } else {
        // The before-wipe call emits `max(0, before - m_retracted)`
        // (`Extruder.cpp:77-84`); the post-wipe `retract()` tops the
        // remainder off to the full length.
        (state.options.retraction_length - during - state.retracted_amount).max(0.0)
    };
    if state.options.use_firmware_retraction {
        firmware_retract(output, state);
        state.retracted = true;
    } else if before > f64::EPSILON {
        // `extrusion::coordinate` accumulates `retracted_amount`
        // (`Extruder.cpp:77-84` semantics live in the shared counter).
        let retract = extrusion::coordinate(state, -before);
        let comment = state
            .options
            .gcode_comments
            .then_some(" ; retract")
            .unwrap_or("");
        output.extend_from_slice(
            format!(
                "G1 E{} F{}{comment}\n",
                format_extrusion(retract),
                format_axis(state.options.retraction_feedrate)
            )
            .as_bytes(),
        );
        state.current_feedrate = state.options.retraction_feedrate;
    }
    if !segments.is_empty() {
        let wipe_start = if state.tags.is_bbl() {
            "; WIPE_START\n"
        } else {
            ";WIPE_START\n"
        };
        output.extend_from_slice(wipe_start.as_bytes());
        let speed_line = if cooling_marker {
            format!("G1 F{};_WIPE\n", format_axis(wipe_speed * 60.0))
        } else {
            format!("G1 F{}\n", format_axis(wipe_speed * 60.0))
        };
        output.extend_from_slice(speed_line.as_bytes());
        state.current_feedrate = wipe_speed * 60.0;
        if let Ok(path) = std::env::var("ARES_WIPE_SEG") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(file, "AWIPE during={during} dist={distribution_distance}");
            }
        }
        for (point, segment_length) in segments {
            let retraction = during * (segment_length / distribution_distance);
            if let Ok(path) = std::env::var("ARES_WIPE_SEG") {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                {
                    let _ = writeln!(file, "ASEG slen={segment_length} dE={retraction}");
                }
            }
            // Orca's `extrude_to_xy` omits the E word when the wipe carries
            // no retraction (`retract_before_wipe` = 100% leaves nothing to
            // distribute during the wipe).
            let line = if retraction > f64::EPSILON {
                let retract = extrusion::coordinate(state, -retraction);
                let comment = state
                    .options
                    .gcode_comments
                    .then_some(" ; wipe and retract")
                    .unwrap_or("");
                format!(
                    "G1 X{} Y{} E{}{comment}\n",
                    format_axis(point.x),
                    format_axis(point.y),
                    format_extrusion(retract)
                )
            } else {
                format!("G1 X{} Y{}\n", format_axis(point.x), format_axis(point.y))
            };
            output.extend_from_slice(line.as_bytes());
            state.x = point.x;
            state.y = point.y;
            // Recover the scaled-int position the mm value came from
            // (upstream wipe moves hold ints; mm→int round-trips are
            // exact under round-to-nearest).
            // `(point - m_origin) + extruder_offset` order
            // (`GCode.cpp:8138-8144`).
            state.last_scaled_position = Some((
                ((point.x - state.origin.0 + state.extruder_offset.0) / state.scale_factor).round()
                    as i64,
                ((point.y - state.origin.1 + state.extruder_offset.1) / state.scale_factor).round()
                    as i64,
            ));
            state.wipe_start = Some(point);
        }
        let wipe_end = if state.tags.is_bbl() {
            "; WIPE_END\n"
        } else {
            ";WIPE_END\n"
        };
        output.extend_from_slice(wipe_end.as_bytes());
    }
    if !state.options.use_relative_e_distances && state.e_position.abs() > f64::EPSILON {
        output.extend_from_slice(b"G92 E0\n");
        state.e_position = 0.0;
    }
}

fn unscaled_position(point: (i64, i64), state: &EmitState) -> arc::Point {
    arc::Point {
        x: point.0 as f64 * state.scale_factor + state.origin.0 - state.extruder_offset.0,
        y: point.1 as f64 * state.scale_factor + state.origin.1 - state.extruder_offset.1,
    }
}

pub(super) fn inside_internal_surfaces(
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
    start: arc::Point,
    end: arc::Point,
    scale: crate::geometry::CoordinateScale,
    offset: (f64, f64),
) -> bool {
    use crate::project_slice::region_slices::RegionSurfaceKind;

    let point = |point: arc::Point| {
        crate::geometry::Point::new(
            scale.checked_scale(point.x - offset.0).unwrap(),
            scale.checked_scale(point.y - offset.1).unwrap(),
        )
    };
    let travel = crate::geometry::Polyline::new(vec![point(start), point(end)]);
    surfaces.iter().any(|surface| {
        let (kind, region, ..) = surface.as_parts();
        matches!(
            kind,
            RegionSurfaceKind::Internal
                | RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::InternalBridge
                | RegionSurfaceKind::InternalVoid
        ) && crate::geometry::open_polyline_inside_expolygon(&travel, region).unwrap()
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
