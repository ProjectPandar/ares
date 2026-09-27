//! First-travel approach emission — the `needs_travel` branch of
//! `start_travel::emit` (`GCode.cpp:6378-7501`, `GCodeWriter.cpp:685-806`).

use super::super::{PathProperties, retraction, travel_emit};
use super::dispatch;
use super::retract;
use super::route::{plan_route, polyline_length};
use crate::project_slice::gcode_emit::motion::features::{feature_description, is_bridge_feature};
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, append_object_start, arc, begin_path_travel,
    format::{axis as format_axis, z as format_z},
    travel,
};

pub(super) struct Approach<'a> {
    pub(super) first_scaled: (i64, i64),
    pub(super) first_x: f64,
    pub(super) first_y: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) first_position: bool,
    pub(super) never_positioned: bool,
    pub(super) layer_change_travel: bool,
    pub(super) slope_start_z: Option<f64>,
    pub(super) target_z: f64,
    pub(super) travel_distance: f64,
}

/// Returns `(travel_set_layer_z, eager_lifted_travel, unclear_position_travel)`.
pub(super) fn emit_needs_travel(
    output: &mut Vec<u8>,
    state: &mut EmitState,
    approach: Approach<'_>,
) -> (bool, bool, bool) {
    let Approach {
        first_scaled,
        first_x,
        first_y,
        properties,
        geometry,
        first_position,
        never_positioned,
        layer_change_travel,
        slope_start_z,
        target_z,
        travel_distance,
    } = approach;
    let mut travel_set_layer_z = false;
    let mut eager_lifted_travel = false;
    let mut unclear_position_travel = false;
    begin_path_travel(output, state, properties.feature, travel_distance);
    // `GCode.cpp:6378`: the path's first travel carries
    // "move to first {description} point" under `gcode_comments`.
    let first_travel_comment = state
        .options
        .gcode_comments
        .then(|| {
            let bridge = if is_bridge_feature(properties.feature) {
                " (bridge)"
            } else {
                ""
            };
            format!(
                " ; move to first {}{bridge} point",
                feature_description(properties.feature)
            )
        })
        .unwrap_or_default();
    let first_travel_comment = first_travel_comment.as_str();
    let inside_internal_surface = travel::inside_internal_surfaces(
        geometry.internal_surfaces,
        arc::Point {
            x: state.x,
            y: state.y,
        },
        arc::Point {
            x: first_x,
            y: first_y,
        },
        geometry.scale,
        state.offset,
    );
    let skip_retraction = super::super::can_skip_retraction(
        state.options.reduce_infill_retraction,
        state.options.has_sparse_infill,
        state.last_feature,
        properties.is_perimeter,
        inside_internal_surface,
    );
    // Orca decides the travel retraction on the routed polyline length
    // (`GCode.cpp:7424-7425` re-checks `needs_retraction` after
    // `avoid_crossing_perimeters.travel_to`), so route first and then
    // decide; a wiping retract moves the head and the route is planned
    // again from the new position (`GCode.cpp:7436-7443`).
    let mut route = plan_route(state, &geometry, properties.feature, first_x, first_y);
    let routed_length = polyline_length(state.x, state.y, &route);
    // Upstream calls `retract()` on every qualifying travel
    // (`needs_retraction` has no retracted check, `GCode.cpp:7528-7647`);
    // `Extruder::retract` emits `max(0, length - m_retracted)`
    // (`Extruder.cpp:75-84`) — a template-assigned PARTIAL retract tops
    // up rather than skipping.
    let retract = state.retracted_amount < travel::effective_retraction_length(state)
        && routed_length >= state.options.retraction_minimum_travel
        && !skip_retraction;
    {
        let geometry_ref = geometry;
        let feature = properties.feature;
        retract::decide(
            output,
            state,
            retract,
            routed_length,
            skip_retraction,
            feature,
            &mut |state| {
                route = plan_route(state, &geometry_ref, feature, first_x, first_y);
            },
        );
    }
    if std::env::var("ARES_DUMP_TRAVEL").is_ok() {
        eprintln!(
            "TV layer={} feat={} from=({:.3},{:.3}) to=({:.3},{:.3}) routed={:.3} min={:.3} retracted={} skip={} retract={} ext_once={} disabled_once={} first_pos={}",
            state.layer_index,
            properties.feature,
            state.x,
            state.y,
            first_x,
            first_y,
            routed_length,
            state.options.retraction_minimum_travel,
            state.retracted,
            skip_retraction,
            retract,
            state.use_external_mp_once,
            state.avoid_crossing_disabled_once,
            first_position,
        );
    }
    let first_travel = route[0];
    let (travel_x, travel_y) = (first_travel.x, first_travel.y);
    append_object_start(output, state);
    // Consume the one-shot disable after the travel is planned
    // (`reset_once_modifiers`, `GCode.cpp:7431`) — only routed
    // travels consume it; the skirt's own travels keep it armed.
    if !matches!(properties.feature, "Skirt" | "Brim") {
        state.avoid_crossing_disabled_once = false;
        state.use_external_mp_once = false;
    }
    // Upstream never schedules a layer-start lift without a retraction
    // (`GCode.cpp:5692-5698`: `change_layer`'s own retract — gated on
    // `retract_when_changing_layer` — is the only deferral source; with
    // it off, `m_to_lift` stays 0 and `travel_to_xyz` leaves the
    // destination at the layer z, `GCodeWriter.cpp:701-710`).
    // `GCodeWriter::travel_to_xyz` (`GCodeWriter.cpp:685-707`) only raises
    // the travel destination for a hop scheduled with this travel
    // (`m_to_lift`); a nozzle already lifted by an earlier sequence (the
    // layer-end timelapse retract, a deferred layer-change lift) travels
    // flat in XY and the unlift descends at the destination.
    let lifted_for_travel = state.pending_lift.is_some();
    let mut pending_target: Option<f64> = None;
    // The print's first travel has no known source position, so Orca
    // cannot slope/spiral from it (`GCodeWriter.cpp:travel_to_xyz`
    // skips those branches when `is_current_position_clear()` is
    // false). A pending normal lift still raises before the XY move
    // (`NormalLift` `slop_move`), and both paths end with the separate
    // `_travel_to_z` re-statement from the unclear-position branch.
    let first_travel_lift = if first_position {
        // Upstream: a first-travel lift exists via the travel's own
        // retraction (`GCode.cpp:7440` needs_retraction →
        // `GCodeWriter.cpp:626-648` maybe_zlift defers `m_to_lift`).
        // With the nozzle ALREADY retracted (the start gcode's
        // retract, e.g. Kobra), `retract()` still runs — dE is a
        // no-op but maybe_zlift defers. And with an UNCLEAR source
        // position upstream's travel polyline starts at (0,0), so
        // Upstream still applies the `retraction_minimum_travel` length
        // gate (`GCode.cpp:7530-7533`) — the (0,0)-origin travel can be
        // below it, in which case no retraction and no deferred hop run.
        // `needs_lift` (`GCode.cpp:7678-7681`): zero retraction length
        // (no firmware retraction) never schedules a deferred hop.
        if !skip_retraction
            && routed_length >= state.options.retraction_minimum_travel
            && state.pending_lift.is_none()
            && state.options.z_hop > 0.0
            && (state.options.retraction_length > 0.0 || state.options.use_firmware_retraction)
            && !state.spiral_vase
            // `lazy_lift` gates on the LIVE writer Z (`GCodeWriter.cpp:
            // 633-639`) — a template `position[2]` assignment moves the
            // eligibility origin off the nominal layer z.
            && travel::lift_is_allowed_at(
                state,
                state.writer_z.unwrap_or(state.layer_z),
            )
            && !state.lifted
        {
            state.pending_lift = Some(travel::lift_mode_for(state, false));
        }
        state.pending_lift.take()
    } else {
        pending_target = travel::emit_pending_lift(
            output,
            arc::Point {
                x: travel_x,
                y: travel_y,
            },
            state,
        );
        None
    };
    if state.spiral_vase
        && state.lifted
        && !first_position
        && (layer_change_travel
            || (state.layer_z + state.options.z_hop) - state.options.z_hop > state.layer_z)
    {
        travel_emit::xyz_with_comment(
            output,
            state,
            travel_x,
            travel_y,
            state.layer_z,
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if state.template_lifted && state.lifted && !first_position {
        // Pure XY while still lifted: upstream's no-Z-change branch calls
        // `travel_to_xy(to_2d(point))` WITHOUT the comment
        // (`GCodeWriter.cpp:771-780`).
        travel_emit::xy_with_comment(output, travel_x, travel_y, state.travel_feedrate, "");
        state.template_lifted = false;
    } else if state.lifted {
        eager_lifted_travel = true;
        if !lifted_for_travel {
            travel_emit::xy_with_comment(output, travel_x, travel_y, state.travel_feedrate, "");
        } else {
            // The lifted nozzle travels at the raised destination —
            // when the pending-lift preliminary move didn't fit (short
            // slope), upstream still targets `dest_point(2) = m_to_lift
            // + m_pos(2)` (`GCodeWriter.cpp:705-710`), which
            // `emit_pending` computed but did not emit as a Z move.
            let raised = pending_target.unwrap_or_else(|| state.writer_z.unwrap_or(state.layer_z));
            if (state.current_feedrate - state.travel_feedrate).abs() > f64::EPSILON {
                travel_emit::xyz_with_comment(
                    output,
                    state,
                    travel_x,
                    travel_y,
                    raised,
                    state.travel_feedrate,
                    first_travel_comment,
                );
            } else {
                output.extend_from_slice(
                    format!(
                        "G1 X{} Y{} Z{}{first_travel_comment}\n",
                        format_axis(travel_x),
                        format_axis(travel_y),
                        format_z(raised)
                    )
                    .as_bytes(),
                );
                state.record_z(raised);
            }
        }
    } else if layer_change_travel && state.retracted {
        // With a hop actually deferred, the earlier `state.lifted`
        // branch emitted the ramp and the combined layer+hop travel;
        // with no hop in play (the enforce gate blocked the
        // change-layer hop), `m_need_change_layer_lift_z`
        // (`GCode.cpp:7479-7482`) forces the plain combined xyz move.
        travel_emit::xyz_with_comment(
            output,
            state,
            travel_x,
            travel_y,
            target_z,
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else if first_position && (first_travel_lift.is_some() || state.pending_lift.is_some()) {
        // `m_to_lift` consumption is independent of the retracted flag
        // (`GCodeWriter.cpp:694-710`) — a deferred hop (scheduled by a
        // qualifying retraction, including a Hilbert-suppressed one or
        // the retract path's own deferral) always emits on the first
        // travel. No scheduled lift → no hop (`GCode.cpp:7530-7533`).
        let lift = first_travel_lift.or_else(|| state.pending_lift.take());
        dispatch::emit_retracted_first_travel(
            output,
            state,
            travel_x,
            travel_y,
            first_travel_comment,
            lift,
            // `is_current_position_clear` decides the XY+Z-restate vs
            // combined-XYZ shape (`GCodeWriter.cpp:748-777`) — a
            // machine-start `position` write-back seeds the cursor
            // (`m_last_pos`) but leaves the position unclear
            // (`GCode.cpp:3139-3140`), so track `state.positioned`, not
            // `never_positioned`.
            !state.positioned,
        );
    } else if layer_change_travel {
        // The sloped split only applies when a lift actually exists
        // (`GCodeWriter.cpp:725-757`: slope/spiral moves come from the
        // `m_to_lift` block; with no lift the plain combined XYZ path
        // emits the destination directly).
        let has_lift = state.lifted || state.pending_lift.is_some();
        if has_lift
            && state.options.z_hop > 0.0
            && retraction::uses_sloped_lift(state.options.z_hop_type)
        {
            travel_emit::xy_with_comment(
                output,
                travel_x,
                travel_y,
                state.travel_feedrate,
                first_travel_comment,
            );
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!("G1 Z{} F{}\n", format_z(target_z), format_axis(z_feedrate)).as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.record_z(target_z);
        } else {
            // `travel_to_xyz`'s force-z/`will_move_z` else-branch emits
            // the combined move at `config.travel_speed` UNCONDITIONALLY
            // (`GCodeWriter.cpp:783-806`) — the first-layer travel speed
            // does not apply to a layer-change approach that carries Z.
            travel_emit::xyz_with_comment(
                output,
                state,
                travel_x,
                travel_y,
                target_z,
                state.options.travel_feedrate,
                first_travel_comment,
            );
        }
        travel_set_layer_z = true;
    } else if let Some(z) = slope_start_z {
        travel_emit::xyz_with_comment(
            output,
            state,
            travel_x,
            travel_y,
            z,
            state.travel_feedrate,
            first_travel_comment,
        );
        travel_set_layer_z = true;
    } else {
        // Same `travel_to_xyz` else-branch: the unclear-position split
        // emits the XY leg at `config.travel_speed` without the
        // first-layer override (`GCodeWriter.cpp:793-798`).
        let xy_feedrate = if first_position {
            state.options.travel_feedrate
        } else {
            state.travel_feedrate
        };
        // The no-Z-change `travel_to_xy(to_2d(point))` branch drops the
        // comment for ordinary same-Z XY travels (`GCodeWriter.cpp:780`);
        // the unclear-position split (first travel) DOES emit it on both
        // legs (`GCodeWriter.cpp:754-763`).
        travel_emit::xy_with_comment(
            output,
            travel_x,
            travel_y,
            xy_feedrate,
            if first_position {
                first_travel_comment
            } else {
                ""
            },
        );
        // The print's first travel from an unknown position always
        // splits: the unclear-position branch of `travel_to_xyz`
        // (`GCodeWriter.cpp:754+`) emits XY then `_travel_to_z`
        // separately — `force_z` only merges Z for CLEAR-position
        // travels.
        if first_position {
            let z_feedrate = travel::lift_z_feedrate(state);
            output.extend_from_slice(
                format!(
                    "G1 Z{} F{}{first_travel_comment}\n",
                    format_z(target_z),
                    format_axis(z_feedrate)
                )
                .as_bytes(),
            );
            state.current_feedrate = z_feedrate;
            state.record_z(target_z);
            unclear_position_travel = true;
        }
    }
    state.x = travel_x;
    state.y = travel_y;
    let route_comment = first_travel_comment.to_string();
    for point in &route[1..] {
        // Upstream emits every route waypoint through
        // `GCodeWriter::travel_to_xy` (`GCode.cpp:7501`) WITH the F
        // word; the cooling buffer then strips the feedrate (unchanged)
        // or DROPS zero-length same-text waypoints
        // (`CoolingBuffer.cpp:911-913`). Emitting without F here
        // bypasses that drop and keeps near-duplicate waypoints the
        // oracle never ships.
        output.extend_from_slice(
            format!(
                "G1 X{} Y{} F{}{route_comment}\n",
                format_axis(point.x),
                format_axis(point.y),
                format_axis(state.travel_feedrate)
            )
            .as_bytes(),
        );
        state.x = point.x;
        state.y = point.y;
    }
    state.last_scaled_position = Some(first_scaled);
    state.positioned = true;
    state.current_feedrate = state.travel_feedrate;

    (
        travel_set_layer_z,
        eager_lifted_travel,
        unclear_position_travel,
    )
}
