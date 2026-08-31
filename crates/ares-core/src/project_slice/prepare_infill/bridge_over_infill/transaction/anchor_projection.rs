use crate::{
    ProcessInfillPattern, RegionOptions, SliceError,
    fill::cross_hatch::line_spacing,
    geometry::CoordinateScale,
    project_slice::{
        group_fills::simple_rotation_angle,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
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
            | ProcessInfillPattern::Gyroid
            | ProcessInfillPattern::ThreeDHoneycomb
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::ZigZag
    ) {
        return unsupported("sparse_infill_pattern");
    }
    if options.sparse_infill_pattern == ProcessInfillPattern::Gyroid && options.gyroid_optimized.0 {
        return unsupported("gyroid_optimized");
    }
    if options.sparse_infill_density.0 <= 0.0 {
        return unsupported("sparse_infill_density");
    }
    if !options.sparse_infill_rotate_template.0.is_empty()
        && simple_rotation_angle(&options.sparse_infill_rotate_template.0, 0).is_none()
    {
        return unsupported("sparse_infill_rotate_template");
    }
    if !options.solid_infill_rotate_template.0.is_empty()
        && simple_rotation_angle(&options.solid_infill_rotate_template.0, 0).is_none()
    {
        return unsupported("solid_infill_rotate_template");
    }
    if !matches!(
        options.top_surface_pattern,
        ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::Concentric
            | ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral
    ) {
        return unsupported("top_surface_pattern");
    }
    if !matches!(
        options.internal_solid_infill_pattern,
        ProcessInfillPattern::Monotonic
            | ProcessInfillPattern::MonotonicLine
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::Concentric
            | ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::HilbertCurve
            | ProcessInfillPattern::ArchimedeanChords
            | ProcessInfillPattern::OctagramSpiral
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
            RegionSurfaceKind::Bottom
                | RegionSurfaceKind::BottomBridge
                | RegionSurfaceKind::InternalBridge
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
