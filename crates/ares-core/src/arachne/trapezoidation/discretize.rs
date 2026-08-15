use crate::geometry::{Point, Polygon};

use super::index::PolygonSegmentIndex;

#[expect(
    clippy::too_many_arguments,
    reason = "matches the pinned VoronoiUtils point-point primitive"
)]
pub(super) fn discretize_point_point(
    left: Point,
    right: Point,
    start: Point,
    end: Point,
    step_size: i64,
    transitioning_angle: f64,
) -> Vec<Point> {
    let d = distance(left, right);
    let middle = Point::new((left.x() + right.x()) / 2, (left.y() + right.y()) / 2);
    let axis = Point::new(-(right.y() - left.y()), right.x() - left.x());
    let axis_length = distance(Point::new(0, 0), axis);
    let projected_x = |point: Point| {
        dot(
            Point::new(point.x() - middle.x(), point.y() - middle.y()),
            axis,
        ) / axis_length
    };
    let start_x = projected_x(start);
    let end_x = projected_x(end);
    let bound = (0.5 / ((std::f64::consts::PI - transitioning_angle) * 0.5).tan()) as f32;
    let mut marking_start_x = (-(d as f32) * bound) as i64;
    let mut marking_end_x = (d as f32 * bound) as i64;
    let mut marking_start = Point::new(
        middle.x() + (axis.x() as i128 * marking_start_x as i128 / axis_length as i128) as i64,
        middle.y() + (axis.y() as i128 * marking_start_x as i128 / axis_length as i128) as i64,
    );
    let mut marking_end = Point::new(
        middle.x() + (axis.x() as i128 * marking_end_x as i128 / axis_length as i128) as i64,
        middle.y() + (axis.y() as i128 * marking_end_x as i128 / axis_length as i128) as i64,
    );
    let direction = if start_x > end_x { -1 } else { 1 };
    if direction < 0 {
        std::mem::swap(&mut marking_start, &mut marking_end);
        std::mem::swap(&mut marking_start_x, &mut marking_end_x);
    }
    let mut add_start = marking_start_x * direction > start_x * direction;
    let mut add_end = marking_end_x * direction > start_x * direction;
    let ab = Point::new(end.x() - start.x(), end.y() - start.y());
    let size = distance(Point::new(0, 0), ab);
    let mut step_count = (size + step_size / 2) / step_size;
    if step_count % 2 == 1 {
        step_count += 1;
    }
    let mut result = vec![start];
    for step in 1..step_count {
        let here = Point::new(
            start.x() + (ab.x() as i128 * step as i128 / step_count as i128) as i64,
            start.y() + (ab.y() as i128 * step as i128 / step_count as i128) as i64,
        );
        let x = projected_x(here);
        if add_start && marking_start_x * direction < x * direction {
            result.push(marking_start);
            add_start = false;
        }
        if add_end && marking_end_x * direction < x * direction {
            result.push(marking_end);
            add_end = false;
        }
        result.push(here);
    }
    if add_end && marking_end_x * direction < end_x * direction {
        result.push(marking_end);
    }
    result.push(end);
    result
}

