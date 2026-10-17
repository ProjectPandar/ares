//! Reduce-crossing-wall travel routing — a source-cited port of
//! `AvoidCrossingPerimeters::travel_to` (`GCode/AvoidCrossingPerimeters.cpp:
//! 1233-1312`) over the layer boundary.

mod boundary;
mod router;
mod safe_zone;

#[cfg(test)]
mod tests;

use crate::{options::FloatOrPercent, project_slice::gcode_emit::motion::arc};

/// `init_layer`'s safe zone and lazily initialized `m_internal` boundary.
#[derive(Clone)]
pub(in crate::project_slice::gcode_emit) struct Boundary {
    safe_zone: Vec<crate::geometry::ExPolygon>,
    internal: Option<boundary::Boundary>,
    external: Option<boundary::Boundary>,
}
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
    /// `m_use_external_mp_once` — route along the chunk-wide hole boundary
    /// (`AvoidCrossingPerimeters.cpp:1553-1556`).
    pub(super) use_external: bool,
    /// `print.max_travel_detour_distance` — detour cap consumed at
    /// `AvoidCrossingPerimeters.cpp:1291-1302`. `Float(0)`/non-positive
    /// disables the cap (`opt.value > 0` gate).
    pub(super) max_detour: FloatOrPercent,
}

/// Initialize the layer safe zone; routing geometry is built only after a
/// travel fails containment (`AvoidCrossingPerimeters.cpp:1250-1258`).
pub(in crate::project_slice::gcode_emit) fn build_boundary(
    geometry: &LayerGeometry<'_>,
) -> Option<Boundary> {
    Some(Boundary {
        safe_zone: safe_zone::build(&geometry.avoid_crossing, geometry.scale).ok()?,
        internal: None,
        external: None,
    })
}

