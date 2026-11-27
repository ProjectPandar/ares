//! `GCode.cpp::collect_layers_to_print` for the single belt object.

use super::LayerToPrint;
use crate::project_slice::{
    belt::brim::BeltBrimPlan, island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

pub(super) fn schedule(
    (merged, raw_zs): (&mut Vec<(f64, LayerToPrint)>, &mut Vec<f64>),
    (objects, support): (&[Vec<OrderedExtrusionLayer>], &[Vec<InstalledSupportLayer>]),
    brim: Option<&BeltBrimPlan>,
    traversal: &PreparedPostClassicTraversal,
) {
    merged.retain(|(_, entry)| {
        let object = entry.object_index;
        let prints_object = entry.object_layer.is_some_and(|layer| {
            brim.is_some_and(|brim| !brim.by_layer[layer].paths.is_empty())
                || objects[entry.object_index][layer]
                    .islands
                    .iter()
                    .any(|island| !island.entities.is_empty())
        });
        let prints_support = entry
            .support_layer
            .is_some_and(|layer| !support[object][layer].support_fills.is_empty());
        if prints_object {
            raw_zs.push(
                traversal.objects[object].physical_layers()[entry.object_layer.unwrap()].print_z,
            );
        }
        if prints_support {
            raw_zs.push(support[object][entry.support_layer.unwrap()].print_z);
        }
        prints_object || prints_support
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
        raw_zs.push(band.print_z);
    }
}
