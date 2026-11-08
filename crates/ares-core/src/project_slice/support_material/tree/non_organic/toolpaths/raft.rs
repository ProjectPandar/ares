//! `TreeSupport.cpp:1382-1480` raft base/interface/gap toolpaths.

use super::super::drawing::groups::AreaKind;
use super::{
    LayerAreas, ToolpathInput,
    fill::{FillSettings, Filler},
    geometry_error,
    perimeter::PerimeterInfill,
};
use crate::{
    ExtrusionRole, SliceError,
    geometry::{JoinType, difference_ex, intersection_ex, offset_expolygons},
    project_slice::{
        raft::fill_params::RaftFillPattern,
        support_material::{SupportExtrusionEntity, install_layers::InstalledSupportLayer},
    },
};

pub(super) fn fill_prefix(
    output: &mut [InstalledSupportLayer],
    layers: &[LayerAreas],
    input: &ToolpathInput<'_>,
) -> Result<(), SliceError> {
    let scale = input.scale;
    let mut areas = input.first_object_slices.to_vec();
    if let Some(first) = layers.get(input.raft_layers) {
        areas.extend_from_slice(&first.floor);
        areas.extend_from_slice(&first.roof);
        areas.extend_from_slice(&first.base);
    }
    let expansion = input.object.raft_first_layer_expansion.0;
    let areas = offset_expolygons(
        &areas,
        (scale.scaled_delta(expansion) as i64) as f32,
        JoinType::Miter,
        3.0,
    )
    .map_err(geometry_error)?;
    let base_layers = input.slicing.base_raft_layers;
    let interface_end = base_layers + input.slicing.interface_raft_layers;
    for (index, layer) in output.iter_mut().enumerate().take(base_layers) {
        let flow = input.support_flow(layer.height)?;
        let mut regions = offset_expolygons(
            &areas,
            if index == 0 {
                (scale.scaled_delta(expansion) as i64) as f32
            } else {
                0.0
            },
            JoinType::Miter,
            3.0,
        )
        .map_err(geometry_error)?;
        let filler = Filler {
            scale,
            pattern: RaftFillPattern::Rectilinear,
            spacing: f64::from(flow.spacing),
            angle: if index == 0 {
                std::f64::consts::FRAC_PI_2 as f32
            } else {
                0.0
            },
            reference: None,
        };
        let density = if index == 0 {
            input.object.raft_first_layer_density.0 * 0.01
        } else {
            (f64::from(flow.spacing)
                / (input.object.support_base_pattern_spacing.0 + f64::from(flow.spacing)))
            .min(1.0)
        };
        if index == 0 {
            let loops = regions
                .iter()
                .flat_map(|area| std::iter::once(area.contour()).chain(area.holes()))
                .cloned()
                .collect();
            SupportExtrusionEntity::append_loops(
                &mut layer.support_fills,
                loops,
                ExtrusionRole::SupportMaterial,
                (f64::from(flow.mm3_per_mm as f32), flow.width, flow.height),
            );
            regions = offset_expolygons(
                &regions,
                (-flow.scaled_spacing(scale) as f64 / 2.0) as f32,
                JoinType::Miter,
                3.0,
            )
            .map_err(geometry_error)?;
        }
        filler
            .append(
                &mut layer.support_fills,
                &regions,
                FillSettings {
                    density,
                    dont_sort: false,
                    flow,
                    role: ExtrusionRole::SupportMaterial,
                },
            )
            .map_err(geometry_error)?;
    }
    let mut bases = Vec::new();
    if let Some(first) = layers.get(input.raft_layers) {
        bases.extend(
            first
                .groups
                .iter()
                .filter(|group| group.kind == AreaKind::Base)
                .map(|group| first.base[group.index].clone()),
        );
    }
    // This native offset is deliberately not scale_(support_extrusion_width).
    let bases = offset_expolygons(&bases, input.flows.base.width, JoinType::Miter, 3.0)
        .map_err(geometry_error)?;
    let base_areas = intersection_ex(&areas, &bases).map_err(geometry_error)?;
    let interface_areas = difference_ex(&areas, &base_areas).map_err(geometry_error)?;
    for layer in output.iter_mut().take(interface_end).skip(base_layers) {
        let flow = input.support_flow(layer.height)?;
        let filler = Filler {
            scale,
            pattern: RaftFillPattern::Rectilinear,
            spacing: f64::from(flow.spacing),
            angle: std::f64::consts::FRAC_PI_2 as f32,
            reference: None,
        };
        let fill = FillSettings {
            density: input.interface_density(false),
            dont_sort: false,
            flow,
            role: ExtrusionRole::SupportMaterialInterface,
        };
        filler
            .append(&mut layer.support_fills, &interface_areas, fill)
            .map_err(geometry_error)?;
        filler
            .append(
                &mut layer.support_fills,
                &base_areas,
                FillSettings {
                    density: input.object.raft_first_layer_density.0 * 0.01,
                    role: ExtrusionRole::SupportMaterial,
                    ..fill
                },
            )
            .map_err(geometry_error)?;
    }
    for layer in output
        .iter_mut()
        .take(input.raft_layers)
        .skip(interface_end)
    {
        let flow = input.support_flow(layer.height)?;
        let filler = Filler {
            scale,
            pattern: RaftFillPattern::Rectilinear,
            spacing: f64::from(flow.spacing),
            angle: std::f64::consts::FRAC_PI_2 as f32,
            reference: None,
        };
        for area in &bases {
            filler
                .perimeter_and_infill(
                    &mut layer.support_fills,
                    area,
                    PerimeterInfill {
                        wall_count: 1.min(input.object.tree_support_wall_count.0 as usize),
                        fill: FillSettings {
                            density: input.interface_density(false),
                            dont_sort: false,
                            flow,
                            role: ExtrusionRole::SupportMaterial,
                        },
                        infill_first: false,
                    },
                )
                .map_err(geometry_error)?;
        }
    }
    Ok(())
}
