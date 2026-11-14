//! `GCode::travel_to` routing and retraction-distance planning.
use super::super::avoid_crossing;
use crate::{
    SliceError,
    project_slice::gcode_emit::motion::{EmitState, LayerGeometry, arc},
};

/// Initial planning conditions (`GCode.cpp:10570–10578`, OrcaSlicer 2.5.0).
pub(super) fn plan_route(
    state: &mut EmitState,
    geometry: &LayerGeometry<'_>,
    first_x: f64,
    first_y: f64,
) -> Result<Vec<arc::Point>, SliceError> {
    let avoid = state.options.reduce_crossing_wall
        && state.positioned
        && !state.avoid_crossing_disabled_once;
    route_from_cursor(
        state,
        geometry,
        arc::Point {
            x: first_x,
            y: first_y,
        },
        avoid,
    )
}

/// Post-retraction planning (`GCode.cpp:10605–10611`, OrcaSlicer 2.5.0)
/// does not repeat the initial writer-position or one-shot-disable conditions.
pub(super) fn replan_after_retract(
    state: &mut EmitState,
    geometry: &LayerGeometry<'_>,
    first_x: f64,
    first_y: f64,
) -> Result<Vec<arc::Point>, SliceError> {
    let avoid = state.options.reduce_crossing_wall;
    route_from_cursor(
        state,
        geometry,
        arc::Point {
            x: first_x,
            y: first_y,
        },
        avoid,
    )
}

fn route_from_cursor(
    state: &mut EmitState,
    geometry: &LayerGeometry<'_>,
    end: arc::Point,
    avoid: bool,
) -> Result<Vec<arc::Point>, SliceError> {
    // Plan from the scaled generator cursor, not the inward writer endpoint.
    let cursor = crate::project_slice::gcode_emit::motion::local_cursor(state, *geometry);
    let start = arc::Point {
        x: geometry.scale.unscale(cursor.x()) + state.offset.0,
        y: geometry.scale.unscale(cursor.y()) + state.offset.1,
    };
    let mut route = if avoid {
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
                end,
                geometry: *geometry,
                offset: state.offset,
                origin: state.origin,
                // `AvoidCrossingPerimeters.cpp:1259–1262`, OrcaSlicer 2.5.0:
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
    route.push(end);
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
