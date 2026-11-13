//! `GCode.hpp:105–130`: tower integration priming-overlap input.
use super::super::{
    layers::Schedule,
    skirt::SkirtPlan,
    wipe_tower::{Data, PrintExtents},
};
use crate::project_slice::{
    island_print_order::PreparedPostIslandPrintOrder,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn priming_overlaps(
    data: &Data,
    traversal: &PreparedPostClassicTraversal,
    prepared: &PreparedPostIslandPrintOrder,
    schedule: &Schedule,
    skirt: Option<&SkirtPlan>,
) -> bool {
    let first_z = schedule.merged[0].0;
    let second_z = schedule
        .merged
        .iter()
        .map(|entry| entry.0)
        .find(|&z| z > first_z)
        .unwrap_or(first_z);
    traversal
        .resolved
        .views
        .full
        .process
        .gcode
        .single_extruder_multi_material_priming
        .0
        && data.priming_overlaps(PrintExtents {
            traversal,
            objects: &prepared.objects,
            instances: &schedule.instances,
            skirt,
            max_print_z: second_z + 1e-4,
        })
}
