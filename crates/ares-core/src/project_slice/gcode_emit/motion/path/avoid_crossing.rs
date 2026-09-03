//! Reduce-crossing-wall travel routing — a source-cited port of
//! `AvoidCrossingPerimeters::travel_to` (`GCode/AvoidCrossingPerimeters.cpp:
//! 1233-1312`) over the layer boundary.

mod boundary;
mod router;

#[cfg(test)]
mod tests;

use crate::project_slice::gcode_emit::motion::arc;

pub(in crate::project_slice::gcode_emit) use boundary::Boundary;
pub(super) mod rectangle;
pub(in crate::project_slice::gcode_emit) use rectangle::route as rectangle_route;

use super::super::state::LayerGeometry;

pub(super) struct Request<'a> {
    pub(super) start: arc::Point,
    pub(super) end: arc::Point,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) offset: (f64, f64),
    pub(super) inset: f64,
    pub(super) after_skirt: bool,
}

/// Build the routing boundary for a layer. Returns `None` when the layer has
/// no usable boundary (`travel_to` falls back to a straight line).
pub(in crate::project_slice::gcode_emit) fn build_boundary(
    geometry: &LayerGeometry<'_>,
) -> Option<Boundary> {
    Boundary::build(&geometry.avoid_crossing, geometry.scale)
        .ok()
        .flatten()
}

/// Route `start`→`end` along the boundary contours, mirroring
/// `AvoidCrossingPerimeters::travel_to`. The boundary is built once per
/// layer and cached in the emit state. Returns `Some(path)` for a routed
/// detour in G-code millimetres (without the endpoints) — or `None` while a
/// piece is still routed by the temporary rectangle shell (`rectangle.rs`):
/// detour waypoints require the multi-point ramp emission branch of
/// `GCodeWriter::travel_to_xyz` (`GCode.cpp:7486-7505`), which lifts Z over
/// the first detour leg and eases it over the last.
pub(super) fn route(request: Request<'_>, boundary: Option<&Boundary>) -> Option<Vec<arc::Point>> {
    // Detour waypoints require the multi-point ramp emission branch of
    // `GCodeWriter::travel_to_xyz` (`GCode.cpp:7486-7505`); until it lands,
    // all routing goes through the rectangle shell.
    if !detour_emission_ready() {
        return None;
    }
    let Request {
        start,
        end,
        geometry,
        offset,
        inset: _,
        after_skirt,
    } = request;
    // After the skirt the planner travels between objects — upstream sets
    // `use_external_mp_once` and routes against the external boundary
    // (`get_boundary_external`), whose entry travel stays direct
    // (`AvoidCrossingPerimeters.cpp:1272-1287`).
    if after_skirt {
        return Some(Vec::new());
    }
    let boundary = boundary?;
    let spacing = geometry.avoid_crossing.perimeter_spacing;
    let scale = geometry.scale;
    let to_scaled = |point: arc::Point| -> Option<crate::geometry::Point> {
        Some(crate::geometry::Point::new(
            scale.checked_scale(point.x - offset.0)?,
            scale.checked_scale(point.y - offset.1)?,
        ))
    };
    let scaled_start = to_scaled(start)?;
    let scaled_end = to_scaled(end)?;
    // Travels fully inside the lslices safe zone never route
    // (`any_expolygon_contains(m_lslices_offset, ...)`,
    // `AvoidCrossingPerimeters.cpp:1255`).
    if boundary.safe_zone_contains(scaled_start, scaled_end) {
        return Some(Vec::new());
    }
    let search_radius = 2.0 * f64::from(spacing);
    let search_radius = scale.checked_scale(search_radius).unwrap_or_default() as f64;
    let (path, _intersections) =
        router::avoid_perimeters(boundary, scaled_start, scaled_end, search_radius).ok()?;
    let mut output = Vec::with_capacity(path.len());
    for point in path {
        output.push(arc::Point {
            x: scale.unscale(point.x()) + offset.0,
            y: scale.unscale(point.y()) + offset.1,
        });
    }
    output.dedup_by(|next, last| {
        (next.x - last.x).abs() < 1.0e-4 && (next.y - last.y).abs() < 1.0e-4
    });
    // Upstream `travel.points` keeps the start at index 0 and emits from
    // index 1 (`GCode.cpp:7481-7505`); drop it so the first entry is the
    // first waypoint.
    if output.first().is_some_and(|point| {
        (point.x - start.x).abs() < 1.0e-4 && (point.y - start.y).abs() < 1.0e-4
    }) {
        output.remove(0);
    }
    Some(output)
}

fn detour_emission_ready() -> bool {
    true
}

pub(super) fn routing_active() -> bool {
    detour_emission_ready()
}
