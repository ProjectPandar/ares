//! PrintObjectSupportMaterial inputs after make_fills, before support toolpaths.

mod annotations;
mod bed;
pub(super) mod region;
mod tree_input;

use super::{
    contact_placement::ContactPlacement,
    grid_generation::{GridSupportInput, generate_grid_support_geometry},
    install_layers::InstalledSupportLayer,
    interface_counts::InterfaceCounts,
    layer_input::LayerInput,
    toolpath_parameters::ToolpathParameters,
    toolpaths::{ToolpathInput, generate_grid_support_toolpaths},
};
use crate::{
    FloatOrPercent, ProcessBrimType, ProcessSupportStyle, ProcessSupportType, ProjectVolumeType,
    SliceError,
    geometry::support_grid::SupportGridParams,
    project_slice::{
        extruders::collect_project_object_extruders,
        fill_entities::PreparedPostFillEntities,
        perimeters::{
            flow::{resolve_support_flows, with_flow_ratio},
            types::{Flow, RegionLayerIndex},
        },
    },
};
use region::{Region, RegionSource};

pub(in crate::project_slice) fn generate(
    prepared: &PreparedPostFillEntities,
    context: &mut crate::SlicerContext,
) -> Result<Vec<Vec<InstalledSupportLayer>>, SliceError> {
    let horizontal = &prepared.predecessor.predecessor.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let original_extruders = collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    );
    let settings = &traversal.resolved.views.full;
    let scale = traversal.scale;
    let max_object_layers = traversal
        .objects
        .iter()
        .map(|object| {
            object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .as_parts()
                .0
                .as_parts()
                .0
                .plan
                .layers
                .len()
        })
        .fold(0, usize::max);
    let mut output = Vec::with_capacity(traversal.objects.len());
    for (index, source) in traversal.objects.iter().enumerate() {
        let input = &source
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let (compensated, _) = input.as_parts();
        let (post_regions, slices) = compensated.as_parts();
        let plan = &post_regions.plan;
        let resolved_index = traversal
            .resolved
            .objects
            .iter()
            .position(|resolved| resolved.source_object_index == plan.source_object_index)
            .expect("planned support object retains its resolved configuration");
        let object = &traversal.resolved.objects[resolved_index].object;
        let tree = matches!(
            object.support_type,
            ProcessSupportType::TreeAuto | ProcessSupportType::TreeManual
        );
        let organic = tree
            && super::style::resolve(object.support_type, object.support_style)
                == ProcessSupportStyle::Organic;
        // PrintObject.cpp:998–1002 dispatches raft-only objects to their support generator.
        let tree_raft = tree && object.raft_layers.0 > 0 && !plan.layers.is_empty();
        if !object.enable_support.0 && object.enforce_support_layers.0 == 0 && !tree_raft {
            output.push(Vec::new());
            continue;
        }
        if plan.layers.len() > 1 || object.raft_layers.0 > 0 {
            context.set_status(50, format_args!("Generating support"), 0);
        }
        let source_object = &traversal.project.objects()[plan.source_object_index];
        let resolved_object = &traversal.resolved.objects[resolved_index];
        let mut enforcers = annotations::slice_volumes(
            source_object,
            resolved_object,
            plan,
            ProjectVolumeType::SupportEnforcer,
            scale,
        )?;
        let mut blockers = annotations::slice_volumes(
            source_object,
            resolved_object,
            plan,
            ProjectVolumeType::SupportBlocker,
            scale,
        )?;
        let facets = super::facet_projection::Input {
            source: source_object,
            resolved: resolved_object,
            plan,
            scale,
        };
        let vertical_enforcers = super::facet_projection::append(&facets, 1, &mut enforcers)?;
        super::facet_projection::append(&facets, 2, &mut blockers)?;
        let bridge_flow_ratio = post_regions
            .regions
            .iter()
            .map(|region| region.options.bridge_flow.0)
            .sum::<f64>()
            / post_regions.regions.len() as f64;
        let data = plan
            .layers
            .iter()
            .enumerate()
            .map(|(layer_index, layer)| {
                post_regions
                    .regions
                    .iter()
                    .enumerate()
                    .map(|(region_index, region)| {
                        let slot = input.region_layer_record_index(RegionLayerIndex {
                            region_index,
                            layer_index,
                        });
                        Region::new(
                            RegionSource {
                                region_index,
                                options: &region.options,
                                object,
                                layer,
                                raw_slices: input.raw_slices(region_index, layer_index),
                                typed: horizontal.objects[index].records[slot].as_ref(),
                                fills: &prepared.objects[index][layer_index],
                            },
                            settings,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let region_views = data
            .iter()
            .map(|regions| {
                regions
                    .iter()
                    .map(|region| region.view(scale))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let layers = region_views
            .iter()
            .enumerate()
            .map(|(layer_index, regions)| LayerInput {
                slices: &slices[layer_index],
                regions,
                height: plan.layers[layer_index].height,
            })
            .collect::<Vec<_>>();
        let top_surfaces = data
            .iter()
            .map(|regions| {
                regions
                    .iter()
                    .flat_map(|region| &region.tops)
                    .flat_map(|polygon| std::iter::once(polygon.contour()).chain(polygon.holes()))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let bridging_heights = data
            .iter()
            .map(|regions| {
                regions.iter().any(Region::has_bridging).then(|| {
                    regions
                        .iter()
                        .map(|region| region.bridging_height)
                        .sum::<f64>()
                        / regions.len() as f64
                })
            })
            .collect::<Vec<_>>();
        let trim_regions = data
            .iter()
            .map(|regions| regions.iter().map(Region::trim).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let slicing = &plan.slicing_parameters;
        let flows = resolve_support_flows(
            settings,
            object,
            slicing.first_print_layer_height,
            slicing.layer_height,
        )?;
        let mut parameters =
            ToolpathParameters::new(object, slicing, &flows, &original_extruders[resolved_index])?;
        if organic {
            parameters.with_sheath = true;
        }
        let counts =
            InterfaceCounts::new(object, slicing, &settings.filament.gcode.filament_soluble);
        let has_brim = object.raft_layers.0 <= 0
            && (object.brim_type == ProcessBrimType::AutoBrim
                || (object.brim_type != ProcessBrimType::NoBrim && object.brim_width.0 > 0.0));
        if tree {
            let bottom_bridges = data
                .iter()
                .map(|regions| {
                    regions
                        .iter()
                        .map(Region::tree_bottom_bridges)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let bridges = region_views
                .iter()
                .enumerate()
                .map(|(layer, regions)| {
                    regions
                        .iter()
                        .enumerate()
                        .map(
                            |(region, view)| super::tree::remove_bridges::RegionBridges {
                                perimeters: view.bridges.perimeters,
                                bridge_width_scaled: view.bridges.bridge_width_scaled as i64,
                                bridge_spacing_scaled: view.bridges.bridge_spacing_scaled as i64,
                                bottom_surfaces: &bottom_bridges[layer][region],
                                unsupported_edges: view.bridges.unsupported_edges,
                            },
                        )
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let regions = post_regions
                .regions
                .iter()
                .map(|region| region.options.clone())
                .collect::<Vec<_>>();
            let nozzles = &settings.project.print.nozzle_diameter;
            let first_nozzle = nozzles.0[0].0;
            let extrusion_width = match object.line_width {
                FloatOrPercent::Float(width) => width,
                FloatOrPercent::Percent(percent) => first_nozzle * percent.0 / 100.0,
            };
            let tree_input = super::tree::TreeInput {
                object,
                flows: &flows,
                regions: &regions,
                nozzles,
                resolution: settings.process.print.resolution.0,
                slicing,
                parameters: &parameters,
                counts: &counts,
                slices,
                layers: &layers,
                planned: &plan.layers,
                bridges: &bridges,
                enforcers: &enforcers,
                vertical_enforcers: &vertical_enforcers,
                blockers: &blockers,
                extrusion_width,
                bed: bed::polygon(
                    settings,
                    source_object,
                    &resolved_object.print_objects[plan.transform_index],
                    scale,
                )?,
                has_brim,
                scale,
            };
            if !organic {
                output.push(tree_input::generate(
                    &tree_input,
                    &tree_input::ProjectInputs {
                        source: source_object,
                        resolved: resolved_object,
                        printing_objects: &traversal.resolved.objects,
                        plan,
                        data: &data,
                        tops: &top_surfaces,
                        trim_regions: &trim_regions,
                        settings,
                        max_object_layers,
                        vertical_enforcers: &vertical_enforcers,
                    },
                    context,
                )?);
                continue;
            }
            let mut geometry = super::tree::generate(&tree_input, context)?;
            output.push(generate_grid_support_toolpaths(
                &mut geometry,
                &ToolpathInput {
                    object,
                    slicing,
                    parameters: &parameters,
                    counts: &counts,
                    scale,
                },
            )?);
            continue;
        }
        let height_min = settings
            .project
            .print
            .min_layer_height
            .0
            .iter()
            .map(|height| height.0.max(0.01))
            .chain(plan.layers.iter().map(|layer| layer.height.max(0.01)))
            .fold(0.01 / scale.factor(), f64::min);
        let interface_nozzle = region::nozzle(
            object.support_interface_filament.0,
            &settings.project.print.nozzle_diameter,
        );
        let extrusion_width = match object.line_width {
            FloatOrPercent::Float(width) => width,
            FloatOrPercent::Percent(percent) => interface_nozzle * percent.0 / 100.0,
        };
        let bottom_interface_flow = if slicing.zero_gap_interface_bottom || !object.thick_bridges.0
        {
            with_flow_ratio(parameters.interface_flow, bridge_flow_ratio)
        } else {
            Flow::bridging_flow(
                (bridge_flow_ratio * f64::from(parameters.interface_flow.nozzle_diameter)) as f32,
                parameters.interface_flow.nozzle_diameter,
            )
        };
        let mut geometry = generate_grid_support_geometry(&GridSupportInput {
            toolpath_parameters: &parameters,
            placement: ContactPlacement {
                parameters: slicing,
                object,
                independent_support_layer_height: settings
                    .process
                    .print
                    .independent_support_layer_height
                    .0,
                support_layer_height_min: height_min,
                layers: &plan.layers,
            },
            layers: &layers,
            enforcers: &enforcers,
            blockers: &blockers,
            top_surfaces: &top_surfaces,
            bridging_heights: &bridging_heights,
            trim_regions: &trim_regions,
            interface_counts: &counts,
            extrusion_width_scaled: extrusion_width / scale.factor(),
            bottom_interface_height: f64::from(bottom_interface_flow.height),
            support_width_scaled: (f64::from(parameters.base_flow.width) / scale.factor()) as i64
                as f32,
            gap_xy: object.support_object_xy_distance.0,
            gap_xy_first_layer: object.support_object_first_layer_gap.0,
            first_layer_flow_width_scaled: (f64::from(parameters.first_layer_flow.width)
                / scale.factor()) as i64,
            has_brim,
            grid: SupportGridParams {
                snug: super::style::resolve(object.support_type, object.support_style)
                    == ProcessSupportStyle::Snug,
                angle: object.support_angle.0 * std::f64::consts::PI / 180.0,
                ..SupportGridParams::new(
                    object.support_base_pattern_spacing.0 + f64::from(parameters.base_flow.spacing),
                    f64::from(parameters.base_flow.spacing),
                    1.0 / scale.factor(),
                )
            },
            scale,
        })
        .map_err(|error| {
            SliceError::InvalidInput(format!("support project geometry: {error:?}"))
        })?;
        output.push(generate_grid_support_toolpaths(
            &mut geometry,
            &ToolpathInput {
                object,
                slicing,
                parameters: &parameters,
                counts: &counts,
                scale,
            },
        )?);
    }
    Ok(output)
}
