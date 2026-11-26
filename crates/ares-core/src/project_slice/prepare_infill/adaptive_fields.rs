//! `PrintObject::prepare_adaptive_infill_fields` (`PrintObject.cpp:1274-1304`).

use std::sync::Arc;

use crate::{
    ProcessAdaptiveInfillMode,
    fill::adaptive_infill_field::{
        AdaptiveInfillField, AdaptiveInfillFields, MODE_COUNT, Slice, is_adaptive_infill_pattern,
        mode_from_index, mode_index,
    },
    geometry::{BoundingBox, CoordinateScale},
    project_slice::perimeters::types::PostPerimeterInputPrintObject,
};

pub(in crate::project_slice) fn prepare_adaptive_infill_fields(
    prelude: &PostPerimeterInputPrintObject,
    scale: CoordinateScale,
) -> AdaptiveInfillFields {
    let (compensated, _) = prelude.as_parts();
    let (post_regions, lslices) = compensated.as_parts();
    let (plan, _, regions) = post_regions.as_parts();

    let mut active_modes = [false; MODE_COUNT];
    for region in regions {
        let config = region.as_parts().1;
        if config.sparse_infill_density.0 > 0.0
            && config.sparse_infill_density.0 < 100.0
            && is_adaptive_infill_pattern(config.sparse_infill_pattern)
        {
            active_modes[mode_index(config.adaptive_infill_mode)] = true;
        }
    }
    active_modes[mode_index(ProcessAdaptiveInfillMode::Disabled)] = false;
    if !active_modes.iter().any(|&active| active) || plan.layers.is_empty() {
        return AdaptiveInfillFields::default();
    }

    let mut slices = Vec::with_capacity(plan.layers.len());
    let mut bbox: Option<BoundingBox> = None;
    for (layer, layer_slices) in plan.layers.iter().zip(lslices) {
        slices.push(Slice {
            bottom_z: layer.print_z - layer.height,
            top_z: layer.print_z,
            expolygons: layer_slices.as_slice(),
        });
        if let Some(layer_bbox) = BoundingBox::from_expolygons(layer_slices) {
            match &mut bbox {
                Some(bounds) => {
                    bounds.merge_point(layer_bbox.min());
                    bounds.merge_point(layer_bbox.max());
                }
                None => bbox = Some(layer_bbox),
            }
        }
    }
    let Some(bbox) = bbox else {
        return AdaptiveInfillFields::default();
    };

    let mut fields: [Option<Arc<AdaptiveInfillField>>; MODE_COUNT] = Default::default();
    for (mode_idx, &active) in active_modes.iter().enumerate() {
        if !active {
            continue;
        }
        let mode = mode_from_index(mode_idx).expect("active adaptive infill mode index is valid");
        let field = AdaptiveInfillField::new(&slices, bbox, mode, scale);
        if !field.empty() {
            fields[mode_idx] = Some(Arc::new(field));
        }
    }
    AdaptiveInfillFields::new(fields)
}
