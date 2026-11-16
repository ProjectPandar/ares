//! GCode.cpp:7238–7362 per-filament instances_to_print and instance_visits.
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::tool_buckets::Entry;
use crate::{
    geometry::Point,
    project_slice::{
        island_print_order::visits::{self, InstanceInput, InstanceVisit},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(super) struct InstanceTour {
    pub(super) instances: Vec<Entry>,
    pub(super) visits: Vec<InstanceVisit>,
}

pub(super) fn plan(
    mut instances: Vec<Entry>,
    traversal: &PreparedPostClassicTraversal,
    copy_filter: Option<usize>,
) -> InstanceTour {
    let print = &traversal.resolved.views.full.process.print;
    if print.print_sequence == crate::ProcessPrintSequence::ByObject
        || copy_filter.is_some()
        || print.print_order == crate::ProcessPrintOrder::AsObjectList
    {
        let instances = super::instances::order(instances, traversal);
        let visits = (0..instances.len())
            .map(|instance_idx| InstanceVisit {
                instance_idx,
                islands: Vec::new(),
                first_visit: true,
            })
            .collect();
        return InstanceTour { instances, visits };
    }
    // Native objects_by_extruder/layer index, then actual instance index.
    instances.sort_by_key(|entry| (entry.source.object_index, entry.instance.copy));
    let inputs = instances
        .iter()
        .map(|entry| InstanceInput {
            shift: entry.instance.shift,
            slices: entry.source.object_layer.map(|layer| {
                traversal.objects[entry.source.object_index]
                    .slices(layer)
                    .unwrap_or(&[])
            }),
            occupied: occupied(entry),
        })
        .collect::<Vec<_>>();
    let visits = visits::produce(&inputs, wipe_start(traversal), traversal.scale);
    InstanceTour { instances, visits }
}

// Config.hpp:676–682 get_at uses element zero beyond the vector's length.
// Point(double,double) rounds; the source does not scale these tower values.
fn wipe_start(traversal: &PreparedPostClassicTraversal) -> Point {
    let print = &traversal.resolved.views.full.project.print;
    let plate = traversal.project.current_plate_index();
    let xs = &print.wipe_tower_x.0;
    let ys = &print.wipe_tower_y.0;
    Point::new(
        xs.get(plate).unwrap_or(&xs[0]).0.round() as i64,
        ys.get(plate).unwrap_or(&ys[0]).0.round() as i64,
    )
}

// Native by_region precedes the wipe/normal split. Recombine presence only,
// without cloning or moving any of the actual per-filament extrusion data.
fn occupied(entry: &Entry) -> Vec<bool> {
    let count = entry
        .body
        .as_ref()
        .or(entry.wipe_body.as_ref())
        .map_or(0, |layer| layer.islands.len());
    let mut occupied = vec![false; count];
    for layer in [entry.body.as_ref(), entry.wipe_body.as_ref()]
        .into_iter()
        .flatten()
    {
        for (index, island) in layer.islands.iter().enumerate() {
            occupied[index] |= !island.entities.is_empty();
        }
    }
    occupied
}
