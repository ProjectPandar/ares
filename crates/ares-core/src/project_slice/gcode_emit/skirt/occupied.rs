//! Local occupied hulls and instance shifts (`Print.cpp:2669-2696,2794-2803`).

use crate::{
    SlicerContext,
    geometry::Point,
    project_slice::{
        gcode_emit::{brim::BrimPlan, footprint},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(super) fn hulls(
    traversal: &PreparedPostClassicTraversal,
    support: &[Vec<InstalledSupportLayer>],
    brim: Option<&BrimPlan>,
    layer_limit: usize,
    context: &mut SlicerContext,
) -> Vec<Vec<Point>> {
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
        // Preserve the existing raft compatibility shell until every raft
        // contributor is installed as source-owned support_fills.
        let resolved = &traversal.resolved.objects[object_index];
        if resolved.object.raft_layers.0 > 0 && support[object_index].is_empty() {
            let lslices = object.slices(0).unwrap_or(&[]).to_vec();
            if let Ok(Some(stream)) = crate::project_slice::raft::bridge::build_project_raft(
                &traversal.resolved.views.full,
                &resolved.object,
                object.slicing_parameters(),
                &lslices,
                scale,
            ) {
                for plan in &stream.plans {
                    if let Ok(entities) = crate::project_slice::raft::fills::raft_layer_entities(
                        &plan.polygons,
                        plan.spec,
                        scale,
                        context,
                    ) {
                        for polyline in entities
                            .iter()
                            .flat_map(crate::project_slice::raft::fills::RaftEntity::polylines)
                        {
                            local.extend(polyline.points().iter().copied());
                        }
                    }
                }
            }
        }
        let local_hull = super::convex_hull(&local);
        let source = &traversal.project.objects()[resolved.source_object_index];
        let center = crate::project_slice::raw_intersections::raw_center(source, scale)
            .expect("sliced object has validated raw bounds");
        for instance in source
            .instances()
            .iter()
            .filter(|instance| instance.printable())
        {
            let shift = footprint::instance_shift(instance, center, scale);
            occupied.extend(
                local_hull
                    .iter()
                    .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y())),
            );
        }
        if object_index == 0
            && let Some(brim) = brim
        {
            let (x, y) = footprint::object_origin(source, scale);
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
            hulls.push(super::convex_hull(&occupied));
        }
    }
    hulls
}
