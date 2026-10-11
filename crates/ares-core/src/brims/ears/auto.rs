//! Brim.cpp:310-352 and Polygon.cpp:235-317: automatic ear polygons.
use crate::geometry::{ExPolygon, Point, Polygon, simplification};

const POLY_SIDE_COUNT: usize = 24;

pub(super) fn make_auto(
    source: &[ExPolygon],
    size: i64,
    detection_length: i64,
    max_angle: f64,
    outer: bool,
) -> Vec<ExPolygon> {
    if size <= 0 {
        return Vec::new();
    }
    let threshold = (180.0 - max_angle) * std::f64::consts::PI / 180.0;
    let circle: Vec<_> = (0..POLY_SIDE_COUNT)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / POLY_SIDE_COUNT as f64;
            Point::new(
                (size as f64 * angle.cos()).round() as i64,
                (size as f64 * angle.sin()).round() as i64,
            )
        })
        .collect();
    let mut ears = Vec::new();
    for polygon in source {
        let original = polygon.contour().points();
        let decimated = if detection_length > 0 {
            let mut closed = original.to_vec();
            closed.push(original[0]);
            let mut points = simplification::douglas_peucker(&closed, detection_length as f64);
            if points.len() > 4 {
                points.pop();
                points
            } else {
                original.to_vec()
            }
        } else {
            original.to_vec()
        };
        for center in candidates(&decimated, threshold, outer) {
            ears.push(ExPolygon::new(
                Polygon::new(
                    circle
                        .iter()
                        .map(|p| Point::new(p.x() + center.x(), p.y() + center.y()))
                        .collect(),
                ),
                Vec::new(),
            ));
        }
    }
    ears
}

fn candidates(points: &[Point], threshold: f64, outer: bool) -> Vec<Point> {
    let mut point = points[points.len() - 1];
    let mut incoming = vector(points[points.len() - 2], point);
    let cosine = threshold.next_up().cos();
    let mut result = Vec::new();
    for &next in points {
        let outgoing = vector(point, next);
        let cross = incoming.0 * outgoing.1 - incoming.1 * outgoing.0;
        let signed = if outer { cross > 0.0 } else { cross < 0.0 };
        let angle_matches = if threshold > 1e-4 {
            let in_norm = incoming.0.hypot(incoming.1);
            let out_norm = outgoing.0.hypot(outgoing.1);
            (incoming.0 / in_norm) * (outgoing.0 / out_norm)
                + (incoming.1 / in_norm) * (outgoing.1 / out_norm)
                < cosine
        } else {
            true
        };
        if signed && angle_matches {
            result.push(point);
        }
        incoming = outgoing;
        point = next;
    }
    result
}

fn vector(a: Point, b: Point) -> (f64, f64) {
    ((b.x() - a.x()) as f64, (b.y() - a.y()) as f64)
}
