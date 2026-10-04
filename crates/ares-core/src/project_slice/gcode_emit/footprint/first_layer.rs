//! Plate-frame bounds (`Print::first_layer_islands`, `GCode.cpp:2930-2939`).

use super::{FirstLayerBounds, instance_shift, object_origin};
use crate::{
    geometry::{CoordinateScale, Point, Polygon},
    project_slice::{
        gcode_emit::{brim::BrimPlan, skirt::SkirtPlan},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(in crate::project_slice::gcode_emit) fn first_layer_bounds(
    traversal: &PreparedPostClassicTraversal,
    skirt: Option<&SkirtPlan>,
    brim: Option<&BrimPlan>,
    tower_corners: &[Point],
) -> Option<FirstLayerBounds> {
    let scale = traversal.scale;
    let mut bounds = None;
    visit_object_contours(traversal, |polygon, shift| {
        include_polygon_bounds(polygon, shift, scale, &mut bounds);
    });
    if let Some(skirt) = skirt {
        include_bounds(skirt.covered_bounds(scale), &mut bounds);
    }
    if let Some(brim) = brim {
        let source =
            &traversal.project.objects()[traversal.resolved.objects[0].source_object_index];
        let (x, y) = object_origin(source, scale);
        include_bounds(
            brim.covered_bounds(scale)
                .map(|(min_x, min_y, max_x, max_y)| (min_x + x, min_y + y, max_x + x, max_y + y)),
            &mut bounds,
        );
    }
    for point in tower_corners {
        let x = scale.unscale(point.x());
        let y = scale.unscale(point.y());
        include_bounds(Some((x, y, x, y)), &mut bounds);
    }
    bounds.map(|(min_x, min_y, max_x, max_y)| (min_x, min_y, max_x - min_x, max_y - min_y))
}

/// Print.cpp:3154–3164: prefer printed brim/skirt outlines, otherwise islands;
/// append the tower only after deciding whether object contours are needed.
pub(in crate::project_slice::gcode_emit) fn first_layer_convex_hull(
    traversal: &PreparedPostClassicTraversal,
    skirt: Option<&SkirtPlan>,
    brim: Option<&BrimPlan>,
    tower_corners: &[Point],
) -> Vec<Point> {
    let mut points = Vec::new();
    if let Some(brim) = brim {
        let source =
            &traversal.project.objects()[traversal.resolved.objects[0].source_object_index];
        let (x, y) = object_origin(source, traversal.scale);
        let shift = Point::new(
            (x / traversal.scale.factor()).round() as i64,
            (y / traversal.scale.factor()).round() as i64,
        );
        points.extend(
            brim.covered_hull()
                .iter()
                .map(|p| Point::new(p.x() + shift.x(), p.y() + shift.y())),
        );
    }
    if let Some(skirt) = skirt {
        points.extend(skirt.covered_points());
    }
    if points.is_empty() {
        visit_object_contours(traversal, |polygon, shift| {
            points.extend(
                polygon
                    .points()
                    .iter()
                    .map(|p| Point::new(p.x() + shift.x(), p.y() + shift.y())),
            );
        });
    }
    points.extend_from_slice(tower_corners);
    super::convex_hull(&points)
}

/// `PrintObject::get_first_layer_bbox` (Print.cpp:2126–2148) uses only
/// the first PrintInstance without brim; the brim bbox covers all copies
/// (Brim.cpp:908–934). GCode.cpp:2999–3006 merges those object boxes.
pub(in crate::project_slice::gcode_emit) fn first_layer_object_center(
    traversal: &PreparedPostClassicTraversal,
    instances: &[Vec<super::super::layers::Instance>],
    brim: Option<&BrimPlan>,
) -> [f64; 2] {
    let scale = traversal.scale;
    let mut bounds = None;
    for (prepared, copies) in traversal.objects.iter().zip(instances) {
        let object = &prepared
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let (source_index, _) = object.identity();
        let config = &traversal
            .resolved
            .objects
            .iter()
            .find(|object| object.source_object_index == source_index)
            .expect("print object retains its resolved configuration")
            .object;
        let has_brim = config.raft_layers.0 <= 0
            && (config.brim_type == crate::ProcessBrimType::AutoBrim
                || (config.brim_type != crate::ProcessBrimType::NoBrim
                    && config.brim_width.0 > 0.0));
        let count = if has_brim { copies.len() } else { 1 };
        for copy in copies.iter().take(count) {
            for polygon in object.object.as_parts().1.iter().take(1).flatten() {
                include_polygon_bounds(polygon.contour(), copy.shift, scale, &mut bounds);
            }
        }
    }
    if let Some(brim) = brim
        && traversal.resolved.objects[0].object.raft_layers.0 <= 0
        && let Some((min_x, min_y, max_x, max_y)) = brim.covered_bounds(scale)
    {
        for copy in &instances[0] {
            let x = scale.unscale(copy.shift.x());
            let y = scale.unscale(copy.shift.y());
            include_bounds(
                Some((min_x + x, min_y + y, max_x + x, max_y + y)),
                &mut bounds,
            );
        }
    }
    bounds.map_or([0.0; 2], |(min_x, min_y, max_x, max_y)| {
        [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0]
    })
}

fn visit_object_contours(
    traversal: &PreparedPostClassicTraversal,
    mut visit: impl FnMut(&Polygon, Point),
) {
    let scale = traversal.scale;
    for (object, resolved) in traversal.objects.iter().zip(&traversal.resolved.objects) {
        let layers = object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object
            .object
            .as_parts()
            .1;
        let source = &traversal.project.objects()[resolved.source_object_index];
        let center = crate::project_slice::raw_intersections::raw_center(source, scale)
            .expect("sliced object has validated raw bounds");
        for instance in source
            .instances()
            .iter()
            .filter(|instance| instance.printable())
        {
            let shift = instance_shift(instance, center, scale);
            for polygon in layers.iter().take(1).flatten() {
                visit(polygon.contour(), shift);
            }
        }
    }
}

fn include_bounds(additional: Option<FirstLayerBounds>, bounds: &mut Option<FirstLayerBounds>) {
    let Some((min_x, min_y, max_x, max_y)) = additional else {
        return;
    };
    *bounds = Some(match *bounds {
        Some((old_min_x, old_min_y, old_max_x, old_max_y)) => (
            old_min_x.min(min_x),
            old_min_y.min(min_y),
            old_max_x.max(max_x),
            old_max_y.max(max_y),
        ),
        None => (min_x, min_y, max_x, max_y),
    });
}

fn include_polygon_bounds(
    polygon: &Polygon,
    shift: Point,
    scale: CoordinateScale,
    bounds: &mut Option<FirstLayerBounds>,
) {
    for point in polygon.points() {
        let x = scale.unscale(point.x() + shift.x());
        let y = scale.unscale(point.y() + shift.y());
        *bounds = Some(match *bounds {
            Some((min_x, min_y, max_x, max_y)) => {
                (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
            }
            None => (x, y, x, y),
        });
    }
}
