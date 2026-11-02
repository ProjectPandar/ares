//! Local occupied hulls and instance shifts (`Print.cpp:2669-2696,2794-2803`).

use crate::{
    geometry::Point,
    project_slice::{
        gcode_emit::{brim::BrimPlan, footprint},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(super) fn items(
    traversal: &PreparedPostClassicTraversal,
    support: &[Vec<InstalledSupportLayer>],
    brim: &[Option<BrimPlan>],
    layer_limit: usize,
    raft: Option<&super::super::layers::RaftSchedule>,
) -> Vec<((usize, usize), Vec<Point>)> {
    let skirt_height_z = traversal
        .objects
        .iter()
        .map(|object| {
            object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .object
                .as_parts()
                .0
                .plan
                .layers
                .iter()
                .take(layer_limit)
                .next_back()
                .map_or(0.0, |layer| layer.print_z)
        })
        .fold(0.0_f64, f64::max);
    let scale = traversal.scale;
    let mut items = Vec::new();
    for (object_index, object) in traversal.objects.iter().enumerate() {
        let input = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let (compensation, layers) = input.object.as_parts();
        let mut local = Vec::new();
        for (layer, _) in layers
            .iter()
            .zip(&compensation.plan.layers)
            .take_while(|(_, planned)| planned.print_z <= skirt_height_z)
        {
            local.extend(
                layer
                    .iter()
                    .flat_map(|expolygon| expolygon.contour().points().iter().copied()),
            );
        }
        for entity in support[object_index]
            .iter()
            .take_while(|layer| layer.print_z <= skirt_height_z)
            .flat_map(|layer| &layer.support_fills)
        {
            entity.for_each_path(&mut |path| {
                local.extend(
                    path.polyline
                        .points
                        .iter()
                        .map(|point| Point::new(point.x, point.y)),
                );
            });
        }
        let (resolved, group) = traversal.print_object_config(object_index);
        if let Some(raft) = raft {
            local.extend(raft.skirt_points(object_index));
        }
        let local_hull = super::convex_hull(&local);
        let source = &traversal.project.objects()[resolved.source_object_index];
        let center = crate::project_slice::centering::center(source, group, scale)
            .expect("sliced object has validated raw bounds");
        // Print.cpp:4188-4202: one item per instance — the object hull
        // translated by this instance's shift plus this instance's
        // brim-area contours (m_objectBrimAreasByInstance keyed
        // ObjectInstanceID), never an owner-folded set.
        let brim = brim[object_index].as_ref();
        for (copy, &member) in group.instance_indices.iter().enumerate() {
            let shift = footprint::instance_shift(&source.instances()[member], center, scale);
            let mut occupied = local_hull
                .iter()
                .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y()))
                .collect::<Vec<_>>();
            if let Some(brim) = brim
                && let Some(entry) = brim.instance_areas.iter().find(|area| area.copy == copy)
            {
                occupied.extend(
                    entry
                        .area
                        .iter()
                        .flat_map(|expolygon| expolygon.contour().points().iter().copied()),
                );
            }
            if occupied.len() >= 3 {
                items.push(((object_index, copy), super::convex_hull(&occupied)));
            }
        }
    }
    items
}
