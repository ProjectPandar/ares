//! Brim.cpp:36-45,573-581: instance areas collected under one PrintObject ID.
use super::bed_area::translate;
use crate::{
    geometry::ExPolygon,
    project_slice::{
        gcode_emit::footprint, perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(super) fn expand(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    area: &[ExPolygon],
    forbidden: &[ExPolygon],
    islands: &[ExPolygon],
) -> (Vec<ExPolygon>, Vec<ExPolygon>, Vec<ExPolygon>) {
    let (resolved, group) = traversal.print_object_config(object_index);
    let object = &traversal.project.objects()[resolved.source_object_index];
    let center = crate::project_slice::centering::center(object, group, traversal.scale)
        .expect("sliced object retains its raw center");
    let mut owner_area = Vec::new();
    let mut owner_forbidden = Vec::new();
    let mut owner_islands = Vec::new();
    for &member in &group.instance_indices {
        let shift = footprint::instance_shift(&object.instances()[member], center, traversal.scale);
        owner_area.extend(translate(area.to_vec(), shift));
        owner_forbidden.extend(translate(forbidden.to_vec(), shift));
        owner_islands.extend(translate(islands.to_vec(), shift));
    }
    (owner_area, owner_forbidden, owner_islands)
}
