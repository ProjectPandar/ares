//! Raft-prefix branches of `SupportCommon.cpp:1442-1521`.

use super::ToolpathInput;
use crate::{
    ExtrusionRole, SliceError,
    geometry::{Polygon, difference_polygons_paths},
    project_slice::{
        perimeters::types::Flow,
        raft::{
            fill_params::{RaftFillPattern, tree_double_wall_area},
            fills::{RaftFillPreprocess, RaftFillSpec, raft_layer_entities},
        },
        support_material::{
            extrusion_path::SupportExtrusionEntity, grid_generation::GeneratedSupportGeometry,
            install_layers::InstalledSupportLayer,
        },
    },
};

pub(super) fn fill_prefix(
    layers: &mut [InstalledSupportLayer],
    geometry: &GeneratedSupportGeometry,
    input: &ToolpathInput<'_>,
) -> Result<(), SliceError> {
    for (index, layer) in layers.iter_mut().enumerate() {
        let raft = &geometry.rafts[index];
        let columns = if index > 0
            && index < geometry.bases.len()
            && (geometry.bases[index].print_z - layer.print_z).abs() < 1e-4
        {
            geometry.bases[index].polygons.as_slice()
        } else {
            &[]
        };
        if index > 0 {
            let regions = if index < input.slicing.base_raft_layers {
                raft.polygons.as_slice()
            } else {
                raft.contact_polygons.as_deref().unwrap_or(&[])
            };
            let flow = input.parameters.base_flow.with_height(raft.height as f32)?;
            if !regions.is_empty() {
                append_fill(
                    layer,
                    (regions, columns),
                    FillPlan {
                        flow,
                        role: ExtrusionRole::SupportMaterial,
                        angle: input.parameters.raft_angle_base,
                        spacing: input.parameters.base_flow.spacing,
                        density: input.parameters.support_density as f32,
                        pattern: input.parameters.base_fill_pattern,
                        sheath: input.parameters.with_sheath,
                        no_sort: false,
                    },
                    input,
                )?;
            }
            if !columns.is_empty() {
                append_fill(
                    layer,
                    (columns, &[]),
                    FillPlan {
                        flow,
                        role: ExtrusionRole::SupportMaterial,
                        angle: 0.0,
                        spacing: flow.spacing,
                        density: 0.0,
                        pattern: RaftFillPattern::SupportBase,
                        sheath: true,
                        no_sort: false,
                    },
                    input,
                )?;
            }
        }
        let (flow, angle, spacing, density) = if index == 0 {
            let flow = input.parameters.first_layer_flow;
            (
                flow,
                input.parameters.raft_angle_first_layer,
                flow.spacing,
                (input.object.raft_first_layer_density.0 * 0.01) as f32,
            )
        } else if index >= input.slicing.base_raft_layers {
            (
                input
                    .parameters
                    .raft_interface_flow
                    .with_height(raft.height as f32)?,
                input.parameters.raft_interface_angle(layer.interface_id),
                input.parameters.base_flow.spacing,
                input.parameters.raft_interface_density as f32,
            )
        } else {
            continue;
        };
        append_fill(
            layer,
            (&raft.polygons, columns),
            FillPlan {
                flow,
                role: if index < input.slicing.base_raft_layers {
                    ExtrusionRole::SupportMaterial
                } else {
                    ExtrusionRole::SupportMaterialInterface
                },
                angle,
                spacing,
                density,
                pattern: input.parameters.raft_interface_fill_pattern,
                sheath: index == 0,
                no_sort: index == 0,
            },
            input,
        )?;
    }
    Ok(())
}

struct FillPlan {
    flow: Flow,
    role: ExtrusionRole,
    angle: f32,
    spacing: f32,
    density: f32,
    pattern: RaftFillPattern,
    sheath: bool,
    no_sort: bool,
}

fn append_fill(
    layer: &mut InstalledSupportLayer,
    (regions, columns): (&[Polygon], &[Polygon]),
    plan: FillPlan,
    input: &ToolpathInput<'_>,
) -> Result<(), SliceError> {
    let spec = RaftFillSpec {
        angle: f64::from(plan.angle),
        spacing: f64::from(plan.spacing),
        sheath_spacing: f64::from(plan.flow.spacing),
        density: plan.density,
        sheath: plan.sheath,
        no_sort: plan.no_sort,
        tree_double_wall_area: tree_double_wall_area(
            input.object.tree_support_wall_count.0,
            1.0 / input.scale.factor(),
        ),
        width_mm: f64::from(plan.flow.width),
        pattern: plan.pattern,
        preprocess: RaftFillPreprocess::Closing,
    };
    let trimmed;
    let regions = if columns.is_empty() {
        regions
    } else {
        trimmed = difference_polygons_paths(regions, columns).map_err(|error| {
            SliceError::InvalidInput(format!("support raft columns trim: {error:?}"))
        })?;
        &trimmed
    };
    let entities = raft_layer_entities(regions, spec, input.scale).map_err(|error| {
        SliceError::InvalidInput(format!("support raft prefix fill: {error:?}"))
    })?;
    layer.support_fills.extend(
        entities
            .into_iter()
            .map(|entity| SupportExtrusionEntity::from_fill_entity(entity, plan.role, plan.flow)),
    );
    Ok(())
}
