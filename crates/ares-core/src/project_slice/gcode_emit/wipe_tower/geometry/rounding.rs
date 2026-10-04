//! `WipeTower2.cpp::rounding_polygon` and `Circle.cpp` arc angle calculation.

use std::f64::consts::{PI, TAU};

use crate::geometry::{CoordinateScale, Point, Polygon};

pub(super) fn round_polygon(polygon: &Polygon, scale: CoordinateScale) -> Polygon {
    let points = polygon.points();
    let mut rounded = Vec::with_capacity(points.len() * 2);
    for (index, &point) in points.iter().enumerate() {
        let a = unscaled(points[(index + points.len() - 1) % points.len()], scale);
        let b = unscaled(point, scale);
        let c = unscaled(points[(index + 1) % points.len()], scale);
        let ab_length = norm([b[0] - a[0], b[1] - a[1]]);
        let bc_length = norm([c[0] - b[0], c[1] - b[1]]);
        let ab = [(b[0] - a[0]) / ab_length, (b[1] - a[1]) / ab_length];
        let bc = [(c[0] - b[0]) / bc_length, (c[1] - b[1]) / bc_length];
        let cosine = ((ab[0] * bc[0] + ab[1] * bc[1]) as f32).clamp(-1.0, 1.0);
        if f64::from(cosine.abs()) >= (30.0 / 180.0 * PI).cos().abs() {
            rounded.push(point);
            continue;
        }
        let counterclockwise = ab[0] * bc[1] - ab[1] * bc[0] > 0.0;
        let distance = 2.0_f64.min(ab_length / 2.1).min(bc_length / 2.1) as f32;
        let left = [
            b[0] - ab[0] * f64::from(distance),
            b[1] - ab[1] * f64::from(distance),
        ];
        let right = [
            b[0] + bc[0] * f64::from(distance),
            b[1] + bc[1] * f64::from(distance),
        ];
        let half_angle = cosine.acos() / 2.0;
        let chord = [right[0] - left[0], right[1] - left[1]];
        let chord_length = norm(chord);
        let sign = if counterclockwise { 1.0 } else { -1.0 };
        let direction = [
            -chord[1] / chord_length * sign,
            chord[0] / chord_length * sign,
        ];
        // The unqualified C++ sin promotes the float angle to double.
        let center_distance = f64::from(distance) / f64::from(half_angle).sin();
        let center = [
            b[0] + direction[0] * center_distance,
            b[1] + direction[1] * center_distance,
        ];
        let radius = (norm([left[0] - center[0], left[1] - center[1]]) / scale.factor()) as i64;
        append_arc(
            &mut rounded,
            scaled(center, scale),
            radius,
            [scaled(left, scale), scaled(right, scale)],
            counterclockwise,
        );
        rounded.push(scaled(right, scale));
    }
    rounded.dedup();
    Polygon::new(rounded)
}

fn append_arc(
    output: &mut Vec<Point>,
    center: Point,
    radius: i64,
    [left, right]: [Point; 2],
    counterclockwise: bool,
) {
    let (start, sweep) = if radius == 0 || left == center || right == center || left == right {
        (0.0, 0.0)
    } else {
        let start = polar_angle(left, center);
        let mut sweep = polar_angle(right, center) - start;
        if sweep < 0.0 && counterclockwise {
            sweep += TAU;
        } else if sweep > 0.0 && !counterclockwise {
            sweep -= TAU;
        }
        (start, sweep)
    };
    for index in 0..20 {
        let mut angle = (start + f64::from(index as f32 / 20.0) * sweep) as f32;
        if f64::from(angle) > TAU {
            angle = (f64::from(angle) - TAU) as f32;
        } else if angle < 0.0 {
            angle = (f64::from(angle) + TAU) as f32;
        }
        output.push(Point::new(
            center.x() + (radius as f64 * f64::from(angle.cos())).round() as i64,
            center.y() + (radius as f64 * f64::from(angle.sin())).round() as i64,
        ));
    }
}

fn polar_angle(point: Point, center: Point) -> f64 {
    let angle = ((point.y() - center.y()) as f64).atan2((point.x() - center.x()) as f64);
    if angle < 0.0 { angle + TAU } else { angle }
}

fn norm(vector: [f64; 2]) -> f64 {
    (vector[0] * vector[0] + vector[1] * vector[1]).sqrt()
}

fn unscaled(point: Point, scale: CoordinateScale) -> [f64; 2] {
    [scale.unscale(point.x()), scale.unscale(point.y())]
}

fn scaled(point: [f64; 2], scale: CoordinateScale) -> Point {
    Point::new(
        (point[0] / scale.factor()) as i64,
        (point[1] / scale.factor()) as i64,
    )
}
