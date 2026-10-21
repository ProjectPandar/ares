//! Chunk-wide object slices and spacing (`AvoidCrossingPerimeters.cpp:511-529`).

use super::schedule::LayerToPrint;
use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

/// The source's leading object layer, otherwise support/raft layer
/// (`GCode.cpp:4559–4591,4664`), installed before per-tool traversal.
pub(super) fn on_first_layer(
    traversal: &PreparedPostClassicTraversal,
    entries: &[LayerToPrint],
    support: &[Vec<crate::project_slice::support_material::InstalledSupportLayer>],
    raft: &Option<super::RaftSchedule>,
) -> bool {
    let entry = entries
        .iter()
        .find(|entry| entry.object_layer.is_some())
        .unwrap_or(&entries[0]);
    if let Some(index) = entry.object_layer {
        let layer = &traversal.objects[entry.object_index].physical_layers()[index];
        layer.id == 0 && (layer.print_z - layer.height).abs() < 1e-4
    } else if let Some(index) = entry.support_layer {
        let layer = &support[entry.object_index][index];
        layer.id == 0 && (layer.print_z - layer.height).abs() < 1e-4
    } else {
        let index = entry.raft_layer.expect("raft entry retains layer index");
        let layer =
            &raft.as_ref().expect("raft entry retains schedule").plans[entry.object_index][index];
        index == 0 && layer.z.bottom_z.abs() < 1e-4
    }
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

/// The EXTERNAL boundary's slice input: every entry's slices at this print
/// z, appended once per print instance and moved to that instance's
/// placement (`get_boundary_external`, `AvoidCrossingPerimeters.cpp:1172-1177`).
/// [`geometry`] stays object-local because brim/skirt/tower consume it;
/// this one is in plate coordinates so the router sees every hole where the
/// object actually sits.
pub(super) fn boundary_slices(
    traversal: &PreparedPostClassicTraversal,
    entries: &[LayerToPrint],
    instances: &[Vec<super::Instance>],
) -> Vec<ExPolygon> {
    let mut slices = Vec::new();
    for entry in entries {
        let Some(index) = entry.object_layer else {
            continue;
        };
        let layer_slices = traversal.objects[entry.object_index]
            .slices(index)
            .expect("object layer retains slices");
        for instance in &instances[entry.object_index] {
            let dx = instance.shift.x();
            let dy = instance.shift.y();
            let translate = |polygon: crate::geometry::Polygon| {
                crate::geometry::Polygon::new(
                    polygon
                        .points()
                        .iter()
                        .map(|point| crate::geometry::Point::new(point.x() + dx, point.y() + dy))
                        .collect(),
                )
            };
            slices.extend(layer_slices.iter().map(|expolygon| {
                let (contour, holes) = expolygon.clone().into_parts();
                ExPolygon::new(
                    translate(contour),
                    holes.into_iter().map(translate).collect(),
                )
            }));
        }
    }
    slices
}
