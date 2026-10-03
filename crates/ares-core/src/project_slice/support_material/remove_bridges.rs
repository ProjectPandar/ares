//! `SupportMaterialInternal::remove_bridges_from_contacts` and the geometric
//! `Polyline::is_straight`/`extend_start`/`extend_end` contracts it uses.

use super::{layer_input::to_polygons, overhangs::contains};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
    difference_open_polylines, difference_polygons_paths,
    difference_polygons_paths_with_safety_offset, offset_open_paths, offset_paths,
    union_polygons_paths,
};

pub(super) struct RegionBridges<'a> {
    pub(super) perimeters: &'a [Polyline],
    pub(super) outer_nozzle_scaled: f32,
    pub(super) bridge_width_scaled: f64,
    pub(super) bridge_spacing_scaled: f64,
    // Native stBottomBridge surfaces with bridge_angle >= 0.
    pub(super) bottom_bridge_surfaces: &'a [ExPolygon],
    pub(super) unsupported_edges: &'a [Polyline],
}

pub(super) fn remove_bridges_from_contacts(
    bridges: &RegionBridges<'_>,
    lower_slices: &[ExPolygon],
    external_width_scaled: f32,
    contacts: &mut Vec<Polygon>,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let grown_lower = offset_paths(
        &to_polygons(lower_slices),
        0.5 * bridges.outer_nozzle_scaled,
        JoinType::Square,
        0.0,
    )?;
    let perimeters = difference_open_polylines(bridges.perimeters, &grown_lower)?;
    let width = (0.5
        * bridges
            .bridge_width_scaled
            .max(bridges.bridge_spacing_scaled)) as f32
        + (0.001 / scale.factor()) as f32;
    let mut bridging_polygons = Vec::new();
    for mut perimeter in perimeters {
        if !is_straight(&perimeter) {
            continue;
        }
        extend_start(&mut perimeter, f64::from(external_width_scaled));
        extend_end(&mut perimeter, f64::from(external_width_scaled));
        if !supported_endpoint(perimeter.front().unwrap(), lower_slices)
            || !supported_endpoint(perimeter.back().unwrap(), lower_slices)
        {
            continue;
        }
        bridging_polygons.extend(offset_open_paths(
            &[Polygon::new(perimeter.into_points())],
            width,
            JoinType::Square,
            0.0,
        )?);
    }
    let mut bridging_polygons = union_polygons_paths(&bridging_polygons)?;
    bridging_polygons.extend(to_polygons(bridges.bottom_bridge_surfaces));
    let unsupported = bridges
        .unsupported_edges
        .iter()
        .map(|edge| Polygon::new(edge.points().to_vec()))
        .collect::<Vec<_>>();
    let unsupported = offset_open_paths(
        &unsupported,
        (1.2 / scale.factor()) as f32,
        JoinType::Square,
        0.0,
    )?;
    let bridging_polygons = difference_polygons_paths(&bridging_polygons, &unsupported)?;
    *contacts = difference_polygons_paths_with_safety_offset(contacts, &bridging_polygons)?;
    Ok(())
}

pub(super) fn supported_endpoint(point: Point, slices: &[ExPolygon]) -> bool {
    slices.iter().any(|polygon| {
        let bounds = BoundingBox::from_expolygon(polygon).unwrap();
        point.x() >= bounds.min().x()
            && point.x() <= bounds.max().x()
            && point.y() >= bounds.min().y()
            && point.y() <= bounds.max().y()
            && contains(polygon, point)
    })
}

fn direction(first: Point, last: Point) -> f64 {
    let angle = ((last.y() - first.y()) as f64).atan2((last.x() - first.x()) as f64);
    if (angle - std::f64::consts::PI).abs() < 1e-4 {
        0.0
    } else if angle < 0.0 {
        angle + std::f64::consts::PI
    } else {
        angle
    }
}

pub(super) fn is_straight(polyline: &Polyline) -> bool {
    let base_direction = direction(polyline.front().unwrap(), polyline.back().unwrap());
    polyline.points().windows(2).all(|points| {
        let difference = (direction(points[0], points[1]) - base_direction).abs();
        difference < 1e-4 || (difference - std::f64::consts::PI).abs() < 1e-4
    })
}

pub(super) fn extend_end(polyline: &mut Polyline, distance: f64) {
    let points = polyline.points_mut();
    let last = points[points.len() - 1];
    let previous = points[points.len() - 2];
    let x = (last.x() - previous.x()) as f64;
    let y = (last.y() - previous.y()) as f64;
    let length = (x * x + y * y).sqrt();
    points.push(Point::new(
        last.x() + (x / length * distance) as i64,
        last.y() + (y / length * distance) as i64,
    ));
}

pub(super) fn extend_start(polyline: &mut Polyline, distance: f64) {
    polyline.reverse();
    extend_end(polyline, distance);
    polyline.reverse();
}
