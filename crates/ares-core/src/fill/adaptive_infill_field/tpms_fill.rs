//! Continuous adaptive TPMS component fill dispatch for `FillGyroid`
//! (`FillGyroid.cpp:25-28,345-349`), `FillTpmsD` (`FillTpmsD.cpp:28-31,133-139`),
//! and `FillTpmsFK` (`FillTpmsFK.cpp:27-30,148-157`).

use crate::{
    ProcessAdaptiveInfillGradient,
    fill::{
        checked_rotate::{rotate_points, rotate_points_with_trig},
        connect::{FillConnectionParams, connect_infill},
        join_close_ends::join_close_ends,
        multiline_offset,
    },
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, Polyline,
        intersection_open_polylines, offset_expolygon,
    },
    scalar_math::sincosf,
};

use super::{
    AdaptiveInfillField, AdaptiveTpms,
    tpms::{make_adaptive_tpms, polyline_length},
};

const EPSILON: f64 = 1.0e-4;
const GYROID_DENSITY_ADJUST: f64 = 2.44;
const TPMSD_DENSITY_ADJUST: f64 = 2.1;
const CORRECTION_ANGLE: f64 = (-45.0_f64 * 2.0 * std::f64::consts::PI) / 360.0;
const PI: f64 = std::f64::consts::PI;

#[derive(Clone, Copy)]
pub(crate) enum AdaptiveTpmsPattern {
    Gyroid,
    TpmsD,
    TpmsFk,
}

impl AdaptiveTpmsPattern {
    fn equation(self) -> fn(f32, f32, f32) -> f32 {
        match self {
            Self::Gyroid => gyroid_equation,
            Self::TpmsD => schwarz_d_equation,
            Self::TpmsFk => fischer_koch_equation,
        }
    }

    fn frequency(self, density: f32, spacing: f64, multiline: i32) -> f64 {
        match self {
            Self::Gyroid => {
                f64::from(density) * GYROID_DENSITY_ADJUST / (f64::from(multiline) * spacing)
            }
            Self::TpmsD => {
                f64::from(density) * TPMSD_DENSITY_ADJUST / (f64::from(multiline) * spacing)
            }
            Self::TpmsFk => {
                let period = f64::from(4.18_f32) * spacing * f64::from(multiline)
                    / f64::from(density.min(0.9_f32));
                2.0 * PI / period
            }
        }
    }

    fn bbox_pad(self, spacing: f64, multiline: i32) -> f64 {
        match self {
            Self::Gyroid => spacing * f64::from(multiline),
            Self::TpmsD | Self::TpmsFk => f64::from(multiline + 1) * spacing,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AdaptiveTpmsFillParams {
    pub(crate) z: f64,
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) interior_density: f32,
    pub(crate) gradient: ProcessAdaptiveInfillGradient,
    pub(crate) multiline: i32,
    pub(crate) layer_height: f64,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

/// `FillGyroid.cpp:25-28`: implicit gyroid equation via deterministic binary32 trig.
fn gyroid_equation(x: f32, y: f32, z: f32) -> f32 {
    let (sin_x, cos_x) = sincosf::sin_cos(x);
    let (sin_y, cos_y) = sincosf::sin_cos(y);
    let (sin_z, cos_z) = sincosf::sin_cos(z);
    sin_x * cos_y + sin_y * cos_z + sin_z * cos_x
}

/// `FillTpmsD.cpp:28-31`: Schwarz-D equation via deterministic binary32 trig.
fn schwarz_d_equation(x: f32, y: f32, z: f32) -> f32 {
    let (sin_x, cos_x) = sincosf::sin_cos(x);
    let (sin_y, cos_y) = sincosf::sin_cos(y);
    let (sin_z, cos_z) = sincosf::sin_cos(z);
    sin_x * sin_y * sin_z - cos_x * cos_y * cos_z
}

/// `FillTpmsFK.cpp:27-30`: Fischer-Koch S equation via deterministic binary32 trig.
fn fischer_koch_equation(x: f32, y: f32, z: f32) -> f32 {
    let (sin_x, cos_x) = sincosf::sin_cos(x);
    let (sin_y, cos_y) = sincosf::sin_cos(y);
    let (sin_z, cos_z) = sincosf::sin_cos(z);
    sincosf::cos(2.0 * x) * sin_y * cos_z
        + sincosf::cos(2.0 * y) * sin_z * cos_x
        + sincosf::cos(2.0 * z) * sin_x * cos_y
}

pub(crate) fn fill_adaptive_tpms(
    surface: &ExPolygon,
    pattern: AdaptiveTpmsPattern,
    params: AdaptiveTpmsFillParams,
    field: &AdaptiveInfillField,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let offset = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components = offset_expolygon(surface, offset, JoinType::Miter, 3.0)?;
    let mut output = Vec::new();
    for component in components {
        output.extend(fill_adaptive_tpms_component(
            &component, pattern, params, field, scale,
        )?);
    }
    Ok(output)
}

pub(crate) fn fill_adaptive_tpms_component(
    surface: &ExPolygon,
    pattern: AdaptiveTpmsPattern,
    params: AdaptiveTpmsFillParams,
    field: &AdaptiveInfillField,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let infill_angle_f32 = (f64::from(params.angle) + CORRECTION_ANGLE) as f32;
    let infill_angle = f64::from(infill_angle_f32);
    let rotate = infill_angle.abs() >= EPSILON;
    let rotated = if rotate {
        rotate_expolygon(surface, -infill_angle)?
    } else {
        surface.clone()
    };
    let Some(mut bb) = BoundingBox::from_polygon(rotated.contour()) else {
        return Ok(Vec::new());
    };
    bb.offset((pattern.bbox_pad(params.spacing, params.multiline) / scale.factor()) as i64);

    let tpms = AdaptiveTpms {
        equation: pattern.equation(),
        surface_frequency: pattern.frequency(params.density, params.spacing, params.multiline),
        interior_frequency: pattern.frequency(
            params.interior_density,
            params.spacing,
            params.multiline,
        ),
        gradient: params.gradient,
    };
    let mut polylines = make_adaptive_tpms(
        tpms,
        field,
        bb,
        (
            params.z,
            params.layer_height,
            params.spacing,
            infill_angle_f32,
        ),
        scale,
    );
    polylines = multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;

    let (contour, holes) = rotated.clone().into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    polylines = intersection_open_polylines(&polylines, &clip)?;
    polylines = join_close_ends(
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
