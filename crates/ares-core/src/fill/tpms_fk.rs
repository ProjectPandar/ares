//! `FillTpmsFK::_fill_surface_single` (`src/libslic3r/Fill/FillTpmsFK.cpp:117`):
//! the Fischer–Koch S TPMS infill. A `ScalarField` evaluates the FK equation
//! on a `rsize`-pixel raster; `marchsq` marching-squares extracts the `isoval`
//! contour rings (window `gsize`); the rings become polylines, Douglas-Peucker
//! simplified, multiline-offset, clipped, and connected.

use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline, intersection_open_polylines,
};

use super::checked_rotate::{rotate_points, rotate_points_with_trig};
use super::connect::{FillConnectionParams, connect_infill};
use super::tpms_d::{boundary_polygons, contour_bounds};
use crate::geometry::marching_squares::{self, Coord};
use crate::geometry::simplification::douglas_peucker;

const CORRECTION_ANGLE: f64 = (-45.0_f64 * 2.0 * std::f64::consts::PI) / 360.0;
const EPSILON: f64 = 1.0e-4;
// `ScalarField` grid/raster sizes (FillTpmsFK.cpp:22-23).
const GSIZE_F: f64 = 0.40;
const RSIZE_F: f64 = 0.004;
/// `SPARSE_INFILL_RESOLUTION` (libslic3r.h:78) — 0.04mm; `SCALED_*` divides by
/// `SCALING_FACTOR` at use-site.
pub(super) const SPARSE_INFILL_RESOLUTION: f64 = 0.04;

pub(crate) struct TpmsFKFillParams {
    pub(crate) spacing: f64,
    pub(crate) density: f32,
    pub(crate) angle: f32,
    pub(crate) layer_z: f64,
    pub(crate) multiline: i32,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

/// `marchsq::ScalarField` (`FillTpmsFK.cpp:18-62`): the FK field over the
/// expolygon bounding box enlarged by the multiline width.
struct ScalarField {
    rsize: i64,
    gsize: i64,
    offs: Point,
    size: Point,
    z: f64,
    freq: f32,
    scale: CoordinateScale,
}

impl ScalarField {
    fn new(min: Point, size: Point, z: f64, period: f32, scale: CoordinateScale) -> Self {
        Self {
            rsize: scale.checked_scale(RSIZE_F).unwrap_or(1),
            gsize: (GSIZE_F / RSIZE_F).round() as i64,
            offs: min,
            size,
            z,
            freq: (2.0 * std::f32::consts::PI / period),
            scale,
        }
    }

    fn get_scalar(&self, x: f64, y: f64) -> f32 {
        // `const float fx = freq * x` (FillTpmsFK.cpp:37): `freq` (float) * `x`
        // (coordf_t=double) promotes to `double`, THEN narrows to `float` — not
        // `freq * (x as f32)` (which rounds the coordinate before multiplying).
        let fx = (self.freq as f64 * x) as f32;
        let fy = (self.freq as f64 * y) as f32;
        let fz = (self.freq as f64 * self.z) as f32;
        // Fischer–Koch S: cos(2x)sin(y)cos(z) + cos(2y)sin(z)cos(x) +
        // cos(2z)sin(x)cos(y) = 0 (FillTpmsFK.cpp:43).
        (2.0 * fx).cos() * fy.sin() * fz.cos()
            + (2.0 * fy).cos() * fz.sin() * fx.cos()
            + (2.0 * fz).cos() * fx.sin() * fy.cos()
    }

    // `to_Point` = raster Coord → scaled Point; `to_Pointf` → unscaled mm.
    fn to_point(&self, c: Coord) -> Point {
        Point::new(
            c.c * self.rsize + self.offs.x(),
            c.r * self.rsize + self.offs.y(),
        )
    }

    fn to_pointf(&self, c: Coord) -> (f64, f64) {
        let p = self.to_point(c);
        (self.scale.unscale(p.x()), self.scale.unscale(p.y()))
    }

    fn value_at(&self, row: i64, col: i64) -> f32 {
        let (x, y) = self.to_pointf(Coord { r: row, c: col });
        self.get_scalar(x, y)
    }

    fn rows(&self) -> i64 {
        self.size.y() / self.rsize
    }
    fn cols(&self) -> i64 {
        self.size.x() / self.rsize
    }
}

pub(crate) fn fill_tpmsfk_surface(
    expolygon: &ExPolygon,
    params: &TpmsFKFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // `float(this->angle + (CorrectionAngle * 2 * M_PI) / 360.)` — computed in
    // double, then narrowed to `float` before the rotate calls (FillTpmsFK.cpp:123).
    let infill_angle = (f64::from(params.angle) + CORRECTION_ANGLE) as f32 as f64;
    let rotate = infill_angle.abs() >= EPSILON;
    let rotated = if rotate {
        rotate_expolygon(expolygon, -infill_angle)?
    } else {
        expolygon.clone()
    };

    // `std::min(0.9f, params.density)` then `4.18f * spacing * params.multiline
    // / density_factor` (FillTpmsFK.cpp:128-129): `4.18f`(f32) * `spacing`(coordf_t=
    // f64) → f64, / `density_factor`(f32→f64) → f64, assigned to `float vari_T`.
    let density_factor = f64::from(params.density.min(0.9));
    let vari_t = 4.18_f32 as f64 * params.spacing * f64::from(params.multiline) / density_factor;

    let (mut min, mut max) = contour_bounds(rotated.contour());
    // Enlarge the box by `(multiline+1)*spacing` (FillTpmsFK.cpp:131).
    let pad = scale
        .checked_scale((f64::from(params.multiline) + 1.0) * params.spacing)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    min = Point::new(min.x() - pad, min.y() - pad);
    max = Point::new(max.x() + pad, max.y() + pad);
    let size = Point::new(max.x() - min.x(), max.y() - min.y());

    let field = ScalarField::new(min, size, params.layer_z, vari_t as f32, scale);
    let rows = field.rows();
    let cols = field.cols();
    let gsize = field.gsize;
    let rings = marching_squares::execute(
        |r, c| field.value_at(r, c),
        rows,
        cols,
        0.0,
        Coord { r: gsize, c: gsize },
    );

    let mut polylines: Vec<Polyline> = rings
        .into_iter()
        .map(|ring| {
            let mut pts: Vec<Point> = ring.iter().map(|&c| field.to_point(c)).collect();
            if let Some(&first) = pts.first() {
                pts.push(first);
            }
            // `poly.simplify(tolerance)` — open Douglas-Peucker.
            Polyline::new(douglas_peucker(
                &pts,
                SPARSE_INFILL_RESOLUTION / scale.factor(),
            ))
        })
        .collect();

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
    polylines.retain(|p| polyline_length(p) >= minimum_length);
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
