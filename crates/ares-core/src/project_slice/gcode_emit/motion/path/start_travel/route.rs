//! `GCode::travel_to` routing and retraction-distance planning.
use super::super::avoid_crossing;
use crate::{
    SliceError,
    project_slice::gcode_emit::motion::{EmitState, LayerGeometry, arc},
};

/// Plan the travel route (detour waypoints plus the destination), routing
/// through the avoid-crossing boundary when armed (`GCode.cpp:7415-7434`).
pub(super) fn plan_route(
    state: &mut EmitState,
    geometry: &LayerGeometry<'_>,
    first_x: f64,
    first_y: f64,
) -> Result<Vec<arc::Point>, SliceError> {
    // Upstream gates routing on `is_current_position_clear()`
    // (`GCode.cpp:7420`).
    let route_gate = state.positioned;
    // The one-shot disable is armed at the END of the skirt/brim emission
    // (`GCode.cpp:4448-4450`, `:4498-4500`) and cleared immediately after
    // the first planning slot (`reset_once_modifiers`,
    // `GCode.cpp:7429-7431`) — BEFORE the retraction branch, so the
    // post-wipe re-plan (`GCode.cpp:7444-7452`) routes with the disable
    // already cleared.
    let avoid_disabled = state.avoid_crossing_disabled_once;
    // `GCode::travel_to` starts its polyline at last_pos(), not the writer's
    // inward-move endpoint (GCode.cpp:7356–7362).
    let cursor = crate::project_slice::gcode_emit::motion::local_cursor(state, *geometry);
    let start = arc::Point {
        x: geometry.scale.unscale(cursor.x()) + state.offset.0,
        y: geometry.scale.unscale(cursor.y()) + state.offset.1,
    };
    let mut route = if state.options.reduce_crossing_wall && route_gate && !avoid_disabled {
        if state.avoid_boundary.is_none() {
            state.avoid_boundary = Some(std::rc::Rc::new(
                avoid_crossing::build_boundary(geometry).map_err(|error| {
                    SliceError::InvalidInput(format!("avoid-crossing safe zone: {error:?}"))
                })?,
            ));
        }
        avoid_crossing::route(
            avoid_crossing::Request {
                start,
                end: arc::Point {
                    x: first_x,
                    y: first_y,
                },
                geometry: *geometry,
                offset: state.offset,
                origin: state.origin,
                // `AvoidCrossingPerimeters.cpp:1237`:
                // `m_use_external_mp || m_use_external_mp_once`.
                use_external: state.use_external_mp || state.use_external_mp_once,
                max_detour: state.options.max_travel_detour_distance,
            },
            std::rc::Rc::make_mut(
                state
                    .avoid_boundary
                    .as_mut()
                    .expect("boundary initialized above"),
            ),
        )?
    } else {
        Vec::new()
    };
    route.push(arc::Point {
        x: first_x,
        y: first_y,
    });
    Ok(route)
}

/// Scaled-domain routed length (`GCode.cpp:7528-7533`): upstream sums the
/// routed polyline's segment hypots in coord_t and compares against
/// `scale_(retraction_minimum_travel)`. Summing in mm loses a sub-ulp unit
/// on travels that reach the threshold exactly (e.g. a 0.6-0.8 diagonal at
/// `min = 1.0` becomes `0.9999999999999943 < 1.0`). Detour points
/// round-trip through mm, so they recover via `checked_scale_rounded`:
/// the mm representation of a coord_t detour vertex errs by < 0.5 unit,
/// so rounding restores the exact integer. The final point is the scaled
/// destination itself.
pub(super) fn polyline_length_scaled(
    origin: crate::geometry::Point,
    destination: crate::geometry::Point,
    route: &[arc::Point],
    scale: crate::geometry::CoordinateScale,
    offset: (f64, f64),
) -> f64 {
    let mut length = 0.0;
    let mut current = (origin.x(), origin.y());
    for (index, point) in route.iter().enumerate() {
        let next = if index + 1 == route.len() {
            (destination.x(), destination.y())
        } else {
            (
                scale
                    .checked_scale_rounded(point.x - offset.0)
                    .expect("avoid-crossing detour vertex stays in coord_t range"),
                scale
                    .checked_scale_rounded(point.y - offset.1)
                    .expect("avoid-crossing detour vertex stays in coord_t range"),
            )
        };
        length += ((next.0 - current.0) as f64).hypot((next.1 - current.1) as f64);
        current = next;
    }
    length
}
