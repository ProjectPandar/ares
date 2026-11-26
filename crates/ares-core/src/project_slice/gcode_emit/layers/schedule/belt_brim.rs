//! `GCode.cpp::collect_layers_to_print` and `ToolOrdering.cpp::collect_extruders`.

use super::{EPSILON, LayerToPrint};
use crate::project_slice::{
    belt::brim::BeltBrimPlan,
    gcode_emit::tool_ordering::{LayerTools, ToolOrdering},
    island_print_order::OrderedExtrusionLayer,
};

// Admission gives the belt object one instance and filament, and prologue
// bands are strictly below all object layers.
pub(super) fn schedule(
    merged: &mut Vec<(f64, LayerToPrint)>,
    objects: &[Vec<OrderedExtrusionLayer>],
    ordering: &mut ToolOrdering,
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
