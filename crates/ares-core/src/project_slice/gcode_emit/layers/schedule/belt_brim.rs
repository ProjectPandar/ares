//! `GCode.cpp::collect_layers_to_print` for the single belt object.

use super::LayerToPrint;
use crate::project_slice::{belt::brim::BeltBrimPlan, island_print_order::OrderedExtrusionLayer};
pub(super) fn schedule(
    merged: &mut Vec<(f64, LayerToPrint)>,
    objects: &[Vec<OrderedExtrusionLayer>],
    brim: Option<&BeltBrimPlan>,
) {
    merged.retain(|(_, entry)| {
        entry.object_layer.is_some_and(|layer| {
            brim.is_some_and(|brim| !brim.by_layer[layer].paths.is_empty())
                || objects[entry.object_index][layer]
                    .islands
                    .iter()
                    .any(|island| !island.entities.is_empty())
        })
    });
    let Some(brim) = brim else {
        return;
    };
    for (index, band) in brim.prologue.iter().enumerate() {
        merged.push((
            band.print_z,
            LayerToPrint {
                object_index: 0,
                object_layer: None,
                support_layer: None,
                raft_layer: None,
                belt_prologue: Some(index),
            },
        ));
    }
}
