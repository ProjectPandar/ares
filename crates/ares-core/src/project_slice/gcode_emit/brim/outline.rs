//! Brim.cpp:55-62,109-118,502-513: post-EFC bottom-region polygon consumer.

use crate::{
    ObjectOptions, ProcessBrimType, SliceError,
    geometry::{CoordinateScale, ExPolygon, JoinType, closing_ex, intersection_ex},
    project_slice::compensation::PostCompensationPrintObject,
};

pub(super) fn select(
    object: &PostCompensationPrintObject,
    options: &ObjectOptions,
    scale: CoordinateScale,
    ordinary: Vec<ExPolygon>,
) -> Result<Vec<ExPolygon>, SliceError> {
    if !matches!(
        options.brim_type,
        ProcessBrimType::OuterOnly | ProcessBrimType::InnerOnly | ProcessBrimType::OuterAndInner
    ) || !options.brim_use_efc_outline.0
        || options.elefant_foot_compensation.0 <= 0.0
        || options.elefant_foot_compensation_layers.0 <= 0
        || options.raft_layers.0 != 0
    {
        return Ok(ordinary);
    }
    let epsilon = super::scaled_f32(scale, 0.0001)?;
    let mut bottom = Vec::new();
    for region in 0..object.as_parts().0.regions.len() {
        bottom.extend(
            closing_ex(object.raw_slices(region, 0), epsilon, JoinType::Miter, 3.0)
                .map_err(super::brim_geometry_error)?,
        );
    }
    intersection_ex(&bottom, &ordinary).map_err(super::brim_geometry_error)
}
