//! `TreeSupport3D.cpp::generate_support_areas` enabled single-object root.

use std::collections::VecDeque;

use super::super::{
    model_volumes::TreeModelVolumes,
    overhangs::{OverhangType, SmartOverhangs, finalize::FinalLayerInputs},
    remove_bridges::RegionBridges,
};
use super::{
    MeshGroupSettings, TreeSettings, drawing,
    initial_areas::{
        generation::{self, ObjectInputs},
        raft_contact,
    },
    interface_placer::{InterfacePlacer, RoofLayers},
    interfaces,
    mesh_settings::MeshGroupInput,
    nodes, output, pathing,
};
use crate::{
    ObjectOptions, OrcaFloats, RegionOptions, SliceError, SlicerContext,
    geometry::{ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, offset_paths},
    project_slice::{
        parameters::SlicingParameters,
        perimeters::flow::SupportFlows,
        support_material::{
            adhesion::{AdhesionInput, generate_raft_base},
            grid_generation::GeneratedSupportGeometry,
            install_layers::generate_support_layers,
            interface_counts::InterfaceCounts,
            layer_input::{LayerInput, to_polygons},
            toolpath_parameters::ToolpathParameters,
        },
    },
};

pub(in crate::project_slice::support_material) struct TreeInput<'a> {
    pub(in crate::project_slice::support_material) object: &'a ObjectOptions,
    pub(in crate::project_slice::support_material) flows: &'a SupportFlows,
    pub(in crate::project_slice::support_material) regions: &'a [RegionOptions],
    pub(in crate::project_slice::support_material) nozzles: &'a OrcaFloats,
    pub(in crate::project_slice::support_material) resolution: f64,
    pub(in crate::project_slice::support_material) slicing: &'a SlicingParameters,
    pub(in crate::project_slice::support_material) parameters: &'a ToolpathParameters,
    pub(in crate::project_slice::support_material) counts: &'a InterfaceCounts,
    pub(in crate::project_slice::support_material) slices: &'a [Vec<ExPolygon>],
    pub(in crate::project_slice::support_material) layers: &'a [LayerInput<'a>],
    pub(in crate::project_slice::support_material) bridges: &'a [Vec<RegionBridges<'a>>],
    pub(in crate::project_slice::support_material) enforcers: &'a [Vec<Polygon>],
    pub(in crate::project_slice::support_material) blockers: &'a [Vec<Polygon>],
    pub(in crate::project_slice::support_material) extrusion_width: f64,
    pub(in crate::project_slice::support_material) bed: Polygon,
    pub(in crate::project_slice::support_material) has_brim: bool,
    pub(in crate::project_slice::support_material) scale: CoordinateScale,
}

