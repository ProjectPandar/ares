//! `TreeSupport.cpp:2651-3170` descending group/merge/drop/prune driver.

use super::{
    data::TreeSupportData,
    drop_settings::DropSettings,
    dropping::DropPass,
    grouping::group_nodes,
    heights::LayerHeightData,
    line_cut::LineCutCache,
    merging::MergePass,
    nodes::{NodeArena, NodeId},
    pruning::prune_unsupported,
};
use crate::{ObjectOptions, ProcessSupportStyle, geometry::ClipperError};

pub(super) struct NodeDropping<'a> {
    pub(super) arena: &'a mut NodeArena,
    pub(super) layers: &'a mut [Vec<NodeId>],
    pub(super) heights: &'a [LayerHeightData],
    pub(super) data: &'a mut TreeSupportData,
    pub(super) settings: &'a DropSettings,
    pub(super) object: &'a ObjectOptions,
    pub(super) style: ProcessSupportStyle,
}

impl NodeDropping<'_> {
    pub(super) fn run(self, context: &mut crate::SlicerContext) -> Result<(), ClipperError> {
        let Self {
            arena,
            layers,
            heights,
            data,
            settings,
            object,
            style,
        } = self;
        if heights.is_empty() {
            return Ok(());
        }
        precalculate(arena, layers, heights, data, settings)?;
        for layer in (1..layers.len()).rev() {
            if layers[layer].is_empty() {
                continue;
            }
            // `TreeSupport.cpp:2955`.
            context.set_status(
                60 + (10.0 * (1.0 - layer as f32 / layers.len() as f32)) as i32,
                format_args!("Generating support"),
                0,
            );
            let current_height = &heights[layer];
            let next_height = &heights[layer - 1];
            let mut cuts = LineCutCache::new(super::super::super::layer_input::to_polygons(
                &data.outlines[current_height.object_layer],
            ));
            let (groups, mut unsupported) = group_nodes(
                arena,
                &layers[layer],
                &data.outlines_below[current_height.object_layer],
                object.support_on_build_plate_only.0,
                settings.scale,
            );
            for (group_index, group) in groups.iter().enumerate() {
                MergePass {
                    group_index,
                    arena,
                    data,
                    settings,
                    next_height,
                    next_nodes: &mut layers[layer - 1],
                }
                .run(group)?;
                DropPass {
                    group_index,
                    arena,
                    data,
                    settings,
                    current_height,
                    next_height,
                    next_nodes: &mut layers[layer - 1],
                    unsupported: &mut unsupported,
                    cuts: &mut cuts,
                    buildplate_only: object.support_on_build_plate_only.0,
                    is_slim: style == ProcessSupportStyle::TreeSlim,
                    is_strong: style == ProcessSupportStyle::TreeStrong,
                }
                .run(group)?;
            }
            prune_unsupported(arena, layers, unsupported);
        }
        Ok(())
    }
}

fn precalculate(
    arena: &NodeArena,
    layers: &[Vec<NodeId>],
    heights: &[LayerHeightData],
    data: &mut TreeSupportData,
    settings: &DropSettings,
) -> Result<(), ClipperError> {
    let mut distances = vec![Vec::new(); layers.len()];
    let mut radii = vec![Vec::new(); layers.len()];
    for layer in (1..layers.len()).rev() {
        distances[layer].extend(
            layers[layer]
                .iter()
                .map(|&id| arena.nodes[id].distance_mm_to_top),
        );
        sort_unique(&mut distances[layer]);
        if layer > 1 {
            let (lower, current) = distances.split_at_mut(layer);
            lower[layer - 1].extend(
                current[0]
                    .iter()
                    .map(|distance| distance + heights[layer].height),
            );
        }
        radii[layer].extend(
            distances[layer]
                .iter()
                .map(|&distance| settings.radius(distance)),
        );
        sort_unique(&mut radii[layer]);
    }
    for layer in 0..layers.len() - 1 {
        for &radius in &radii[layer] {
            let object_layer = heights[layer].object_layer;
            data.avoidance(radius, object_layer)?;
            data.collision(0.0, object_layer)?;
            data.collision(radius, object_layer)?;
        }
    }
    Ok(())
}

fn sort_unique(values: &mut Vec<f64>) {
    values.sort_unstable_by(f64::total_cmp);
    values.dedup_by(|first, second| *first == *second);
}
