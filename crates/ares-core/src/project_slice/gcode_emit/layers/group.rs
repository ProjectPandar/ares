//! Chunk-wide object slices and spacing (`AvoidCrossingPerimeters.cpp:511-529`).

use super::schedule::LayerToPrint;
use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

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
