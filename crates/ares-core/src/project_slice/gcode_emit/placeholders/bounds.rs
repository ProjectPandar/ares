use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

use super::super::{
    footprint::FirstLayerBounds,
    value::{self, Value},
};

/// `in_head_wrap_detect_zone` (`GCode.cpp:2958-2968`): true when the
/// first-layer projection of any object intersects the printer's head wrap
/// detect zone polygon.
pub(super) fn insert_head_wrap_detect_zone(
    config: &mut value::Config,
    traversal: &PreparedPostClassicTraversal,
) {
    use crate::geometry::{Polygon, intersection_polygons_paths};

    let zone_mm = &traversal
        .resolved
        .views
        .full
        .printer
        .remaining
        .head_wrap_detect_zone;
    let scale = traversal.scale;
    let zone: Vec<crate::geometry::Point> = zone_mm
        .0
        .iter()
        .filter_map(|point| {
            let x = scale.checked_scale(point.x)?;
            let y = scale.checked_scale(point.y)?;
            Some(crate::geometry::Point::new(x, y))
        })
        .collect();
    let in_zone = if zone.len() < 3 {
        false
    } else {
        let zone_polygon = Polygon::new(zone.clone());
        let mut hit = false;
        for object in &traversal.objects {
            let layers = &object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .object
                .as_parts()
                .1;
            let Some(first) = layers.first() else {
                continue;
            };
            let points: Vec<crate::geometry::Point> = first
                .iter()
                .flat_map(|expolygon| expolygon.contour().points().iter().copied())
                .collect();
            if points.len() < 3 {
                continue;
            }
            let hull = crate::project_slice::gcode_emit::skirt::convex_hull(&points);
            if hull.len() < 3 {
                continue;
            }
            let clipped = intersection_polygons_paths(
                &[Polygon::new(hull)],
                std::slice::from_ref(&zone_polygon),
            );
            if matches!(clipped, Ok(intersection) if !intersection.is_empty()) {
                hit = true;
                break;
            }
        }
        hit
    };
    config.insert("in_head_wrap_detect_zone", Value::option_bool(in_zone));
}

pub(super) fn insert_first_layer_bounds(
    config: &mut value::Config,
    bounds: Option<FirstLayerBounds>,
) {
    let Some((min_x, min_y, size_x, size_y)) = bounds else {
        return;
    };
    let point = |x, y| Value::List(vec![Value::number(x), Value::number(y)]);
    config.insert("first_layer_print_min", point(min_x, min_y));
    config.insert(
        "first_layer_print_max",
        point(min_x + size_x, min_y + size_y),
    );
    config.insert("first_layer_print_size", point(size_x, size_y));
    config.insert(
        "first_layer_center_no_wipe_tower",
        point(min_x + 0.5 * size_x, min_y + 0.5 * size_y),
    );
}

/// `print_bed_min`/`print_bed_max`/`print_bed_size` from the printable-area
/// bounding box (`GCode.cpp:2908-2912`).
pub(super) fn insert_adaptive_bed_mesh(config: &mut value::Config) {
    let Some(mesh_min) = point_value(config, "bed_mesh_min") else {
        return;
    };
    let Some(mesh_max) = point_value(config, "bed_mesh_max") else {
        return;
    };
    let bounds_min = point_value(config, "first_layer_print_min").unwrap_or(mesh_min);
    let bounds_max = point_value(config, "first_layer_print_max").unwrap_or(mesh_max);
    let margin = config
        .get("adaptive_bed_mesh_margin")
        .and_then(Value::as_number)
        .unwrap_or(0.0);
    let minimum = [
        mesh_min[0].max(bounds_min[0] - margin),
        mesh_min[1].max(bounds_min[1] - margin),
    ];
    let maximum = [
        mesh_max[0].min(bounds_max[0] + margin),
        mesh_max[1].min(bounds_max[1] + margin),
    ];
    let distance = point_value(config, "bed_mesh_probe_distance").unwrap_or([50.0, 50.0]);
    let mut probe_count = [
        ((maximum[0] - minimum[0]) / distance[0].max(1.0)).ceil() + 1.0,
        ((maximum[1] - minimum[1]) / distance[1].max(1.0)).ceil() + 1.0,
    ];
    probe_count[0] = probe_count[0].max(3.0);
    probe_count[1] = probe_count[1].max(3.0);
    let algorithm = if probe_count[0] * probe_count[1] <= 6.0 {
        "lagrange"
    } else {
        if config.get("gcode_flavor").map(Value::as_string).as_deref() == Some("klipper") {
            probe_count[0] = probe_count[0].max(4.0);
            probe_count[1] = probe_count[1].max(4.0);
        }
        "bicubic"
    };
    let point = |coordinates: [f64; 2]| {
        Value::List(
            coordinates
                .into_iter()
                .map(Value::number)
                .collect::<Vec<_>>(),
        )
    };
    config.insert("adaptive_bed_mesh_min", point(minimum));
    config.insert("adaptive_bed_mesh_max", point(maximum));
    config.insert("bed_mesh_probe_count", point(probe_count));
    config.insert("bed_mesh_algo", Value::String(algorithm.to_owned()));
}

fn point_value(config: &value::Config, key: &str) -> Option<[f64; 2]> {
    let value = config.get(key)?;
    Some([value.index(0)?.as_number()?, value.index(1)?.as_number()?])
}

pub(super) fn insert_print_bed_bounds(config: &mut value::Config) {
    let Some(area) = config.get("printable_area") else {
        return;
    };
    let corners = area
        .iter_list()
        .filter_map(|corner| {
            let text = corner.as_string();
            let (x, y) = text.split_once('x')?;
            Some((x.parse::<f64>().ok()?, y.parse::<f64>().ok()?))
        })
        .collect::<Vec<_>>();
    if corners.len() < 3 {
        return;
    }
    let min_x = corners
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::INFINITY, f64::min);
    let min_y = corners
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    let max_x = corners
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = corners
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    let point = |x: f64, y: f64| Value::List(vec![Value::number(x), Value::number(y)]);
    config.insert("print_bed_min", point(min_x, min_y));
    config.insert("print_bed_max", point(max_x, max_y));
    config.insert("print_bed_size", point(max_x - min_x, max_y - min_y));
}
