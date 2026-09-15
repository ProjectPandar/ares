//! Parametric Gyroid sparse infill rewrite from `FillGyroid.cpp:108-324`.

mod waves;

use waves::{
    contour_bounds, make_one_period, make_wave, polyline_length, rotate_expolygon,
    rotate_polylines, translate,
};

use super::{
    checked_rotate::{rotate_points, rotate_points_with_trig},
    connect::{FillConnectionParams, connect_infill},
};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Line, Point, Polygon, Polyline,
    intersection_open_polylines, offset_expolygon,
};

pub(super) const DENSITY_ADJUST: f64 = 2.44;
pub(super) const PATTERN_TOLERANCE: f64 = 0.2;
pub(super) const EPSILON: f64 = 1.0e-4;
const CORRECTION_ANGLE: f64 = -std::f64::consts::FRAC_PI_4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GyroidFillParams {
    pub(crate) z: f64,
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) multiline: i32,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

#[derive(Clone, Copy)]
pub(super) struct WaveContext {
    z_sin: f64,
    z_cos: f64,
    vertical: bool,
    tolerance: f64,
}

#[derive(Clone, Copy)]
pub(super) struct WavePlacement {
    width: f64,
    height: f64,
    offset: f64,
    scale_factor: f64,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: GyroidFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let offset = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components = offset_expolygon(surface, offset, JoinType::Miter, 3.0)?;
    let mut output = Vec::new();
    for component in components {
        output.extend(fill_component(&component, params, scale)?);
    }
    Ok(output)
}

fn fill_component(
    surface: &ExPolygon,
    params: GyroidFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // Upstream computes `float(angle + CorrectionAngle)` and skips both
    // rotations when `|infill_angle| < EPSILON` (FillGyroid.cpp:294-297,
    // 369-371); rotating by a near-zero residual shifts large plate
    // coordinates past integer rounding boundaries.
    let infill_angle = (f64::from(params.angle) + CORRECTION_ANGLE) as f32;
    let infill_angle = f64::from(infill_angle);
    let rotate = infill_angle.abs() >= EPSILON;
    let rotated = if rotate {
        rotate_expolygon(surface, -infill_angle)?
    } else {
        surface.clone()
    };
    let (mut minimum, mut maximum) = contour_bounds(rotated.contour());
    let density =
        (f64::from(params.density) * DENSITY_ADJUST / f64::from(params.multiline)).max(0.0);
    let distance = (params.spacing / scale.factor() / density) as i64;
    // Upstream: `bb.merge(align_to_grid(bb.min, Point(2*M_PI*distance,
    // 2*M_PI*distance)))` — the `Point(double, double)` ctor applies
    // `std::round` (`Point.hpp:197`), not truncation.
    let period = (2.0 * std::f64::consts::PI * distance as f64).round() as i64;
    minimum = Point::new(
        minimum.x().div_euclid(period) * period,
        minimum.y().div_euclid(period) * period,
    );
    // Upstream: `coord_t expand = 10 * (scale_(this->spacing))` — scale
    // first, then multiply, then truncate (FillGyroid.cpp:309).
    let expand = (10.0 * (params.spacing / scale.factor())) as i64;
    minimum = Point::new(
        minimum
            .x()
            .checked_sub(expand)
            .ok_or(ClipperError::CoordinateOutOfRange)?,
        minimum
            .y()
            .checked_sub(expand)
            .ok_or(ClipperError::CoordinateOutOfRange)?,
    );
    maximum = Point::new(
        maximum
            .x()
            .checked_add(expand)
            .ok_or(ClipperError::CoordinateOutOfRange)?,
        maximum
            .y()
            .checked_add(expand)
            .ok_or(ClipperError::CoordinateOutOfRange)?,
    );
    // Upstream `ceil(bb.size()(0) / distance) + 1.` divides coord_t by coord_t —
    // integer division (ceil is a no-op on the truncated quotient), so the
    // wave extent is floor(size/distance) + 1, not the float-ceil result.
    let width = ((maximum.x() - minimum.x()) / distance) as f64 + 1.0;
    let height = ((maximum.y() - minimum.y()) / distance) as f64 + 1.0;
    let mut polylines = make_gyroid_waves(
        params.z / scale.factor(),
        density,
        params.spacing,
        (width, height),
        scale,
    )?;
    translate(&mut polylines, minimum)?;
    polylines = super::multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;

    let (contour, holes) = rotated.clone().into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    if let Ok(path) = std::env::var("ARES_DUMP_GYROID_PRECLIP") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "PRECLIP n={}", polylines.len());
            for polyline in &polylines {
                let _ = write!(file, "PL {}:", polyline.points().len());
                for point in polyline.points() {
                    let _ = write!(
                        file,
                        " ({},{})",
                        scale.unscale(point.x()),
                        scale.unscale(point.y())
                    );
                }
                let _ = writeln!(file);
            }
            for polygon in &clip {
                let _ = write!(file, "CLIP_CONTOUR:");
                for point in polygon.points() {
                    let _ = write!(
                        file,
                        " ({},{})",
                        scale.unscale(point.x()),
                        scale.unscale(point.y())
                    );
                }
                let _ = writeln!(file);
            }
        }
    }
    polylines = intersection_open_polylines(&polylines, &clip)?;
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

