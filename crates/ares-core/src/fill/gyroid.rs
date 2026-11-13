//! Parametric Gyroid sparse infill rewrite from `FillGyroid.cpp:108-324`.

mod waves;

use waves::{
    contour_bounds, make_one_period, make_wave, polyline_length, rotate_expolygon,
    rotate_polylines, translate,
};

use super::connect::{FillConnectionParams, connect_infill};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polyline,
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
    /// `params.layer_height` (FillGyroid.cpp:319) — layer thickness for the
    /// `gyroid_optimized` omega factor.
    pub(crate) layer_height: f32,
    /// `params.gyroid_optimized` — selects the marching-squares implicit-field
    /// path over the parametric `make_gyroid_waves` path.
    pub(crate) gyroid_optimized: bool,
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
    minimum: f64,
    maximum: f64,
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
    let (minimum, maximum) = contour_bounds(rotated.contour());
    let density =
        (f64::from(params.density) * DENSITY_ADJUST / f64::from(params.multiline)).max(0.0);
    let distance = (params.spacing / scale.factor() / density) as i64;
    // `Point(double, double)` rounds the grid module; the phase shift is
    // scaled first, multiplied by ten, then truncated to coord_t.
    let period = (2.0 * std::f64::consts::PI * distance as f64).round() as i64;
    let aligned = Point::new(
        minimum.x().div_euclid(period) * period,
        minimum.y().div_euclid(period) * period,
    );
    let shift = (10.0 * (params.spacing / scale.factor())) as i64;
    let origin = Point::new(
        aligned
            .x()
            .checked_sub(shift)
            .ok_or(ClipperError::CoordinateOutOfRange)?,
        aligned
            .y()
            .checked_sub(shift)
            .ok_or(ClipperError::CoordinateOutOfRange)?,
    );
    let mut polylines = if params.gyroid_optimized {
        let maximum = Point::new(
            maximum
                .x()
                .checked_add(shift)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
            maximum
                .y()
                .checked_add(shift)
                .ok_or(ClipperError::CoordinateOutOfRange)?,
        );
        // `gyroid_optimized` (FillGyroid.cpp:314-333): marching-squares on the
        // gyroid implicit field. `omega` shortens the vertical wavelength for
        // buckling resistance; XY period stays at baseline.
        let lh = if params.layer_height > 0.0 {
            f64::from(params.layer_height)
        } else {
            params.spacing
        };
        let omega = super::gyroid_field::omega_factor(
            density,
            params.spacing * f64::from(params.multiline),
            lh,
        );
        // `density_factor = max(0.001f, density * DensityAdjust / multiline)`;
        // `period = float(2π) * spacing / density_factor`.
        let density_factor =
            (f64::from(params.density) * DENSITY_ADJUST / f64::from(params.multiline)).max(0.001);
        let period = 2.0_f32 * std::f32::consts::PI * params.spacing as f32 / density_factor as f32;
        let bb = BoundingBox::new(origin, maximum);
        // `GyroidField(bb, this->z, period, omega)` — `this->z` is the mm
        // height (FillGyroid.cpp:332); only the XY raster is scale-domain, the
        // Z frequency arg stays millimetres.
        let field =
            super::gyroid_field::GyroidField::new(bb, params.z, period, omega as f32, scale);
        super::gyroid_field::get_gyroid_polylines(&field, scale)
    } else {
        let bounds = BoundingBox::new(minimum, maximum)
            .inflated((params.spacing * f64::from(params.multiline) / scale.factor()) as i64);
        make_gyroid_waves(
            params.z / scale.factor(),
            density,
            params.spacing,
            (bounds, origin),
            scale,
        )?
    };
    polylines = super::multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;

    let (contour, holes) = rotated.clone().into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
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
    (bounds, origin): (BoundingBox, Point),
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let scale_factor = spacing / scale.factor() / density;
    let tolerance = (spacing / 2.0).min(PATTERN_TOLERANCE) / (scale_factor * scale.factor());
    let z = grid_z / scale_factor;
    let (z_sin, z_cos) = z.sin_cos();
    let context = WaveContext {
        z_sin,
        z_cos,
        vertical: z_sin.abs() <= z_cos.abs(),
        tolerance,
    };
    let mut lo = (
        (bounds.min().x() - origin.x()) as f64 / scale_factor,
        (bounds.min().y() - origin.y()) as f64 / scale_factor,
    );
    let mut hi = (
        (bounds.max().x() - origin.x()) as f64 / scale_factor,
        (bounds.max().y() - origin.y()) as f64 / scale_factor,
    );
    let (mut lower, mut flip) = (0.0, true);
    if context.vertical {
        flip = false;
        lower = -std::f64::consts::PI;
        std::mem::swap(&mut lo.0, &mut lo.1);
        std::mem::swap(&mut hi.0, &mut hi.1);
    }
    let odd = make_one_period(context, flip);
    let even = make_one_period(context, !flip);
    let (mut f_min, mut f_max) = (f64::MAX, f64::MIN);
    for point in odd.iter().chain(&even) {
        f_min = f_min.min(point.1);
        f_max = f_max.max(point.1);
    }
    let mut output = Vec::new();
    let mut index = ((lo.1 - f_max - lower) / std::f64::consts::PI).ceil() as i32;
    while lower + f64::from(index) * std::f64::consts::PI + f_min <= hi.1 {
        let mut wave = make_wave(
            if index % 2 == 0 { &odd } else { &even },
            WavePlacement {
                minimum: lo.0,
                maximum: hi.0,
                offset: lower + f64::from(index) * std::f64::consts::PI,
                scale_factor,
            },
            context.vertical,
        )?;
        translate(std::slice::from_mut(&mut wave), origin)?;
        output.push(wave);
        index += 1;
    }
    Ok(output)
}
