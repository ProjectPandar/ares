//! `Fill.cpp::generate_sparse_infill_polylines_for_anchoring` plane-path handoff.

use crate::{
    ProcessInfillPattern, SliceError,
    geometry::Polyline,
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    },
};

pub(super) fn append(
    fill: SurfaceFill,
    pattern: ProcessInfillPattern,
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    let plane_path_pattern = match pattern {
        ProcessInfillPattern::HilbertCurve => {
            crate::fill::plane_path::PlanePathPattern::HilbertCurve
        }
        ProcessInfillPattern::ArchimedeanChords => {
            crate::fill::plane_path::PlanePathPattern::ArchimedeanChords
        }
        ProcessInfillPattern::OctagramSpiral => {
            crate::fill::plane_path::PlanePathPattern::OctagramSpiral
        }
        _ => unreachable!("plane-path arm only admits its three patterns"),
    };
    let (object, print_object) = traversal.print_object_config(object_index);
    let object_bounding_box = crate::project_slice::fill_entities::print_object_bounds(
        &traversal.project.objects()[object.source_object_index],
        print_object.transform,
        traversal.scale,
    );
    let resolution = traversal.resolved.views.full.process.print.resolution.0;
    let params = fill.params;
    let fill_params = crate::fill::plane_path::PlanePathFillParams {
        sparse: params.extrusion_role == crate::ExtrusionRole::InternalInfill,
        spacing: params.spacing,
        overlap: params.overlap,
        density: (0.01 * f64::from(params.density)) as f32,
        angle: params.angle,
        multiline: params.multiline,
        resolution,
        smooth_factor: params.smooth_factor,
        anchor_length: params.anchor_length,
        anchor_length_max: params.anchor_length_max,
        object_bounding_box,
        // `calibration_pattern_order` only applies to top-surface
        // (ironing) fill — sparse internal infill never takes it.
        calibration_order: false,
    };
    for expolygon in fill.expolygons {
        let components = crate::fill::plane_path::fill_surface(
            &expolygon,
            plane_path_pattern,
            fill_params,
            traversal.scale,
        )
        .map_err(super::super::transaction::geometry_error)?;
        result.extend(components.into_iter().flatten());
    }
    Ok(())
}