#[expect(
    clippy::too_many_arguments,
    reason = "matches the pinned VoronoiUtils parabola primitive"
)]
pub(super) fn discretize_parabola(
    source_point: Point,
    source_segment: PolygonSegmentIndex,
    polygons: &[Polygon],
    start: Point,
    end: Point,
    step_size: i64,
    transitioning_angle: f64,
) -> Vec<Point> {
    let a = source_segment.from(polygons);
    let b = source_segment.to(polygons);
    let ab = sub(b, a);
    let ab_size = distance(a, b);
    let sx = dot(sub(start, a), ab) / ab_size;
    let ex = dot(sub(end, a), ab) / ab_size;
    let px = dot(sub(source_point, a), ab) / ab_size;
    let projection = project_infinite(a, b, source_point);
    let ppxx = sub(projection, source_point);
    let d = distance(Point::new(0, 0), ppxx);
    if d == 0 {
        return vec![start, end];
    }
    let perpendicular = Point::new(-ppxx.y(), ppxx.x());
    let perpendicular_length = distance(Point::new(0, 0), perpendicular) as f64;
    let cosine = perpendicular.x() as f64 / perpendicular_length;
    let sine = perpendicular.y() as f64 / perpendicular_length;
    let marking_bound = (transitioning_angle * 0.5).atan();
    let mut marking_start_x = (-marking_bound * d as f64) as i64;
    let mut marking_end_x = (marking_bound * d as f64) as i64;
    let marking_height = marking_start_x * marking_start_x / (2 * d) + d / 2;
    let mut marking_start = add(
        rotate(Point::new(marking_start_x, marking_height), cosine, sine),
        projection,
    );
    let mut marking_end = add(
        rotate(Point::new(marking_end_x, marking_height), cosine, sine),
        projection,
    );
    let direction = if sx > ex { -1 } else { 1 };
    if direction < 0 {
        std::mem::swap(&mut marking_start, &mut marking_end);
        std::mem::swap(&mut marking_start_x, &mut marking_end_x);
    }
    let mut add_start = marking_start_x * direction > (sx - px) * direction
        && marking_start_x * direction < (ex - px) * direction;
    let mut add_end = marking_end_x * direction > (sx - px) * direction
        && marking_end_x * direction < (ex - px) * direction;
    let apex = add(rotate(Point::new(0, d / 2), cosine, sine), projection);
    let mut add_apex = (sx - px) * direction < 0 && (ex - px) * direction > 0;
    let step_count = ((ex - sx).unsigned_abs() as f64 / step_size as f64).round() as i64;
    let mut result = vec![start];
    for step in 1..step_count {
        let x = sx + (ex - sx) * step / step_count - px;
        let y = x * x / (2 * d) + d / 2;
        if add_start && marking_start_x * direction < x * direction {
            result.push(marking_start);
            add_start = false;
        }
        if add_apex && x * direction > 0 {
            result.push(apex);
            add_apex = false;
        }
        if add_end && marking_end_x * direction < x * direction {
            result.push(marking_end);
            add_end = false;
        }
        result.push(add(rotate(Point::new(x, y), cosine, sine), projection));
    }
    if add_apex {
        result.push(apex);
    }
    if add_end {
        result.push(marking_end);
    }
    result.push(end);
    result
}

fn project_infinite(a: Point, b: Point, point: Point) -> Point {
    let dx = (b.x() - a.x()) as f64;
    let dy = (b.y() - a.y()) as f64;
    let t =
        ((point.x() - a.x()) as f64 * dx + (point.y() - a.y()) as f64 * dy) / (dx * dx + dy * dy);
    Point::new(
        (a.x() as f64 + t * dx) as i64,
        (a.y() as f64 + t * dy) as i64,
    )
}

fn distance(a: Point, b: Point) -> i64 {
    let dx = (b.x() - a.x()) as f64;
    let dy = (b.y() - a.y()) as f64;
    (dx * dx + dy * dy).sqrt() as i64
}

fn dot(a: Point, b: Point) -> i64 {
    (a.x() as i128 * b.x() as i128 + a.y() as i128 * b.y() as i128) as i64
}

const fn sub(a: Point, b: Point) -> Point {
    Point::new(a.x() - b.x(), a.y() - b.y())
}

const fn add(a: Point, b: Point) -> Point {
    Point::new(a.x() + b.x(), a.y() + b.y())
}

fn rotate(point: Point, cosine: f64, sine: f64) -> Point {
    Point::new(
        (point.x() as f64 * cosine - point.y() as f64 * sine).round() as i64,
        (point.x() as f64 * sine + point.y() as f64 * cosine).round() as i64,
    )
}
