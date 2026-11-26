//! TPMS-D and TPMS-FK sparse anchoring (`Fill.cpp:1620,1654,1660-1662`,
//! `FillTpmsD.cpp:120-139`, `FillTpmsFK.cpp:135-157`).

use crate::{
    ProcessAdaptiveInfillMode, SliceError,
    fill::{
        adaptive_infill_field::{
            AdaptiveInfillField, AdaptiveInfillFields,
            shells::{ShellFillSpec, fill_adaptive_infill_shells},
            tpms::{AdaptiveTpmsFillParams, AdaptiveTpmsPattern, fill_adaptive_tpms_component},
        },
        tpms_d::{TpmsDFillParams, fill_tpmsd_surface},
        tpms_fk::{TpmsFKFillParams, fill_tpmsfk_surface},
    },
    geometry::{CoordinateScale, ExPolygon, JoinType, Polyline, offset_expolygon},
    project_slice::{
        group_fills::SurfaceFill,
        perimeters::classic::traversal::types::PreparedPostClassicTraversal,
    },
};

fn planned_layer_height(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    layer_index: usize,
) -> f64 {
    let prepared = &traversal.objects[object_index].predecessor.predecessor;
    let prelude = &prepared.predecessor.predecessor.object;
    let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
    plan.layers[layer_index].height
}

fn adaptive_params(fill: &SurfaceFill, z: f64, layer_height: f64) -> AdaptiveTpmsFillParams {
    AdaptiveTpmsFillParams {
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
    }
}

fn shell_spec(adaptive: AdaptiveTpmsFillParams) -> ShellFillSpec {
    ShellFillSpec {
        z: adaptive.z - 0.5 * adaptive.layer_height,
        spacing: adaptive.spacing,
        surface_density: adaptive.density,
        interior_density: adaptive.interior_density,
        gradient: adaptive.gradient,
    }
}

fn fill_tpms_d_domain(
    domain: &ExPolygon,
    mode: ProcessAdaptiveInfillMode,
    adaptive_field: Option<&AdaptiveInfillField>,
    (params, adaptive): (TpmsDFillParams, AdaptiveTpmsFillParams),
    (scale, result): (CoordinateScale, &mut Vec<Polyline>),
) -> Result<(), SliceError> {
    if mode == ProcessAdaptiveInfillMode::SteppedShells
        && let Some(field) = adaptive_field
    {
        return fill_adaptive_infill_shells(
            field,
            domain,
            shell_spec(adaptive),
            scale,
            |shell_density, part| {
                let mut shell_params = params;
                shell_params.density = shell_density;
                result.extend(fill_tpmsd_surface(part, &shell_params, scale)?);
                Ok(())
            },
        )
        .map_err(super::super::transaction::geometry_error);
    }
    if mode != ProcessAdaptiveInfillMode::Disabled
        && let Some(field) = adaptive_field
    {
        result.extend(
            fill_adaptive_tpms_component(
                domain,
                AdaptiveTpmsPattern::TpmsD,
                adaptive,
                field,
                scale,
            )
            .map_err(super::super::transaction::geometry_error)?,
        );
        return Ok(());
    }
    result.extend(
        fill_tpmsd_surface(domain, &params, scale)
            .map_err(super::super::transaction::geometry_error)?,
    );
    Ok(())
}

fn fill_tpms_fk_domain(
    domain: &ExPolygon,
    mode: ProcessAdaptiveInfillMode,
    adaptive_field: Option<&AdaptiveInfillField>,
    (params, adaptive): (TpmsFKFillParams, AdaptiveTpmsFillParams),
    (scale, result): (CoordinateScale, &mut Vec<Polyline>),
) -> Result<(), SliceError> {
    if mode == ProcessAdaptiveInfillMode::SteppedShells
        && let Some(field) = adaptive_field
    {
        return fill_adaptive_infill_shells(
            field,
            domain,
            shell_spec(adaptive),
            scale,
            |shell_density, part| {
                let mut shell_params = params;
                shell_params.density = shell_density;
                result.extend(fill_tpmsfk_surface(part, &shell_params, scale)?);
                Ok(())
            },
        )
        .map_err(super::super::transaction::geometry_error);
    }
    if mode != ProcessAdaptiveInfillMode::Disabled
        && let Some(field) = adaptive_field
    {
        result.extend(
            fill_adaptive_tpms_component(
                domain,
                AdaptiveTpmsPattern::TpmsFk,
                adaptive,
                field,
                scale,
            )
            .map_err(super::super::transaction::geometry_error)?,
        );
        return Ok(());
    }
    result.extend(
        fill_tpmsfk_surface(domain, &params, scale)
            .map_err(super::super::transaction::geometry_error)?,
    );
    Ok(())
}

pub(super) fn append_tpms_d(
    fill: SurfaceFill,
    traversal: &PreparedPostClassicTraversal,
    (object_index, layer_index, z): (usize, usize, f64),
    adaptive_fields: &AdaptiveInfillFields,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    let layer_height = planned_layer_height(traversal, object_index, layer_index);
    let adaptive = adaptive_params(&fill, z, layer_height);
    let adaptive_field = adaptive_fields.get(fill.params.adaptive_infill_mode);
    let params = TpmsDFillParams {
        spacing: fill.params.spacing,
        density: adaptive.density,
        angle: fill.params.angle,
        layer_z: z,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let delta = (-0.5 * params.spacing / traversal.scale.factor()) as f32;
    for expolygon in fill.expolygons {
        for domain in offset_expolygon(&expolygon, delta, JoinType::Miter, 3.0)
            .map_err(super::super::transaction::geometry_error)?
        {
            fill_tpms_d_domain(
                &domain,
                fill.params.adaptive_infill_mode,
                adaptive_field,
                (params, adaptive),
                (traversal.scale, result),
            )?;
        }
    }
    Ok(())
}

pub(super) fn append_tpms_fk(
    fill: SurfaceFill,
    traversal: &PreparedPostClassicTraversal,
    (object_index, layer_index, z): (usize, usize, f64),
    adaptive_fields: &AdaptiveInfillFields,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    let layer_height = planned_layer_height(traversal, object_index, layer_index);
    let adaptive = adaptive_params(&fill, z, layer_height);
    let adaptive_field = adaptive_fields.get(fill.params.adaptive_infill_mode);
    let params = TpmsFKFillParams {
        spacing: fill.params.spacing,
        density: adaptive.density,
        angle: fill.params.angle,
        layer_z: z,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let delta = (-0.5 * params.spacing / traversal.scale.factor()) as f32;
    for expolygon in fill.expolygons {
        for domain in offset_expolygon(&expolygon, delta, JoinType::Miter, 3.0)
            .map_err(super::super::transaction::geometry_error)?
        {
            fill_tpms_fk_domain(
                &domain,
                fill.params.adaptive_infill_mode,
                adaptive_field,
                (params, adaptive),
                (traversal.scale, result),
            )?;
        }
    }
    Ok(())
}
