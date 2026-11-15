//! `FillTpmsD::_fill_surface_single` (`src/libslic3r/Fill/FillTpmsD.cpp:101`):
//! the TPMS-D (diamond) implicit-surface infill. `make_waves` builds one
//! analytic wave `acos(a/b·cos(u))` in `(u = x−y, v = x+y)` space, tiles it
//! over the bounding box, then the polylines are translated to the grid
//! origin, multiline-offset, clipped, and connected.
//!
//! The surface `cos(x−y)·a − cos(x+y)·b = 0` with `a = sin z − cos z`,
//! `b = sin z + cos z` is parameterized so `u = acos(b/a·cos v)` is continuous
//! (swapping `u`/`v` when `|b| ≥ |a|`).

use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline, intersection_open_polylines,
};

use super::checked_rotate::{rotate_points, rotate_points_with_trig};
use super::connect::{FillConnectionParams, connect_infill};

const CORRECTION_ANGLE: f64 = (-45.0_f64 * 2.0 * std::f64::consts::PI) / 360.0;
const DENSITY_ADJUST: f64 = 2.1;
const PATTERN_TOLERANCE: f64 = 0.1;
const EPSILON: f64 = 1.0e-4;

pub(crate) struct TpmsDFillParams {
    pub(crate) spacing: f64,
    pub(crate) density: f32,
    pub(crate) angle: f32,
    pub(crate) layer_z: f64,
    pub(crate) multiline: i32,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

pub(crate) fn fill_tpmsd_surface(
    expolygon: &ExPolygon,
    params: &TpmsDFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // FillTpmsD.cpp:108-110,155-160: narrow the corrected angle to float
    // before both rotations and the EPSILON check.
    let infill_angle = f64::from((f64::from(params.angle) + CORRECTION_ANGLE) as f32);
    let rotate = infill_angle.abs() >= EPSILON;
    let rotated = if rotate {
        rotate_expolygon(expolygon, -infill_angle)?
    } else {
        expolygon.clone()
    };
    let (mut min, mut max) = contour_bounds(rotated.contour());

    let density_adjusted =
        (f64::from(params.density) * DENSITY_ADJUST / f64::from(params.multiline)).max(0.0);
    let distance = (params.spacing / scale.factor() / density_adjusted) as i64;

    // `bb.merge(align_to_grid(bb.min, Point(module, module)))` — extend the
    // box to include the grid-aligned minimum (per-axis min/max merge).
    // `Point(2*M_PI*distance, ...)` rounds (`Point(double,double)` ctor is
    // `coord_t(std::round(x))`), not `as i64` truncation.
    let module = (2.0 * std::f64::consts::PI * distance as f64).round() as i64;
    let aligned = align_to_grid(min, module);
    min = Point::new(min.x().min(aligned.x()), min.y().min(aligned.y()));
    max = Point::new(max.x().max(aligned.x()), max.y().max(aligned.y()));

    // `make_waves(scale_(this->z), ...)` — `scale_` is `z/SCALING_FACTOR`
    // (double, no truncation; `libslic3r.h:94`), so keep the fraction.
    let grid_z = scale.scaled_delta(params.layer_z);
    let mut polylines = make_waves(
        grid_z,
        density_adjusted,
        params.spacing,
        // Upstream `ceil(bb.size()(0) / distance) + 1.` divides coord_t by
        // coord_t — integer division, `ceil` a no-op → `size/distance + 1`
        // (floor), not float-ceil (FillTpmsD.cpp:120-121).
        [
            ((max.x() - min.x()) / distance + 1) as f64,
            ((max.y() - min.y()) / distance + 1) as f64,
        ],
        scale,
    )?;
    translate(&mut polylines, min)?;

    polylines = super::multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;
    let clip = boundary_polygons(&rotated);
    polylines = intersection_open_polylines(&polylines, &clip)?;
    polylines = super::join_close_ends::join_close_ends(
        polylines,
        EPSILON / scale.factor(),
        true,
        EPSILON / scale.factor(),
    );
    let minimum_length = 0.8 * params.spacing / scale.factor();
    polylines.retain(|polyline| polyline_length(polyline) >= minimum_length);
    if polylines.is_empty() {
        return Ok(Vec::new());
    }
    let mut connected = connect_infill(
        polylines,
        &rotated,
        params.spacing,
        FillConnectionParams {
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            multiline: params.multiline,
            dont_sort: params.dont_sort,
        },
        scale,
    )?;
    if rotate {
        rotate_polylines(&mut connected, infill_angle)?;
    }
    Ok(connected)
}

fn make_waves(
    grid_z: f64,
    density_adjusted: f64,
    line_spacing: f64,
    [width, height]: [f64; 2],
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // `scaleFactor = scale_(line_spacing)/density_adjusted` — the scaled
    // (micron) wave period. `grid_z` arrives already scaled (`scale_(z)`).
    let scale_factor = scale.scaled_delta(line_spacing) / density_adjusted;
    // `tolerance = min(line_spacing/2, PatternTolerance) / unscale(scaleFactor)`
    // (`FillTpmsD.cpp:27`) — `unscale(scaleFactor) = scaleFactor * SCALING_FACTOR`,
    // so the denominator is `scale_factor * scale.factor()` (multiply, not divide).
    let tolerance = (line_spacing / 2.0).min(PATTERN_TOLERANCE) / (scale_factor * scale.factor());
    let z = grid_z / scale_factor;
    let a = z.sin() - z.cos();
    let b = z.sin() + z.cos();
    let swap_uv = a.abs() > b.abs();
    // `minU = 0 - height; maxU = width; minV = 0; maxV = width + height`,
    // swapped alongside a/b when `swapUV` (FillTpmsD.cpp:50-58).
    let (a, b, min_u, max_u, min_v, max_v) = if swap_uv {
        (b, a, 0.0, width + height, -height, width)
    } else {
        (a, b, -height, width, 0.0, width + height)
    };
    let v_of = |u: f64| (a / b * u.cos()).acos();
    let mut wave: Vec<(f64, f64)> = Vec::new();
    for c in 0..=16 {
        let u = min_u + 2.0 * std::f64::consts::PI * c as f64 / 16.0;
        wave.push((u, v_of(u)));
    }
    let mut current = 0;
    while current + 1 < wave.len() {
        let (u1, v1) = wave[current];
        let (u2, v2) = wave[current + 1];
        let mid_u = (u1 + u2) / 2.0;
        let mid_v = v_of(mid_u);
        if (mid_v - (v1 + v2) / 2.0).abs() > tolerance {
            wave.insert(current + 1, (mid_u, mid_v));
        } else {
            current += 1;
        }
    }
    let mut c = 1;
    while c < wave.len() && wave.last().unwrap().0 < max_u {
        wave.push((wave[c].0 + 2.0 * std::f64::consts::PI, wave[c].1));
        c += 1;
    }
    let mut result = Vec::new();
    let mut v_shift = scaled_floor(min_v, 2.0 * std::f64::consts::PI);
    while v_shift < max_v + 2.0 * std::f64::consts::PI {
        for forward_root in [false, true] {
            let points = wave_points(&wave, v_shift, forward_root, swap_uv, scale_factor);
            result.push(Polyline::new(points));
        }
        v_shift += 2.0 * std::f64::consts::PI;
    }
    Ok(result)
}

fn wave_points(
    wave: &[(f64, f64)],
    v_shift: f64,
    forward_root: bool,
    swap_uv: bool,
    scale_factor: f64,
) -> Vec<Point> {
    let mut points = Vec::new();
    for &(u, vv) in wave {
        let v = (if forward_root { vv } else { -vv }) + v_shift;
        let x = (u + v) / 2.0;
        let y = (v - u) / 2.0 * if swap_uv { -1.0 } else { 1.0 };
        points.push(Point::new(
            (x * scale_factor).round() as i64,
            (y * scale_factor).round() as i64,
        ));
    }
    points
}

fn scaled_floor(x: f64, scale: f64) -> f64 {
    (x / scale).floor() * scale
}

pub(crate) fn contour_bounds(contour: &Polygon) -> (Point, Point) {
    let mut min = Point::new(i64::MAX, i64::MAX);
    let mut max = Point::new(i64::MIN, i64::MIN);
    for p in contour.points() {
        min = Point::new(min.x().min(p.x()), min.y().min(p.y()));
        max = Point::new(max.x().max(p.x()), max.y().max(p.y()));
    }
    (min, max)
}

fn align_to_grid(point: Point, module: i64) -> Point {
    let align = |c: i64| -> i64 { c.div_euclid(module) * module };
    Point::new(align(point.x()), align(point.y()))
}

pub(crate) fn boundary_polygons(expolygon: &ExPolygon) -> Vec<Polygon> {
    std::iter::once(expolygon.contour().clone())
        .chain(expolygon.holes().iter().cloned())
        .collect()
}

fn translate(polylines: &mut [Polyline], offset: Point) -> Result<(), ClipperError> {
    for polyline in polylines {
        let points = polyline
            .points()
            .iter()
            .map(|p| {
                Ok(Point::new(
                    p.x()
                        .checked_add(offset.x())
                        .ok_or(ClipperError::CoordinateOutOfRange)?,
                    p.y()
                        .checked_add(offset.y())
                        .ok_or(ClipperError::CoordinateOutOfRange)?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        *polyline = Polyline::new(points);
    }
    Ok(())
}

fn polyline_length(polyline: &Polyline) -> f64 {
    polyline
        .points()
        .windows(2)
        .map(|w| {
            let dx = (w[1].x() - w[0].x()) as f64;
            let dy = (w[1].y() - w[0].y()) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

fn rotate_expolygon(expolygon: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    let rotate =
        |polygon: &Polygon| rotate_points(polygon.points().to_vec(), angle).map(Polygon::new);
    Ok(ExPolygon::new(
        rotate(expolygon.contour())?,
        expolygon
            .holes()
            .iter()
            .map(rotate)
            .collect::<Result<_, _>>()?,
    ))
}

fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let (cosine, sine) = (angle.cos(), angle.sin());
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}
