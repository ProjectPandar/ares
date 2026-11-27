//! `GCode.cpp::collect_layers_to_print` and `ToolOrdering.cpp::collect_extruders`.

use super::{EPSILON, LayerToPrint};
use crate::project_slice::{
    belt::brim::BeltBrimPlan,
    gcode_emit::tool_ordering::{LayerTools, ToolOrdering},
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

// Admission gives the belt object one instance and filament, and prologue
// bands are strictly below all object layers.
pub(super) fn schedule(
    (merged, raw_zs): (&mut Vec<(f64, LayerToPrint)>, &mut Vec<f64>),
    (objects, support): (&[Vec<OrderedExtrusionLayer>], &[Vec<InstalledSupportLayer>]),
    ordering: &mut ToolOrdering,
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
    for (z, entry) in merged.iter() {
        if entry
            .object_layer
            .is_some_and(|layer| !brim.by_layer[layer].paths.is_empty())
        {
            register(ordering, *z);
        }
    }
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
        register(ordering, band.print_z);
        raw_zs.push(band.print_z);
    }
}

fn register(ordering: &mut ToolOrdering, print_z: f64) {
    let layers = &mut ordering.layers;
    let index = layers.partition_point(|layer| layer.print_z < print_z - EPSILON);
    if let Some(layer) = layers
        .get_mut(index)
        .filter(|layer| (layer.print_z - print_z).abs() < EPSILON)
    {
        if layer.extruders.is_empty() {
            layer.extruders.push(0);
        }
    } else {
        let mut layer = LayerTools::default();
        layer.print_z = print_z;
        layer.extruders.push(0);
        layers.insert(index, layer);
    }
}
