//! `TreeSupport.cpp:1724-1781::generate` non-Organic execution order.

use super::super::{
    organic::generation::TreeInput,
    overhangs::{SmartOverhangs, finalize::FinalLayerInputs},
};
use super::{
    contacts::{self, ContactInput, VerticalEnforcer},
    data::TreeSupportData,
    drawing::Drawing,
    drop_nodes::NodeDropping,
    drop_settings::DropSettings,
    heights::{self, HeightInput},
    nodes::NodeArena,
    raft_layers, smoothing, toolpaths,
};
use crate::{
    FloatOrPercent, SliceError, SlicerContext,
    fill::lightning::generator::SupportSettings,
    geometry::{BoundingBox, ClipperError, Polygon},
    project_slice::{
        layers::PlannedLayer,
        perimeters::types::Flow,
        support_material::{
            install_layers::InstalledSupportLayer,
            trim_object::{TrimLayer, TrimRegion},
        },
    },
};

pub(in crate::project_slice::support_material) struct NonOrganicInput<'a> {
    pub(in crate::project_slice::support_material) tree: &'a TreeInput<'a>,
    pub(in crate::project_slice::support_material) planned: &'a [PlannedLayer],
    pub(in crate::project_slice::support_material) tops: &'a [Vec<Polygon>],
    pub(in crate::project_slice::support_material) bottoms: &'a [Vec<Polygon>],
    pub(in crate::project_slice::support_material) trim_regions: &'a [Vec<TrimRegion<'a>>],
    pub(in crate::project_slice::support_material) bounds: BoundingBox,
    pub(in crate::project_slice::support_material) vertical_enforcers: &'a [VerticalEnforcer],
    pub(in crate::project_slice::support_material) independent: bool,
    pub(in crate::project_slice::support_material) brim_flow: Flow,
    pub(in crate::project_slice::support_material) brim_skirt_layers: usize,
    pub(in crate::project_slice::support_material) lightning_settings: SupportSettings,
}

pub(in crate::project_slice::support_material) fn generate(
    input: &NonOrganicInput<'_>,
    context: &mut SlicerContext,
) -> Result<Vec<InstalledSupportLayer>, SliceError> {
    let tree = input.tree;
    let object = tree.object;
    let mut overhangs = SmartOverhangs::new(object, tree.layers, tree.extrusion_width, tree.scale)
        .map_err(error)?;
    if object.enable_support.0 {
        let start = web_time::Instant::now();
        for layer in 0..tree.layers.len() {
            overhangs
                .detect_layer(layer, || start.elapsed().as_secs_f64())
                .map_err(error)?;
        }
        overhangs.propagate_sharp_tails().map_err(error)?;
        overhangs.collect_cluster_overhangs().map_err(error)?;
        for layer in 0..tree.layers.len() {
            overhangs
                .finalize_layer(
                    layer,
                    FinalLayerInputs {
                        lower_slices: layer
                            .checked_sub(1)
                            .map(|index| tree.slices[index].as_slice()),
                        regions: &tree.bridges[layer],
                        blockers: tree.blockers.get(layer).map(Vec::as_slice),
                        enforcers: tree.enforcers.get(layer).map(Vec::as_slice),
                    },
                )
                .map_err(error)?;
        }
    }
    let mut prefix = raft_layers::create(tree.slicing);
    let raft_count = prefix.len();
    let resolution = f64::from(0.2_f32);
    let interface_nozzle = super::super::super::project::region::nozzle(
        object.support_interface_filament.0,
        tree.nozzles,
    );
    let support_width = absolute(object.support_line_width, interface_nozzle);
    let support_width = if support_width > 0.0 {
        support_width
    } else {
        absolute(object.line_width, interface_nozzle)
    };
    let settings = DropSettings::new(object, support_width, resolution, tree.scale);
    let mut arena = NodeArena::new(settings.diameter_angle_scale);
    let mut data = TreeSupportData::new(
        tree.layers,
        object.tree_support_branch_angle.0,
        object.support_object_xy_distance.0,
        resolution,
        tree.scale,
    )
    .map_err(error)?;
    let mut top_z_distance = f64::from(object.support_top_z_distance.0 as f32);
    if top_z_distance > 0.0001 {
        top_z_distance = top_z_distance.max(f64::from(tree.slicing.min_layer_height as f32));
    }
    let style = super::super::super::style::resolve(object.support_type, object.support_style);
    let mut contacts = contacts::generate_contact_points(
        &ContactInput {
            object,
            layers: input.planned,
            overhangs: &overhangs.layers,
            vertical_enforcers: input.vertical_enforcers,
            bounds: input.bounds,
            top_z_distance,
            style,
            scale: tree.scale,
        },
        &data,
        &mut arena,
    )
    .map_err(error)?;
    let heights = heights::plan_layer_heights(
        &HeightInput {
            layers: input.planned,
            slicing: tree.slicing,
            independent: input.independent,
            top_z_distance,
            raft_layers: raft_count,
        },
        &mut arena,
        &mut contacts.layers,
    );
    NodeDropping {
        arena: &mut arena,
        layers: &mut contacts.layers,
        heights: &heights,
        data: &mut data,
        settings: &settings,
        object,
        style,
    }
    .run()
    .map_err(error)?;
    smoothing::smooth_nodes(&mut arena, &contacts.layers, object, tree.scale);
    let trim_layers = tree
        .slices
        .iter()
        .enumerate()
        .map(|(layer, slices)| TrimLayer {
            slices,
            sharp_tails: &overhangs.layers[layer].sharp_tails,
            regions: &input.trim_regions[layer],
        })
        .collect::<Vec<_>>();
    let mut drawn = Drawing {
        object,
        arena: &arena,
        contacts: &contacts,
        data: &mut data,
        heights: &heights,
        layers: tree.layers,
        object_layers: input.planned,
        tops: input.tops,
        bottoms: input.bottoms,
        raft_layers: raft_count,
        top_z_distance,
        line_width: absolute(object.support_line_width, tree.nozzles.0[0].0),
        support_width,
        bottom_gap: tree.slicing.gap_object_support,
        nominal_height: object.layer_height.0,
        nominal_support_flow: &tree.flows.base,
        lightning_settings: &input.lightning_settings,
        counts: tree.counts,
        trim_layers: &trim_layers,
        zero_gap_interface_top: tree.slicing.zero_gap_interface_top,
        gap_xy_first_layer: object.support_object_first_layer_gap.0,
        first_layer_width_scaled: tree
            .scale
            .scaled_delta(f64::from(tree.flows.first_layer.width))
            as i64,
        brim_skirt_layers: input.brim_skirt_layers,
        bed: std::slice::from_ref(&tree.bed),
        scale: tree.scale,
    }
    .draw_node_areas()
    .map_err(error)?;
    prefix.append(&mut drawn.layers);
    drawn.layers = prefix;
    toolpaths::generate(
        drawn,
        &toolpaths::ToolpathInput {
            object,
            parameters: tree.parameters,
            slicing: tree.slicing,
            flows: tree.flows,
            support_width,
            brim_flow: input.brim_flow,
            first_object_slices: &tree.slices[0],
            raft_layers: raft_count,
            scale: tree.scale,
        },
        context,
    )
}

fn absolute(width: FloatOrPercent, nozzle: f64) -> f64 {
    match width {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => nozzle * value.0 / 100.0,
    }
}

fn error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("tree support geometry: {error:?}"))
}