/// Route `start`→`end` along the boundary contours, mirroring
/// `AvoidCrossingPerimeters::travel_to`. Rebuild the cached internal boundary
/// when either endpoint leaves its original (pre-EdgeGrid expansion) bounds.
/// Return interior waypoints in G-code millimetres; the caller owns endpoints.
/// `None` still reaches the existing temporary rectangle shell.
pub(super) fn route(
    request: Request<'_>,
    boundary: Option<&mut Boundary>,
) -> Option<Vec<arc::Point>> {
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
        after_skirt: _,
        use_external,
        max_detour,
    } = request;
    let boundary = boundary?;
    let scale = geometry.scale;
    // The mm cursor round-trips through `unscale(int) + offset`, which lands
    // a few ULPs below the exact value; a truncating cast would drop one
    // lattice unit and skew every route waypoint (qVMcEP X3946461 vs
    // upstream's exact X3946462). Upstream routes on `m_last_pos` ints, so
    // round the conversion back to the originating integer.
    let to_scaled = |point: arc::Point| -> Option<crate::geometry::Point> {
        Some(crate::geometry::Point::new(
            scale.checked_scale_rounded(point.x - offset.0)?,
            scale.checked_scale_rounded(point.y - offset.1)?,
        ))
    };
    let scaled_start = to_scaled(start)?;
    let scaled_end = to_scaled(end)?;
    // Travels fully inside the lslices safe zone never route
    // (`any_expolygon_contains(m_lslices_offset, ...)`,
    // `AvoidCrossingPerimeters.cpp:1255`).
    if boundary.safe_zone.is_empty()
        || boundary
            .safe_zone
            .iter()
            .any(|polygon| safe_zone::contains(scaled_start, scaled_end, polygon))
    {
        return Some(Vec::new());
    }
    if boundary
        .internal
        .as_ref()
        .is_none_or(|internal| !internal.contains(scaled_start) || !internal.contains(scaled_end))
        && !use_external
    {
        let built = if let Some(spacing) = geometry
            .support_islands
            .and_then(|layer| layer.scaled_default_perimeter_spacing)
        {
            boundary::Boundary::build_with_spacing(
                &geometry.avoid_crossing,
                scale,
                [scaled_start, scaled_end],
                spacing,
            )
        } else {
            boundary::Boundary::build(&geometry.avoid_crossing, scale, [scaled_start, scaled_end])
        };
        boundary.internal = match built.ok()? {
            boundary::BuildResult::Unavailable => None,
            // `travel_to` (:1259–1264, :1285–1288) skips an empty internal
            // boundary and returns {start, end}, not a rectangle detour.
            boundary::BuildResult::Empty => {
                boundary.internal = None;
                return Some(Vec::new());
            }
            boundary::BuildResult::Ready(internal) => Some(internal),
        };
    }
    let boundary = if use_external {
        // `get_boundary_external` builds from every object's holes at this
        // print z; an empty hole set or a travel clear of it routes
        // straight (`travel_to` :1557-1571, Liang-Barsky bbox gate).
        if boundary.external.is_none() {
            boundary.external = match boundary::Boundary::build_external(
                geometry.avoid_crossing.chunk_slices,
                geometry.avoid_crossing.chunk_perimeter_spacing,
                scale,
                [scaled_start, scaled_end],
            )
            .ok()?
            {
                boundary::BuildResult::Unavailable => None,
                // No holes anywhere in the chunk: every travel stays
                // outside the (empty) boundary bbox and routes straight.
                boundary::BuildResult::Empty => return Some(Vec::new()),
                boundary::BuildResult::Ready(external) => Some(external),
            };
        }
        boundary.external.as_ref()?
    } else {
        boundary.internal.as_ref()?
    };
    let (mut path, _intersections) =
        router::avoid_perimeters(boundary, scaled_start, scaled_end).ok()?;
    // Upstream restores the original endpoints before measuring and
    // emitting (`result_pl.points.front() = start`, `back() = end` at
    // `AvoidCrossingPerimeters.cpp:1261-1262` and `:1278-1279`).
    if let Some(first) = path.first_mut() {
        *first = scaled_start;
    }
    if let Some(last) = path.last_mut() {
        *last = scaled_end;
    }
    // `AvoidCrossingPerimeters.cpp:1291-1302`: when
    // `max_travel_detour_distance > 0` and the routed detour
    // (`result_pl.length() - direct_pl.length()`, scaled coord units)
    // exceeds the cap — `percent ? direct*0.01*value : scale_(value)` —
    // the route collapses to `{start, end}` and upstream also forces
    // `could_be_wipe_disabled = false`; that flag's only consumer
    // (`GCode.cpp:7435-7437`) is commented out, so the observable effect
    // is the straight travel.
    if detour_cap_exceeded(&path, scaled_start, scaled_end, max_detour, scale.factor()) {
        return Some(Vec::new());
    }
    let mut output = Vec::with_capacity(path.len());
    // Upstream restores the original endpoints after routing (which may nudge
    // them), then emits each interior point, even if formatting rounds two
    // distinct points to the same command. The caller appends the destination.
    // Polyline::append suppresses equal integer endpoints, yielding a singleton.
    let interior_count = path.len().saturating_sub(2);
    for point in path.into_iter().skip(1).take(interior_count) {
        output.push(arc::Point {
            x: scale.unscale(point.x()) + offset.0,
            y: scale.unscale(point.y()) + offset.1,
        });
    }
    Some(output)
}

/// Verbatim port of the `travel_to` detour cap
/// (`AvoidCrossingPerimeters.cpp:1291-1302`): computes in scaled coord
/// units like upstream (`Polyline::length`); the mm branch uses
/// `scale_(v)` = `v / factor` (`libslic3r.h:134-139`), percent uses
/// `direct_length * 0.01 * value` — including the upstream quirk that
/// percent treats the number itself as the percentage ("10%" -> 10).
fn detour_cap_exceeded(
    path: &[crate::geometry::Point],
    start: crate::geometry::Point,
    end: crate::geometry::Point,
    opt: FloatOrPercent,
    factor: f64,
) -> bool {
    let direct = hypot_between(start, end);
    let cap = match opt {
        FloatOrPercent::Percent(value) if value.0 > 0.0 => direct * 0.01 * value.0,
        FloatOrPercent::Float(value) if value > 0.0 => value / factor,
        _ => return false,
    };
    let routed = path
        .windows(2)
        .map(|points| hypot_between(points[0], points[1]))
        .sum::<f64>();
    routed - direct > cap
}

fn hypot_between(a: crate::geometry::Point, b: crate::geometry::Point) -> f64 {
    ((a.x() - b.x()) as f64).hypot((a.y() - b.y()) as f64)
}

fn detour_emission_ready() -> bool {
    true
}

pub(super) fn routing_active() -> bool {
    detour_emission_ready()
}
