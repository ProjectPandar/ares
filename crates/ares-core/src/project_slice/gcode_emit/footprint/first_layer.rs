//! Plate-frame bounds (`Print::first_layer_islands`, `GCode.cpp:2930-2939`).

use super::FirstLayerBounds;
use crate::{
    geometry::{CoordinateScale, Point},
    project_slice::{
        gcode_emit::{brim::BrimPlan, layers::Schedule, skirt::SkirtPlan},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
    },
};

pub(in crate::project_slice::gcode_emit) struct ObjectFootprint {
    pub bounds: [f64; 4],
    pub area: f32,
    pub layer_height: f32,
}

/// `Print.cpp:3197–3218`: merge the first layer's slice boxes in plate
/// coordinates, unless make_brim recorded a defined object box.
pub(in crate::project_slice::gcode_emit) fn export_object_footprint(
    traversal: &PreparedPostClassicTraversal,
    schedule: &Schedule,
    brim: Option<&BrimPlan>,
    index: usize,
) -> Option<ObjectFootprint> {
    let prepared = &traversal.objects[index];
    let [copy] = schedule.instances[index].as_slice() else {
        return None;
    };
    let object = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let first_slice = object.object.as_parts().1.first()?;
    let first_record = prepared.records.iter().flatten().next()?;
    let (plate_x, plate_y) = traversal.project.plate_origin();
    let scale = traversal.scale;
    let mut bounds = None;
    for polygon in first_slice {
        include_points(polygon.contour().points(), copy.shift, scale, &mut bounds);
    }
    let (x0, y0, x1, y1) = brim
        .and_then(|brim| brim.covered_bounds(scale))
        .or(bounds)?;
    let scaled_area = first_slice.iter().fold(0_f32, |area, polygon| {
        (f64::from(area) + polygon.area()) as f32
    });
    Some(ObjectFootprint {
        bounds: [x0 - plate_x, y0 - plate_y, x1 - plate_x, y1 - plate_y],
        area: (f64::from(scaled_area) * scale.factor().powi(2)) as f32,
        layer_height: first_record.layer_height as f32,
    })
}

pub(in crate::project_slice::gcode_emit) fn first_layer_bounds(
    traversal: &PreparedPostClassicTraversal,
    skirt: Option<&SkirtPlan>,
    brim: &[Option<BrimPlan>],
    tower_corners: &[Point],
    islands: &[Point],
) -> Option<FirstLayerBounds> {
    let scale = traversal.scale;
    let mut bounds = None;
    include_points(islands, Point::new(0, 0), scale, &mut bounds);
    if let Some(skirt) = skirt {
        include_bounds(skirt.covered_bounds(scale), &mut bounds);
    }
    for brim in brim.iter().flatten() {
        let (x, y) = brim.occupied_origin();
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
    brim: &[Option<BrimPlan>],
    tower_corners: &[Point],
    islands: &[Point],
) -> Vec<Point> {
    let mut points = Vec::new();
    for brim in brim.iter().flatten() {
        let (x, y) = brim.occupied_origin();
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
        points.extend_from_slice(islands);
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
    brim: &[Option<BrimPlan>],
    support_spacing: &[Vec<Point>],
) -> [f64; 2] {
    let scale = traversal.scale;
    let mut bounds = None;
    for (index, (prepared, copies)) in traversal.objects.iter().zip(instances).enumerate() {
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
                include_points(polygon.contour().points(), copy.shift, scale, &mut bounds);
            }
            if has_brim {
                include_points(&support_spacing[index], copy.shift, scale, &mut bounds);
            }
        }
    }
    for (index, brim) in brim.iter().enumerate() {
        if traversal.print_object_config(index).0.object.raft_layers.0 <= 0
            && let Some(brim) = brim
            && let Some((min_x, min_y, max_x, max_y)) = brim.covered_bounds(scale)
        {
            // Brim.cpp:914-935: the plate-frame owner box already includes
            // every instance.
            include_bounds(Some((min_x, min_y, max_x, max_y)), &mut bounds);
        }
    }
    bounds.map_or([0.0; 2], |(min_x, min_y, max_x, max_y)| {
        [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0]
    })
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

fn include_points(
    points: &[Point],
    shift: Point,
    scale: CoordinateScale,
    bounds: &mut Option<FirstLayerBounds>,
) {
    for point in points {
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
