//! Gyroid sparse anchoring (`Fill.cpp:1620,1660-1662`, `FillGyroid.cpp:345-349`).

use crate::{
    ProcessAdaptiveInfillMode, SliceError,
    fill::{
        adaptive_infill_field::{
            AdaptiveInfillFields,
            tpms::{AdaptiveGyroidParams, fill_adaptive_gyroid},
        },
        gyroid::{GyroidFillParams, fill_surface as fill_gyroid},
    },
    geometry::Polyline,
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    },
};

pub(super) fn append(
    fill: SurfaceFill,
    traversal: &PreparedPostClassicTraversal,
    (object_index, layer_index, z): (usize, usize, f64),
    adaptive_fields: &AdaptiveInfillFields,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    if fill.params.adaptive_infill_mode != ProcessAdaptiveInfillMode::Disabled
        && let Some(field) = adaptive_fields.get(fill.params.adaptive_infill_mode)
    {
        let layer_height = {
            let prepared = &traversal.objects[object_index].predecessor.predecessor;
            let prelude = &prepared.predecessor.predecessor.object;
            let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
            plan.layers[layer_index].height
        };
        let params = AdaptiveGyroidParams {
            z,
            spacing: fill.params.spacing,
            overlap: 0.0,
            angle: fill.params.angle,
            density: (0.01_f64 * f64::from(fill.params.density)) as f32,
            interior_density: (0.01_f64 * f64::from(fill.params.adaptive_infill_interior_density))
                as f32,
            gradient: fill.params.adaptive_infill_gradient,
            multiline: fill.params.multiline,
            layer_height,
            anchor_length: fill.params.anchor_length,
            anchor_length_max: fill.params.anchor_length_max,
            dont_sort: false,
        };
        for expolygon in fill.expolygons {
            result.extend(
                fill_adaptive_gyroid(&expolygon, params, field, traversal.scale)
                    .map_err(super::super::transaction::geometry_error)?,
            );
        }
        return Ok(());
    }
    let params = GyroidFillParams {
        z,
        spacing: fill.params.spacing,
        overlap: 0.0,
        angle: fill.params.angle,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        layer_height: fill.params.flow.height,
        gyroid_optimized: fill.params.gyroid_optimized,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    for expolygon in fill.expolygons {
        result.extend(
            fill_gyroid(&expolygon, params, traversal.scale)
                .map_err(super::super::transaction::geometry_error)?,
        );
    }
    Ok(())
}
