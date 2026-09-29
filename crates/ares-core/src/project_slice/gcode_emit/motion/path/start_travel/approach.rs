//! First-travel approach emission — the `needs_travel` branch of
//! `start_travel::emit` (`GCode.cpp:6378-7501`, `GCodeWriter.cpp:685-806`).

use super::super::PathProperties;
mod move_emit;
mod waypoints;

use super::dispatch;
use super::retract;
use super::route::{plan_route, polyline_length};
use crate::project_slice::gcode_emit::motion::features::{feature_description, is_bridge_feature};
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, append_object_start, arc, begin_path_travel, travel,
};

pub(super) struct Approach<'a> {
    pub(super) first_scaled: (i64, i64),
    pub(super) first_x: f64,
    pub(super) first_y: f64,
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) first_position: bool,
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
    let first_travel_comment = if state.options.gcode_comments {
        {
            let bridge = if is_bridge_feature(properties.feature) {
                " (bridge)"
            } else {
                ""
            };
            format!(
                " ; move to first {}{bridge} point",
                feature_description(properties.feature)
            )
        }
    } else {
        Default::default()
    };
    let first_travel_comment = first_travel_comment.as_str();
    // Upstream's `travel` polyline anchors at `m_last_pos` — the last
    // EXTRUDED point (`GCode.cpp:7356`), not the head position left by a
    // clipped loop-tail / wipe move. `local_cursor` returns exactly that
    // scaled-int anchor (`motion::local_cursor`), so the retraction length
    // and internal-region checks measure from the pre-clip position.
    let cursor = crate::project_slice::gcode_emit::motion::local_cursor(state, geometry);
    let travel_origin = arc::Point {
        x: cursor.x() as f64 * geometry.scale.factor() + state.offset.0,
        y: cursor.y() as f64 * geometry.scale.factor() + state.offset.1,
    };
    let inside_internal_surface = travel::inside_internal_surfaces(
        geometry.internal_surfaces,
        travel_origin,
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
    let routed_length = polyline_length(travel_origin.x, travel_origin.y, &route);
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
            && state.lifted_amount <= 0.0
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
            // The upstream lift target compares against the travel's
            // destination z for THIS leg (`GCodeWriter.cpp:701-709`):
            // `travel_to_xyz` lifts to the NOMINAL z on the first
            // detour waypoint and applies the scarf z only on the
            // route's final leg (`GCode.cpp:7488-7498`).
            if route.len() > 1 {
                // First DETOUR waypoint lifts to the NOMINAL layer z
                // (`m_nominal_z`, `GCode.cpp:7489-7492`) — `target_z`
                // already resolves to the scarf start when a seam is
                // present (`start_travel.rs`).
                state.layer_z
            } else {
                slope_start_z.unwrap_or(target_z)
            },
            state,
        );
        None
    };
    let move_outcome = move_emit::emit_approach_move(
        output,
        state,
        move_emit::MoveContext {
            travel_x,
            travel_y,
            first_travel_comment,
            first_position,
            layer_change_travel,
            lifted_for_travel,
            pending_target,
            first_travel_lift,
            slope_start_z,
            target_z,
            route_len: route.len(),
        },
    );
    travel_set_layer_z |= move_outcome.travel_set_layer_z;
    eager_lifted_travel |= move_outcome.eager_lifted_travel;
    unclear_position_travel |= move_outcome.unclear_position_travel;
    state.x = travel_x;
    state.y = travel_y;
    let route_comment = first_travel_comment.to_string();
    travel_set_layer_z |=
        waypoints::emit_detour_waypoints(output, state, &route, slope_start_z, &route_comment);
    state.last_scaled_position = Some(first_scaled);
    state.positioned = true;
    state.current_feedrate = state.travel_feedrate;

    (
        travel_set_layer_z,
        eager_lifted_travel,
        unclear_position_travel,
    )
}
