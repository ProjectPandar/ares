//! `TreeSupport.cpp:2512-2536::intersects_contour` vertex search.

use super::super::movement::distance_squared;
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polygon, intersection_pl};
use crate::project_slice::support_material::layer_input::to_polygons;

pub(super) struct ContourContact {
    pub(super) intersects: bool,
    pub(super) on_hole: Point,
    pub(super) on_area: Point,
    pub(super) far_on_hole: Point,
}

pub(super) fn intersects_contour(
    hole: &Polygon,
    area: &ExPolygon,
    scale: CoordinateScale,
) -> Result<ContourContact, ClipperError> {
    let mut contact = ContourContact {
        intersects: false,
        on_hole: Point::new(0, 0),
        on_area: Point::new(0, 0),
        far_on_hole: Point::new(0, 0),
    };
    if intersection_pl(
        &to_polygons(std::slice::from_ref(area)),
        std::slice::from_ref(hole),
    )?
    .is_empty()
    {
        return Ok(contact);
    }
    let mut minimum = f32::MAX;
    let mut maximum = 0.0_f32;
    for &from in hole.points() {
        for contour in std::iter::once(area.contour()).chain(area.holes()) {
            let candidate = closest_vertex(contour, from);
            let distance = distance_squared(
                Point::new(candidate.x() - from.x(), candidate.y() - from.y()),
                scale,
            );
            if distance < f64::from(minimum) {
                minimum = distance as f32;
                contact.on_hole = from;
                contact.on_area = candidate;
            }
            if distance > f64::from(maximum) {
                maximum = distance as f32;
                contact.far_on_hole = from;
            }
            if distance < f64::from(0.01_f32) {
                contact.intersects = true;
                return Ok(contact);
            }
        }
    }
    Ok(contact)
}

fn closest_vertex(contour: &Polygon, from: Point) -> Point {
    let mut closest = contour.points()[0];
    let mut minimum = coordinate_distance(closest, from);
    for &point in &contour.points()[1..] {
        let distance = coordinate_distance(point, from);
        if distance < minimum {
            closest = point;
            minimum = distance;
        }
    }
    closest
}

fn coordinate_distance(left: Point, right: Point) -> f64 {
    let x = (left.x() - right.x()) as f64;
    let y = (left.y() - right.y()) as f64;
    (x * x + y * y).sqrt()
}
