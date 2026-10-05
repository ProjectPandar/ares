//! `TreeSupport3D.cpp::organic_draw_branches` mesh/contact aggregation prefix.

mod aggregation;
mod clipping;
mod contacts;
mod finalize;
mod mesh_slices;
mod traversal;

use std::collections::VecDeque;

use super::super::model_volumes::TreeModelVolumes;
use super::{
    TreeSettings,
    branch_mesh::{BranchExtrusion, IndexedTriangleSet},
    interface_placer::InterfacePlacer,
    smoothing::organic_smooth_branches_avoid_collisions,
    support_element::SupportElement,
};
use crate::{
    SliceError,
    geometry::{ClipperError, CoordinateScale, Polygon},
};

pub(super) use aggregation::Slice;
pub(super) use finalize::allocate_layers;
use traversal::{Branch, Node};

/// Layer-Z inputs to `TreeSupport3D.cpp::organic_draw_branches`.
pub(super) struct OrganicLayerContext<'a> {
    pub(super) config: &'a TreeSettings,
    pub(super) slicing: &'a crate::project_slice::parameters::SlicingParameters,
}

struct Drawing<'a, 'v> {
    graph: &'a [VecDeque<SupportElement>],
    volumes: &'a mut TreeModelVolumes<'v>,
    extrusion: BranchExtrusion<'a>,
}

struct BranchSlices {
    begin: usize,
    polygons: Vec<Vec<Polygon>>,
    contacts: Vec<Vec<Polygon>>,
}

pub(super) fn draw_branch_slices(
    volumes: &mut TreeModelVolumes<'_>,
    graph: &mut [VecDeque<SupportElement>],
    interface: &mut InterfacePlacer<'_>,
    scale: CoordinateScale,
) -> Result<Vec<Slice>, SliceError> {
    let config = interface.config;
    organic_smooth_branches_avoid_collisions(volumes, config, interface.slicing, graph, scale);
    volumes.clear_all_but_object_collision();
    let trees = traversal::collect(graph);
    let mut drawing = Drawing {
        graph,
        volumes,
        extrusion: BranchExtrusion {
            config,
            slicing: interface.slicing,
            scale,
        },
    };
    let mut mesh = IndexedTriangleSet::default();
    let mut slice_z = Vec::new();
    let mut sliced_trees = Vec::with_capacity(trees.len());
    for branches in trees {
        let mut tree = aggregation::TreeSlices::default();
        for branch in branches {
            let Some(mut slices) = drawing.slice_branch(&branch, &mut mesh, &mut slice_z)? else {
                continue;
            };
            drawing.recover_roofs(interface, &branch, &mut slices);
            while slices.polygons.last().is_some_and(Vec::is_empty) {
                slices.polygons.pop();
            }
            tree.insert(slices);
        }
        tree.union_branches().map_err(geometry_error)?;
        sliced_trees.push(tree);
    }
    Ok(aggregation::combine_trees(sliced_trees))
}

impl Drawing<'_, '_> {
    fn config(&self) -> &TreeSettings {
        self.extrusion.config
    }

    fn element(&self, node: Node) -> &SupportElement {
        &self.graph[node.layer][node.index]
    }

    fn recover_roofs(
        &self,
        interface: &mut InterfacePlacer<'_>,
        branch: &Branch,
        slices: &mut BranchSlices,
    ) {
        if interface.counts.top == 0 {
            return;
        }
        for &node in branch.path.iter().rev() {
            let state = &self.element(node).state;
            if state.missing_roof_layers == 0 {
                break;
            }
            let layer = state.layer_index;
            if layer < slices.begin || layer - slices.begin >= slices.polygons.len() {
                continue;
            }
            let polygons = &mut slices.polygons[layer - slices.begin];
            if polygons.is_empty() || state.roof_recovery_depth as usize > interface.counts.top {
                continue;
            }
            interface.add_roof(
                std::mem::take(polygons),
                layer,
                state.roof_recovery_depth as usize,
            );
        }
    }
}

fn geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("tree support geometry failed: {error:?}"))
}
