//! Active movement from `TreeSupport.cpp:69-77,581-628`.

use super::sampling;
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon,
    offset_expolygons, point_in_polygon, union_expolygons,
};

pub(super) fn is_inside(polygon: &ExPolygon, point: Point) -> bool {
    let bounds = BoundingBox::from_expolygon(polygon).expect("tree outline has a contour");
    sampling::contains(bounds, point)
        && polygon.contour().contains(&point)
        && !polygon
            .holes()
            .iter()
            .any(|hole| point_in_polygon(point, hole.points()) == 1)
}

pub(super) fn is_inside_any(polygons: &[ExPolygon], point: Point) -> bool {
    polygons.iter().any(|polygon| is_inside(polygon, point))
}

pub(super) fn distance_squared(point: Point, scale: CoordinateScale) -> f64 {
    let x = scale.unscale(point.x());
    let y = scale.unscale(point.y());
    x * x + y * y
}

pub(super) fn normal(point: Point, magnitude: f64, scale: CoordinateScale) -> Point {
    let length = scale.scaled_delta(distance_squared(point, scale).sqrt());
    if length < scale.scaled_delta(0.0001) {
        return point;
    }
    let ratio = magnitude / length;
    // Native by-value Point selects its rounded non-const multiplication.
    Point::new(
        (point.x() as f64 * ratio).round() as i64,
        (point.y() as f64 * ratio).round() as i64,
    )
}

pub(super) fn move_out(
    polygons: &[ExPolygon],
    from: &mut Point,
    distance: f64,
    max_move_distance: f64,
    scale: CoordinateScale,
) -> Result<bool, ClipperError> {
    let dilated = union_expolygons(
        &offset_expolygons(
            polygons,
            scale.scaled_delta(distance) as f32,
            JoinType::Miter,
            3.0,
        )?,
        &[],
    )?;
    let mut projected = projection_onto(&dilated, *from);
    let outward = Point::new(projected.x() - from.x(), projected.y() - from.y());
    let movement = normal(outward, scale.scaled_delta(max_move_distance), scale);
    let maximum = Point::new(from.x() + movement.x(), from.y() + movement.y());
    if distance_squared(outward, scale) > max_move_distance * max_move_distance {
        projected = maximum;
    }
    if !is_inside_any(&dilated, *from) {
        Ok(true)
    } else if !is_inside_any(polygons, *from) {
        *from = projected;
        Ok(true)
    } else if !is_inside_any(polygons, maximum) {
        *from = maximum;
        Ok(true)
    } else {
        Ok(false)
    }
}

// ExPolygon.cpp:204-223 and Point.cpp:90-128, not point_projection.
pub(super) fn projection_onto(polygons: &[ExPolygon], from: Point) -> Point {
    let mut projected = Point::new(0, 0);
    let mut minimum = f64::MAX;
    for polygon in polygons {
        for contour in std::iter::once(polygon.contour()).chain(polygon.holes()) {
            let point = project_polygon(contour, from);
            let distance = coordinate_distance_squared(from, point);
            if distance < minimum {
                projected = point;
                minimum = distance;
            }
        }
    }
    projected
}

fn project_polygon(polygon: &Polygon, from: Point) -> Point {
    let points = polygon.points();
    let mut projected = points[0];
    if points.len() <= 2 {
        return projected;
    }
    let mut minimum = coordinate_distance_squared(projected, from).sqrt();
    for index in 0..points.len() {
        let point = project_line(points[index], points[(index + 1) % points.len()], from);
        let distance = coordinate_distance_squared(point, from).sqrt();
        if distance < minimum {
            projected = point;
            minimum = distance;
        }
    }
    projected
}

fn project_line(first: Point, second: Point, from: Point) -> Point {
    if first == second {
        return first;
    }
    let x = (second.x() - first.x()) as f64;
    let y = (second.y() - first.y()) as f64;
    let theta =
        ((second.x() - from.x()) as f64 * x + (second.y() - from.y()) as f64 * y) / (x * x + y * y);
    if (0.0..=1.0).contains(&theta) {
        Point::new(
            (theta * first.x() as f64 + (1.0 - theta) * second.x() as f64) as i64,
            (theta * first.y() as f64 + (1.0 - theta) * second.y() as f64) as i64,
        )
    } else if coordinate_distance_squared(first, from) < coordinate_distance_squared(second, from) {
        first
    } else {
        second
    }
}

fn coordinate_distance_squared(first: Point, second: Point) -> f64 {
    let x = (first.x() - second.x()) as f64;
    let y = (first.y() - second.y()) as f64;
    x * x + y * y
}
