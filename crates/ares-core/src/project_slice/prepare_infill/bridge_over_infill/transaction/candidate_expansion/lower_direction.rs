use std::f64::consts::PI;

use crate::{
    ProcessInfillPattern,
    geometry::{ClipperError, Polygon, intersection_polygons_paths},
    project_slice::{
        group_fills::template_rotation_angle, perimeters::types::RegionLayerIndex,
        prepare_infill::bridge_over_infill::types::CandidateSurface,
        region_slices::RegionSurfaceKind,
    },
};

use super::ObjectView;

// PrintObject.cpp:3913-3937: configured direction only over the same region's sparse infill.
pub(super) fn configured_angle(
    view: ObjectView<'_>,
    candidate: &CandidateSurface,
    area: &[Polygon],
    has_anchors: bool,
) -> Result<Option<f64>, ClipperError> {
    let (_, inputs) = view.prelude.as_parts();
    let slot = view.prelude.region_layer_record_index(RegionLayerIndex {
        region_index: candidate.source.region_index,
        layer_index: candidate.source.layer_index,
    });
    let input = inputs[slot]
        .as_ref()
        .expect("candidate retains its source input");
    let region = view.prelude.region_options(input);
    if !has_anchors
        || !matches!(
            region.sparse_infill_pattern,
            ProcessInfillPattern::HilbertCurve | ProcessInfillPattern::OctagramSpiral
        )
    {
        return Ok(None);
    }
    let lower_index = candidate.source.layer_index - 1;
    let lower_slot = view.prelude.region_layer_record_index(RegionLayerIndex {
        region_index: candidate.source.region_index,
        layer_index: lower_index,
    });
    let Some(lower) = &view.horizontal.records[lower_slot] else {
        return Ok(None);
    };
    let mut sparse = Vec::new();
    for surface in &lower.fill_surfaces {
        let (kind, expolygon, ..) = surface.as_parts();
        if kind == RegionSurfaceKind::Internal {
            sparse.push(expolygon.contour().clone());
            sparse.extend(expolygon.holes().iter().cloned());
        }
    }
    if intersection_polygons_paths(area, &sparse)?.is_empty() {
        return Ok(None);
    }
    let (plan, _, _) = view.prelude.as_parts().0.as_parts().0.as_parts();
    let template = &region.sparse_infill_rotate_template.0;
    let degrees = if template.is_empty() {
        region.infill_direction.0
    } else {
        template_rotation_angle(
            template,
            &plan.layers,
            view.object_options,
            view.default_region,
            plan.layers[lower_index].id,
        )
    };
    let mut angle = PI * degrees / 180.0 + 0.5 * PI;
    if region.align_infill_direction_to_model.0 {
        angle += input.model_rotation_rad;
    }
    angle %= PI;
    if angle < 0.0 {
        angle += PI;
    }
    Ok(Some(angle))
}
