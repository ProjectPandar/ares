//! Chunk-wide object slices and spacing (`AvoidCrossingPerimeters.cpp:511-529`).

use super::schedule::LayerToPrint;
use crate::{
    geometry::ExPolygon,
    project_slice::{
        island_print_order::OrderedExtrusionLayer,
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

// `ToolOrdering.cpp::collect_extruders`: support geometry alone owns no tool.
pub(super) fn has_tools(
    entries: &[LayerToPrint],
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    raft: &[Vec<OrderedExtrusionLayer>],
) -> bool {
    entries.iter().any(|entry| {
        let object = entry.object_index;
        entry.object_layer.is_some_and(|layer| {
            objects[object][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty())
        }) || entry.raft_layer.is_some_and(|layer| {
            raft[object][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty())
        }) || entry.support_layer.is_some_and(|layer| {
            support[object][layer].support_fills.iter().any(|entity| {
                let role = entity.role();
                matches!(
                    role,
                    crate::ExtrusionRole::SupportMaterial
                        | crate::ExtrusionRole::SupportMaterialInterface
                        | crate::ExtrusionRole::SupportTransition
                )
            })
        })
    })
}

pub(super) fn geometry(
    traversal: &PreparedPostClassicTraversal,
    entries: &[LayerToPrint],
) -> (Vec<ExPolygon>, f64) {
    let mut slices = Vec::new();
    let mut spacing = 0.0f32;
    let mut count = 0;
    for entry in entries {
        if let Some(index) = entry.object_layer {
            let object = &traversal.objects[entry.object_index];
            let layer_slices = object.slices(index).expect("object layer retains slices");
            slices.extend_from_slice(layer_slices);
            if !layer_slices.is_empty() {
                spacing += object
                    .perimeter_spacing(index)
                    .expect("object layer retains perimeter flow");
                count += 1;
            }
        }
    }
    if count > 0 {
        spacing /= count as f32;
    }
    (slices, f64::from(spacing))
}
