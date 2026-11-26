//! Gyroid sparse anchoring (`Fill.cpp:1620,1654,1660-1662`,
//! `FillGyroid.cpp:318-324,345-349`).

use crate::{
    ProcessAdaptiveInfillMode, SliceError,
    fill::{
        adaptive_infill_field::{
            AdaptiveInfillField, AdaptiveInfillFields,
            shells::{ShellFillSpec, fill_adaptive_infill_shells},
            tpms::{AdaptiveTpmsFillParams, AdaptiveTpmsPattern, fill_adaptive_tpms},
        },
        gyroid::{GyroidFillParams, fill_component, fill_surface as fill_gyroid},
    },
    geometry::{CoordinateScale, ExPolygon, JoinType, Polyline, offset_expolygon},
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    },
};

fn append_stepped_shells(
    expolygons: &[ExPolygon],
    field: &AdaptiveInfillField,
    base_params: GyroidFillParams,
    spec: ShellFillSpec,
    (scale, result): (CoordinateScale, &mut Vec<Polyline>),
) -> Result<(), SliceError> {
    let delta = (-0.5 * base_params.spacing / scale.factor()) as f32;
    for expolygon in expolygons {
        let components = offset_expolygon(expolygon, delta, JoinType::Miter, 3.0)
            .map_err(super::super::transaction::geometry_error)?;
        for component in components {
            fill_adaptive_infill_shells(field, &component, spec, scale, |shell_density, part| {
                let mut shell_params = base_params;
                shell_params.density = shell_density;
                result.extend(fill_component(part, shell_params, scale)?);
                Ok(())
            })
            .map_err(super::super::transaction::geometry_error)?;
        }
    }
    Ok(())
}

pub(super) fn append(
    fill: SurfaceFill,
    traversal: &PreparedPostClassicTraversal,
    (object_index, layer_index, z): (usize, usize, f64),
    adaptive_fields: &AdaptiveInfillFields,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let interior_density =
        (0.01_f64 * f64::from(fill.params.adaptive_infill_interior_density)) as f32;
    let layer_height = {
        let prepared = &traversal.objects[object_index].predecessor.predecessor;
        let prelude = &prepared.predecessor.predecessor.object;
        let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
        plan.layers[layer_index].height
    };
    if fill.params.adaptive_infill_mode == ProcessAdaptiveInfillMode::SteppedShells
        && let Some(field) = adaptive_fields.get(fill.params.adaptive_infill_mode)
    {
        let base_params = GyroidFillParams {
            z,
            spacing: fill.params.spacing,
            overlap: 0.0,
            angle: fill.params.angle,
            density,
            multiline: fill.params.multiline,
            layer_height: layer_height as f32,
            gyroid_optimized: fill.params.gyroid_optimized,
            anchor_length: fill.params.anchor_length,
            anchor_length_max: fill.params.anchor_length_max,
            dont_sort: false,
        };
        let spec = ShellFillSpec {
            z: z - 0.5 * layer_height,
            spacing: fill.params.spacing,
            surface_density: density,
            interior_density,
            gradient: fill.params.adaptive_infill_gradient,
        };
        return append_stepped_shells(
            &fill.expolygons,
            field,
            base_params,
            spec,
            (traversal.scale, result),
        );
    }
    if fill.params.adaptive_infill_mode != ProcessAdaptiveInfillMode::Disabled
        && let Some(field) = adaptive_fields.get(fill.params.adaptive_infill_mode)
    {
        let params = AdaptiveTpmsFillParams {
            z,
            spacing: fill.params.spacing,
            overlap: 0.0,
            angle: fill.params.angle,
            density,
            interior_density,
            gradient: fill.params.adaptive_infill_gradient,
            multiline: fill.params.multiline,
            layer_height,
            anchor_length: fill.params.anchor_length,
            anchor_length_max: fill.params.anchor_length_max,
            dont_sort: false,
        };
        for expolygon in fill.expolygons {
            result.extend(
                fill_adaptive_tpms(
                    &expolygon,
                    AdaptiveTpmsPattern::Gyroid,
                    params,
                    field,
                    traversal.scale,
                )
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
        density,
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