pub(in crate::project_slice::support_material) fn generate(
    input: &TreeInput<'_>,
    context: &mut SlicerContext,
) -> Result<GeneratedSupportGeometry, SliceError> {
    let mesh = MeshGroupSettings::new(MeshGroupInput {
        object: input.object,
        nominal_flows: input.flows,
        regions: input.regions,
        nozzle_diameters: input.nozzles,
        resolution: input.resolution,
        slicing: input.slicing,
        scale: input.scale,
    })?;
    let config = TreeSettings::new(
        mesh,
        input.slicing,
        input.slicing.zero_gap_interface_top,
        input.scale,
    );
    let mut volumes = TreeModelVolumes::new(
        &config,
        input.slices,
        input.blockers.to_vec(),
        input.bed.clone(),
        input.scale,
    )
    .map_err(error)?;
    let overhangs = detect(input, config.raft_layers.len()).map_err(error)?;
    let maximum = overhangs
        .iter()
        .enumerate()
        .skip(config.raft_layers.len().max(1))
        .filter(|(_, polygons)| !polygons.is_empty())
        .map(|(index, _)| index)
        .next_back()
        .unwrap_or(0);
    let support_count = (maximum as i32 - config.z_distance_top_layers as i32).max(0) as usize;
    if support_count > 0 {
        volumes.precalculate(support_count).map_err(error)?;
    }
    let layer_count = support_count.max(config.raft_layers.len());
    let mut roofs = RoofLayers::new(layer_count);
    if layer_count == 0 {
        return Ok(output::geometry(&mut roofs));
    }
    let mut interface = InterfacePlacer {
        slicing: input.slicing,
        parameters: input.parameters,
        config: &config,
        counts: input.counts,
        roofs: &mut roofs,
    };
    let objects = ObjectInputs {
        slices: input.slices,
        overhangs: &overhangs,
        options: input.object,
        scale: input.scale,
    };
    if support_count > 0 {
        let mut graph = (0..layer_count)
            .map(|_| VecDeque::new())
            .collect::<Vec<_>>();
        generation::generate(objects, &mut volumes, &mut interface, &mut graph, context)
            .map_err(error)?;
        pathing::create_layer_pathing(&mut volumes, &config, &mut graph, input.scale)
            .map_err(error)?;
        nodes::create_nodes_from_area(&mut volumes, &config, &mut graph).map_err(error)?;
        let slices =
            drawing::draw_branch_slices(&mut volumes, &mut graph, &mut interface, input.scale)?;
        drawing::allocate_layers(
            slices,
            graph.len(),
            interface.roofs,
            &drawing::OrganicLayerContext {
                config: &config,
                slicing: input.slicing,
            },
            input.scale,
        )
        .map_err(error)?;
        interfaces::generate(interface.roofs, input.counts, input.parameters, input.scale)
            .map_err(error)?;
    } else {
        raft_contact::generate(&objects, &mut interface).map_err(error)?;
    }
    let mut geometry = output::geometry(&mut roofs);
    geometry.rafts = generate_raft_base(
        &AdhesionInput {
            object: input.object,
            parameters: input.slicing,
            first_object_slices: &input.slices[0],
            has_brim: input.has_brim,
            gap_xy_first_layer: input.object.support_object_first_layer_gap.0,
            first_layer_flow_width_scaled: input
                .scale
                .scaled_delta(f64::from(input.parameters.first_layer_flow.width))
                as i64,
            scale: input.scale,
        },
        &mut geometry.tops,
        &mut geometry.interfaces,
        &mut geometry.base_interfaces,
        &mut geometry.bases,
    )
    .map_err(error)?;
    geometry.support_layers = generate_support_layers(&mut geometry);
    output::clip_to_bed(&mut geometry, std::slice::from_ref(&input.bed)).map_err(error)?;
    Ok(geometry)
}

fn detect(input: &TreeInput<'_>, raft_count: usize) -> Result<Vec<Vec<Polygon>>, ClipperError> {
    let mut overhangs = vec![Vec::new(); raft_count + input.slices.len()];
    if !input.object.enable_support.0 {
        return Ok(overhangs);
    }
    let mut smart = SmartOverhangs::new(
        input.object,
        input.layers,
        input.extrusion_width,
        input.scale,
    )?;
    let start = web_time::Instant::now();
    for layer in 0..input.layers.len() {
        smart.detect_layer(layer, || start.elapsed().as_secs_f64())?;
    }
    smart.propagate_sharp_tails()?;
    smart.collect_cluster_overhangs()?;
    for layer in 0..input.layers.len() {
        smart.finalize_layer(
            layer,
            FinalLayerInputs {
                lower_slices: layer
                    .checked_sub(1)
                    .map(|index| input.slices[index].as_slice()),
                regions: &input.bridges[layer],
                blockers: input.blockers.get(layer).map(Vec::as_slice),
                enforcers: input.enforcers.get(layer).map(Vec::as_slice),
            },
        )?;
        for (polygon, kind) in smart.layers[layer]
            .overhangs
            .iter()
            .zip(&smart.layers[layer].overhang_types)
        {
            let mut polygons = to_polygons(std::slice::from_ref(polygon));
            if *kind == OverhangType::SharpTail {
                polygons = offset_paths(
                    &polygons,
                    input.scale.scaled_delta(0.2) as f32,
                    JoinType::Miter,
                    3.0,
                )?;
            }
            overhangs[layer + raft_count].extend(polygons);
        }
    }
    Ok(overhangs)
}

fn error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("organic support geometry: {error:?}"))
}
