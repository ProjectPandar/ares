//! `TreeSupport.cpp:1355-1693::generate_toolpaths`.

mod base;
mod fill;
mod interfaces;
mod loops;
mod perimeter;
mod raft;

use super::drawing::{DrawnLayers, areas::LayerAreas, groups::AreaKind};
use crate::{
    FloatOrPercent, ObjectOptions, SliceError,
    geometry::{ClipperError, CoordinateScale, ExPolygon},
    project_slice::{
        parameters::SlicingParameters,
        perimeters::{
            classic::shortest_path::chain_and_reorder_entities_with_optional_start,
            flow::{SupportFlows, build_nonbridging_flow},
            types::Flow,
        },
        support_material::{
            install_layers::InstalledSupportLayer, toolpath_parameters::ToolpathParameters,
        },
    },
};

pub(super) struct ToolpathInput<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) parameters: &'a ToolpathParameters,
    pub(super) slicing: &'a SlicingParameters,
    pub(super) flows: &'a SupportFlows,
    pub(super) support_width: f64,
    pub(super) brim_flow: Flow,
    pub(super) first_object_slices: &'a [ExPolygon],
    pub(super) raft_layers: usize,
    pub(super) scale: CoordinateScale,
}

pub(super) fn generate(
    drawn: DrawnLayers,
    input: &ToolpathInput<'_>,
) -> Result<Vec<InstalledSupportLayer>, SliceError> {
    let mut output = drawn
        .layers
        .iter()
        .enumerate()
        .map(|(id, layer)| InstalledSupportLayer {
            id,
            // `add_tree_support_layer` (PrintObject.cpp:1073-1077) marks
            // classic-tree layers stInnerTree.
            inner_tree: true,
            interface_id: 0,
            print_z: layer.print_z,
            height: layer.height,
            slices: Vec::new(),
            support_islands: Vec::new(),
            tree_base_areas: None,
            support_fills: Vec::new(),
        })
        .collect::<Vec<_>>();
    if output.is_empty() {
        return Ok(output);
    }
    raft::fill_prefix(&mut output, &drawn.layers, input)?;
    for (layer_id, (source, layer)) in drawn
        .layers
        .iter()
        .zip(&mut output)
        .enumerate()
        .skip(input.raft_layers)
    {
        let support_flow = input.support_flow(source.height)?;
        let interface_flow = input.flows.interface.with_height(source.height as f32)?;
        for group in &source.groups {
            let area = match group.kind {
                AreaKind::Base => &source.base[group.index],
                AreaKind::Roof => &source.roof[group.index],
                AreaKind::Floor => &source.floor[group.index],
                AreaKind::RoofFirst => &source.roof_first[group.index],
            };
            if group.kind == AreaKind::Base {
                base::BaseArea {
                    input,
                    area,
                    group,
                    support_flow,
                    layer_id,
                    print_z: source.print_z,
                    lightning: drawn.lightning.as_ref(),
                }
                .append(&mut layer.support_fills)?;
            } else {
                interfaces::InterfaceArea {
                    input,
                    area,
                    group,
                    support_flow,
                    interface_flow,
                    layer_id,
                }
                .append(&mut layer.support_fills)?;
            }
        }
        chain_and_reorder_entities_with_optional_start(&mut layer.support_fills, None);
    }
    for (layer, source) in output.iter_mut().zip(drawn.layers) {
        layer.slices = source.slices;
        layer.tree_base_areas = Some(source.base);
    }
    Ok(output)
}

impl ToolpathInput<'_> {
    fn support_flow(&self, height: f64) -> Result<Flow, SliceError> {
        let nozzle = self.flows.base.nozzle_diameter;
        let width = if self.support_width <= 0.0 {
            f64::from(nozzle)
        } else {
            self.support_width
        };
        build_nonbridging_flow(FloatOrPercent::Float(width), height as f32, nozzle)
    }

    fn interface_density(&self, bottom: bool) -> f64 {
        let spacing = f64::from(self.flows.interface.spacing);
        let extra = if bottom {
            self.object.support_bottom_interface_spacing.0
        } else {
            self.object.support_interface_spacing.0
        };
        (spacing / (spacing + extra)).min(1.0)
    }

    fn base_angle(&self) -> f32 {
        (self.object.support_angle.0 * std::f64::consts::PI / 180.0) as f32
    }
}

fn geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("tree toolpath geometry: {error:?}"))
}
