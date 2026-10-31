//! Reduce-crossing-wall travel routing — a source-cited port of
//! `AvoidCrossingPerimeters::travel_to` (`GCode/AvoidCrossingPerimeters.cpp:
//! 1233-1312`) over the layer boundary.

mod boundary;
mod router;
mod safe_zone;

use crate::{
    SliceError, geometry::Point, options::FloatOrPercent, project_slice::gcode_emit::motion::arc,
};

/// `init_layer`'s safe zone and lazily initialized `m_internal` boundary.
#[derive(Clone)]
pub(in crate::project_slice::gcode_emit) struct Boundary {
    safe_zone: Vec<crate::geometry::ExPolygon>,
    internal: Option<boundary::Boundary>,
    external: Option<boundary::Boundary>,
}
use super::super::state::LayerGeometry;

pub(super) struct Request<'a> {
    pub(super) start: arc::Point,
    pub(super) end: arc::Point,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) offset: (f64, f64),
    /// `gcodegen.origin()` — the planning origin `set_origin(unscale(
    /// instance.shift))` maintains (`GCode.cpp:5385`), i.e. `state.origin`;
    /// the emission `offset` minus the extruder offset. Only the external
    /// planner consumes it (`AvoidCrossingPerimeters.cpp:1237-1240`).
    pub(super) origin: (f64, f64),
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
) -> Result<Boundary, crate::geometry::ClipperError> {
    Ok(Boundary {
        safe_zone: safe_zone::build(&geometry.avoid_crossing, geometry.scale)?,
        internal: None,
        external: None,
    })
}

/// Route `start`→`end` along the boundary contours, mirroring
/// `AvoidCrossingPerimeters::travel_to`. Rebuild the cached internal boundary
/// when either endpoint leaves its original (pre-EdgeGrid expansion) bounds.
/// Return interior waypoints in G-code millimetres; the caller owns endpoints.
/// A `BuildResult::Unavailable` internal/external boundary maps to upstream's
/// `{start, end}` straight travel (`AvoidCrossingPerimeters.cpp:1259-1264`
/// and `:1284-1288` — the `!boundaries.empty()` gate fails and `result_pl`
/// stays the `{start, end}` default); Clipper/overflow failures propagate.
pub(super) fn route(
    request: Request<'_>,
    boundary: &mut Boundary,
) -> Result<Vec<arc::Point>, SliceError> {
    let Request {
        start,
        end,
        geometry,
        offset,
        origin,
        use_external,
        max_detour,
    } = request;
    let scale = geometry.scale;
    // The mm cursor round-trips through `unscale(int) + offset`, which lands
    // a few ULPs below the exact value; a truncating cast would drop one
    // lattice unit and skew every route waypoint (qVMcEP X3946461 vs
    // upstream's exact X3946462). Upstream routes on `m_last_pos` ints, so
    // round the conversion back to the originating integer.
    let to_scaled = |point: arc::Point| -> Result<crate::geometry::Point, SliceError> {
        Ok(crate::geometry::Point::new(
            scale
                .checked_scale_rounded(point.x - offset.0)
                .ok_or_else(|| {
                    SliceError::InvalidInput("avoid-crossing endpoint out of range".into())
                })?,
            scale
                .checked_scale_rounded(point.y - offset.1)
                .ok_or_else(|| {
                    SliceError::InvalidInput("avoid-crossing endpoint out of range".into())
                })?,
        ))
    };
    let scaled_start = to_scaled(start)?;
    let scaled_end = to_scaled(end)?;
    // `Point::new_scale(gcodegen.origin())` — the use_external planning
    // frame's `scaled_origin` (`AvoidCrossingPerimeters.cpp:1237-1240`).
    // The plate-frame external holes move into this frame by
    // `-scaled_origin`; deriving the shift from the emission `offset`
    // instead displaced the boundary by the extruder offset and changed
    // which boundary lines the travel reaches. `checked_scale` truncates
    // exactly like upstream `coord_t(scale_(v))`.
    let scaled_origin = Point::new(
        scale.checked_scale(origin.0).ok_or_else(|| {
            SliceError::InvalidInput("avoid-crossing endpoint out of range".into())
        })?,
        scale.checked_scale(origin.1).ok_or_else(|| {
            SliceError::InvalidInput("avoid-crossing endpoint out of range".into())
        })?,
    );
    // Travels fully inside the lslices safe zone never route
    // (`any_expolygon_contains(m_lslices_offset, ...)`,
    // `AvoidCrossingPerimeters.cpp:1255`).
    if boundary.safe_zone.is_empty()
        || boundary
            .safe_zone
            .iter()
            .any(|polygon| safe_zone::contains(scaled_start, scaled_end, polygon))
    {
        return Ok(Vec::new());
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
        boundary.internal = match built {
            Err(error) => {
                return Err(SliceError::InvalidInput(format!(
                    "avoid-crossing internal boundary: {error:?}"
                )));
            }
            // `travel_to` (:1259–1264, :1285–1288) skips an empty internal
            // boundary and returns {start, end}.
            Ok(boundary::BuildResult::Unavailable) | Ok(boundary::BuildResult::Empty) => {
                boundary.internal = None;
                return Ok(Vec::new());
            }
            Ok(boundary::BuildResult::Ready(internal)) => Some(internal),
        };
    }
    let boundary = if use_external {
        // `get_boundary_external` builds from every object's holes at this
        // print z; an empty hole set or a travel clear of it routes
        // straight (`travel_to` :1557-1571, Liang-Barsky bbox gate).
        //
        // `travel_to` :1262-1268: rebuild when the external boundary is
        // empty or either endpoint left its (radius-padded) bbox.
        if boundary.external.as_ref().is_none_or(|external| {
            !external.contains(scaled_start) || !external.contains(scaled_end)
        }) {
            // The boundary input is plate-frame (`:1172-1177`); the router
            // frame is `last_pos + scaled_origin`, so shift the holes by
            // `-scaled_origin` (`:1237-1240`).
            let frame_shift = Point::new(-scaled_origin.x(), -scaled_origin.y());
            boundary.external = match boundary::Boundary::build_external(
                geometry.avoid_crossing.chunk_slices,
                geometry.avoid_crossing.chunk_perimeter_spacing,
                scale,
                [scaled_start, scaled_end],
                frame_shift,
            ) {
                Err(error) => {
                    return Err(SliceError::InvalidInput(format!(
                        "avoid-crossing external boundary: {error:?}"
                    )));
                }
                // No holes anywhere in the chunk — or no boundary at all
                // (`:1273-1288`): every travel stays outside the (empty)
                // boundary bbox and routes straight.
                Ok(boundary::BuildResult::Unavailable) | Ok(boundary::BuildResult::Empty) => {
                    return Ok(Vec::new());
                }
                Ok(boundary::BuildResult::Ready(external)) => Some(external),
            };
        }
        boundary.external.as_ref().expect("built above")
    } else {
        boundary.internal.as_ref().expect("built above")
    };
    let (mut path, _intersections) = router::avoid_perimeters(boundary, scaled_start, scaled_end)
        .map_err(|error| {
        SliceError::InvalidInput(format!("avoid-crossing routing: {error:?}"))
    })?;
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
        return Ok(Vec::new());
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
    Ok(output)
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
