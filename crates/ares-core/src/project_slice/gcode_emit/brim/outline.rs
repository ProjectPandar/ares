//! Brim.cpp:55-62,109-118,468-472: post-EFC bottom-region polygon
//! consumer and painted-ear snap outline source.

use crate::{
    ObjectOptions, ProcessBrimType, SliceError,
    geometry::{CoordinateScale, ExPolygon, JoinType, closing_ex, intersection_ex},
    project_slice::compensation::PostCompensationPrintObject,
};

/// `use_brim_efc_outline` (Brim.cpp:56-62). The EFC base-slices swap of
/// `:468-472` is brim-type agnostic upstream; NoBrim remains deferred.
pub(super) fn efc_enabled(options: &ObjectOptions) -> bool {
    matches!(
        options.brim_type,
        ProcessBrimType::AutoBrim
            | ProcessBrimType::OuterOnly
            | ProcessBrimType::InnerOnly
            | ProcessBrimType::OuterAndInner
            | ProcessBrimType::BrimEars
            | ProcessBrimType::Painted
    ) && options.brim_use_efc_outline.0
        && options.elefant_foot_compensation.0 > 0.0
        && options.elefant_foot_compensation_layers.0 > 0
        && options.raft_layers.0 == 0
}

/// `get_print_object_bottom_layer_expolygons` (Brim.cpp:112-118):
/// bottom-layer region slices joined with a `SCALED_EPSILON` closing
/// (the EFC-adjusted outline, un-intersected). Computed once per object
/// under the gate and shared by `select` and the painted-ear snap
/// (upstream stores it lazily, Brim.cpp:364-367).
pub(super) fn bottom_regions(
    object: &PostCompensationPrintObject,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, SliceError> {
    let epsilon = super::scaled_f32(scale, 0.0001)?;
    let mut bottom = Vec::new();
    for region in 0..object.as_parts().0.regions.len() {
        bottom.extend(
            closing_ex(object.raw_slices(region, 0), epsilon, JoinType::Miter, 3.0)
                .map_err(super::brim_geometry_error)?,
        );
    }
    Ok(bottom)
}

pub(super) fn select(
    ordinary: Vec<ExPolygon>,
    bottom: Option<&[ExPolygon]>,
) -> Result<Vec<ExPolygon>, SliceError> {
    let Some(bottom) = bottom else {
        return Ok(ordinary);
    };
    intersection_ex(bottom, &ordinary).map_err(super::brim_geometry_error)
}
