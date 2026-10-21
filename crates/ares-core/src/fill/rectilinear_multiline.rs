//! FillRectilinear.cpp:2996-3045,3386-3399: sparse rectilinear multiline sweep.

use crate::{
    ProcessInfillPattern,
    geometry::{ClipperError, CoordinateScale, ExPolygon},
};

use super::{
    multiline::{MultilineFillParams, Sweep},
    rectilinear::{MonotonicFillOutput, MonotonicFillParams},
};

pub(crate) fn enabled(pattern: ProcessInfillPattern, multiline: i32, density: f32) -> bool {
    // FillBase.hpp:50 and FillRectilinear.cpp:3391 retain the by-lines full branch.
    let full_infill = density > 0.9999;
    // FillRectilinear.hpp:38-46: Aligned inherits this dispatch, fixing only its angle.
    matches!(
        pattern,
        ProcessInfillPattern::Rectilinear | ProcessInfillPattern::AlignedRectilinear
    ) && multiline > 1
        && !full_infill
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: MonotonicFillParams,
    multiline: i32,
    anchor_length: f32,
    scale: CoordinateScale,
) -> Result<MonotonicFillOutput, ClipperError> {
    // FillBase.cpp:275-319: preserve f32 additions and effective layer/thickness.
    let mut angle = params.bridge_angle.unwrap_or(params.angle);
    if params.bridge_angle.is_none()
        && !params.fixed_angle
        && !params.layer_angle_fixed
        && !params.dont_alternate_fill_direction
        && (params.layer_index / usize::from(params.thickness_layers)) & 1 == 1
    {
        angle += std::f32::consts::FRAC_PI_2;
    }
    angle += std::f32::consts::FRAC_PI_2;
    let polylines = super::multiline::fill_surface(
        surface,
        MultilineFillParams {
            spacing: params.spacing,
            overlap: params.overlap,
            angle,
            reference: params.reference_point,
            density: params.density,
            multiline,
            anchor_length,
            anchor_length_max: params.anchor_length_max,
            dont_sort: false,
        },
        &[Sweep {
            angle: 0.0,
            shift: 0.0,
        }],
        scale,
    )?;
    // FillRectilinear.cpp:2996-3045 does not adjust the generator's spacing.
    Ok(MonotonicFillOutput {
        polylines,
        spacing: params.spacing as f32,
    })
}
