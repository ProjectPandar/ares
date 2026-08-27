use crate::{
    ProcessInfillPattern, RegionOptions, SliceError,
    fill::cross_hatch::line_spacing,
    geometry::CoordinateScale,
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::sparse_anchoring::{projected_anchor_lengths, projected_sparse_density};

pub(super) fn validate(
    options: &RegionOptions,
    surfaces: &[RegionSurface],
    spacing: f32,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if !matches!(
        options.sparse_infill_pattern,
        ProcessInfillPattern::CrossHatch
            | ProcessInfillPattern::Grid
            | ProcessInfillPattern::Triangles
            | ProcessInfillPattern::Cubic
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::ZigZag
    ) {
        return unsupported("sparse_infill_pattern");
    }
    if options.sparse_infill_density.0 <= 0.0 {
        return unsupported("sparse_infill_density");
    }
    if options.top_surface_density.0 <= 0.0 {
        return unsupported("top_surface_density");
    }
    if !options.sparse_infill_rotate_template.0.is_empty() {
        return unsupported("sparse_infill_rotate_template");
    }
    if !options.solid_infill_rotate_template.0.is_empty() {
        return unsupported("solid_infill_rotate_template");
    }
    if options.align_infill_direction_to_model.0 {
        return unsupported("align_infill_direction_to_model");
    }
    if options.fill_multiline.0 != 1 {
        return unsupported("fill_multiline");
    }
    if !matches!(
        options.top_surface_pattern,
        ProcessInfillPattern::Monotonic | ProcessInfillPattern::MonotonicLine
    ) {
        return unsupported("top_surface_pattern");
    }
    if !matches!(
        options.internal_solid_infill_pattern,
        ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::Rectilinear
    ) {
        return unsupported("internal_solid_infill_pattern");
    }
    if options.sparse_infill_filament_id != options.internal_solid_filament_id
        || options.sparse_infill_filament_id != options.top_surface_filament_id
    {
        return unsupported("bridge_over_infill_anchor_extruder_order");
    }
    let (anchor, anchor_max) = projected_anchor_lengths(options, f64::from(spacing));
    if !anchor_max.is_finite() || anchor_max < 0.05 {
        return unsupported("infill_anchor_max");
    }
    if !anchor.is_finite() || anchor < 0.0 {
        return unsupported("infill_anchor");
    }
    let density = projected_sparse_density(options);
    if line_spacing(f64::from(spacing), density, 1, scale) <= 0 {
        return Err(SliceError::InvalidInput(
            "invalid Orca option sparse_infill_line_width".to_owned(),
        ));
    }
    if surfaces.iter().any(|surface| {
        !matches!(
            surface.as_parts().0,
            RegionSurfaceKind::BottomBridge
                | RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::Top
                | RegionSurfaceKind::Internal
        )
    }) {
        return unsupported("bridge_over_infill_anchor_surface_kind");
    }
    Ok(())
}

fn unsupported(key: &str) -> Result<(), SliceError> {
    Err(SliceError::UnsupportedProjectFeature(key.to_owned()))
}
