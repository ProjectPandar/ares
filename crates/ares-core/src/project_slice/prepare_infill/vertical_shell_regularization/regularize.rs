use crate::{
    SliceError,
    geometry::{FillRule, JoinType, offset_expolygons, offset2_ex, union_ex},
    project_slice::prepare_infill::{
        vertical_shell_regularization::range_error,
        vertical_shell_trimming::types::VerticalShellTrim,
    },
};

use super::types::VerticalShellRegularization;

const DEFAULT_MITER_LIMIT: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Radii {
    pub(super) narrow_ensure: f32,
    pub(super) narrow_sparse: f32,
    pub(super) tiny_overlap: f32,
}

pub(super) fn min_perimeter_infill_spacing(solid_infill_spacing: i64) -> f32 {
    (solid_infill_spacing as f32) * 1.05_f32
}

pub(super) fn radii(solid_infill_spacing: i64) -> Radii {
    let min_perimeter_infill_spacing = min_perimeter_infill_spacing(solid_infill_spacing);
    Radii {
        narrow_ensure: 0.5_f32 * 0.65_f32 * min_perimeter_infill_spacing,
        narrow_sparse: 0.5_f32 * 1.2_f32 * min_perimeter_infill_spacing,
        tiny_overlap: 0.2_f32 * min_perimeter_infill_spacing,
    }
}

pub(super) fn regularize_record(
    trim: &VerticalShellTrim,
    solid_infill_spacing: i64,
) -> Result<VerticalShellRegularization, SliceError> {
    if trim.shell.is_empty() {
        return Ok(VerticalShellRegularization {
            regularized_shell: Vec::new(),
        });
    }

    let radii = radii(solid_infill_spacing);
    let united = union_ex(&trim.shell, FillRule::NonZero).map_err(|_| range_error())?;

    let opened_and_closed = offset2_ex(
        &united,
        -radii.narrow_ensure,
        radii.narrow_ensure + radii.narrow_sparse,
        JoinType::Square,
        DEFAULT_MITER_LIMIT,
    )
    .map_err(|_| range_error())?;

    let regularized_shell = offset_expolygons(
        &opened_and_closed,
        -(radii.narrow_sparse - radii.tiny_overlap),
        JoinType::Square,
        DEFAULT_MITER_LIMIT,
    )
    .map_err(|_| range_error())?;

    Ok(VerticalShellRegularization { regularized_shell })
}