impl WaveContext {
    fn value(self, x: f64, flip: bool) -> f64 {
        if self.vertical {
            let phase = if self.z_cos < 0.0 {
                2.0 * std::f64::consts::PI
            } else {
                std::f64::consts::PI
            };
            let a = (x + phase).sin();
            let b = -self.z_cos;
            let result =
                self.z_sin * (x + phase + if flip { std::f64::consts::PI } else { 0.0 }).cos();
            // Upstream f() computes r = sqrt(sqr(a)+sqr(b)) — NOT hypot (which
            // rounds differently); match the exact expression.
            let radius = (a * a + b * b).sqrt();
            (a / radius).asin() + (result / radius).asin() + std::f64::consts::PI
        } else {
            let phase = if self.z_sin < 0.0 {
                std::f64::consts::PI
            } else {
                0.0
            };
            let a = (x + phase).cos();
            let b = -self.z_sin;
            let result =
                self.z_cos * (x + phase + if flip { 0.0 } else { std::f64::consts::PI }).sin();
            let radius = (a * a + b * b).sqrt();
            (a / radius).asin() + (result / radius).asin() + std::f64::consts::FRAC_PI_2
        }
    }
}

fn make_gyroid_waves(
    grid_z: f64,
    density: f64,
    spacing: f64,
    (mut width, mut height): (f64, f64),
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let scale_factor = spacing / scale.factor() / density;
    let tolerance = (spacing / 2.0).min(PATTERN_TOLERANCE) / (scale_factor * scale.factor());
    if let Ok(path) = std::env::var("ARES_DUMP_GWAVE") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "GWAVE sf={scale_factor:.17e} tol={tolerance:.17e} z={grid_z:.17e} den={density:.17e} sp={spacing:.17e}"
            );
        }
    }
    let z = grid_z / scale_factor;
    let (z_sin, z_cos) = z.sin_cos();
    let context = WaveContext {
        z_sin,
        z_cos,
        vertical: z_sin.abs() <= z_cos.abs(),
        tolerance,
    };
    let (mut lower, mut upper, mut flip) = (0.0, height, true);
    if context.vertical {
        flip = false;
        lower = -std::f64::consts::PI;
        upper = width - std::f64::consts::FRAC_PI_2;
        std::mem::swap(&mut width, &mut height);
    }
    let odd = make_one_period(width, context, flip);
    flip = !flip;
    let even = make_one_period(width, context, flip);
    let mut output = Vec::new();
    let mut offset = lower;
    while offset < upper + EPSILON {
        output.push(make_wave(
            &odd,
            WavePlacement {
                width,
                height,
                offset,
                scale_factor,
            },
            context,
            flip,
        )?);
        offset += std::f64::consts::PI;
        if offset < upper + EPSILON {
            output.push(make_wave(
                &even,
                WavePlacement {
                    width,
                    height,
                    offset,
                    scale_factor,
                },
                context,
                flip,
            )?);
        }
        offset += std::f64::consts::PI;
    }
    Ok(output)
}
