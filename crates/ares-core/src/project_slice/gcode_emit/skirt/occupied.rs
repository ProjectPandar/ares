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
) -> Vec<(usize, Vec<Point>)> {
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
    let mut hulls = Vec::new();
    for (object_index, object) in traversal.objects.iter().enumerate() {
        let mut occupied = Vec::new();
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
        for &member in &group.instance_indices {
            let shift = footprint::instance_shift(&source.instances()[member], center, scale);
            occupied.extend(
                local_hull
                    .iter()
                    .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y())),
            );
        }
        if let Some(brim) = &brim[object_index] {
            let (x, y) = brim.occupied_origin(traversal, object_index);
            let shift = Point::new(
                (x / scale.factor()).round() as i64,
                (y / scale.factor()).round() as i64,
            );
            occupied.extend(
                brim.covered_hull()
                    .iter()
                    .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y())),
            );
        }
        if occupied.len() >= 3 {
            hulls.push((object_index, super::convex_hull(&occupied)));
        }
    }
    hulls
}
