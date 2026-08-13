use super::scale::coord_from_completed;
use crate::geometry::{ClipperError, Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct F64Point {
    pub(super) x: f64,
    pub(super) y: f64,
}

impl F64Point {
    pub(super) const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub(super) fn from_point(point: Point) -> Self {
        Self::new(point.x() as f64, point.y() as f64)
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    fn squared_norm(self) -> f64 {
        self.dot(self)
    }

    fn norm(self) -> f64 {
        self.squared_norm().sqrt()
    }

    fn midpoint(self, other: Self) -> Self {
        self.add(other).scale(0.5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct F64Segment {
    pub(super) a: F64Point,
    pub(super) b: F64Point,
}

impl F64Segment {
    pub(super) const fn new(a: F64Point, b: F64Point) -> Self {
        Self { a, b }
    }

    pub(super) fn from_points(a: Point, b: Point) -> Self {
        Self::new(F64Point::from_point(a), F64Point::from_point(b))
    }

    fn vector(self) -> F64Point {
        self.b.sub(self.a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EuclideanInterval {
    pub(super) start: f64,
    pub(super) end: f64,
}

#[derive(Clone, Copy)]
struct SegmentPoint {
    segment_index: usize,
    t: f64,
    point: F64Point,
}

pub(super) fn clipped_infill_segments(points: &[Point], clip_distance: f64) -> Vec<F64Segment> {
    debug_assert!(points.len() >= 2);
    debug_assert!(clip_distance > 0.0);

    let Some(start) = clip_start(points, clip_distance) else {
        return Vec::new();
    };
    let Some(end) = clip_end(points, clip_distance) else {
        return Vec::new();
    };
    if start.segment_index > end.segment_index
        || (start.segment_index == end.segment_index && start.t >= end.t)
    {
        return Vec::new();
    }

    (start.segment_index..=end.segment_index)
        .map(|index| {
            let a = if index == start.segment_index {
                start.point
            } else {
                F64Point::from_point(points[index])
            };
            let b = if index == end.segment_index {
                end.point
            } else {
                F64Point::from_point(points[index + 1])
            };
            F64Segment::new(a, b)
        })
        .collect()
}

fn clip_start(points: &[Point], mut distance: f64) -> Option<SegmentPoint> {
    let mut previous = F64Point::from_point(points[0]);
    for (index, &point) in points.iter().enumerate().skip(1) {
        let point = F64Point::from_point(point);
        let vector = point.sub(previous);
        let length = vector.norm();
        if length > distance {
            let t = distance / length;
            return Some(SegmentPoint {
                segment_index: index - 1,
                t,
                point: previous.add(vector.scale(t)),
            });
        }
        distance -= length;
        previous = point;
    }
    None
}

fn clip_end(points: &[Point], mut distance: f64) -> Option<SegmentPoint> {
    let mut next = F64Point::from_point(points[points.len() - 1]);
    for index in (0..points.len() - 1).rev() {
        let point = F64Point::from_point(points[index]);
        let vector = point.sub(next);
        let length = vector.norm();
        if length > distance {
            let end_t = distance / length;
            return Some(SegmentPoint {
                segment_index: index,
                t: 1.0 - end_t,
                point: next.add(vector.scale(end_t)),
            });
        }
        distance -= length;
        next = point;
    }
    None
}

pub(super) fn collision_interval_prefiltered(
    boundary: F64Segment,
    infill: F64Segment,
    radius: f64,
    scaled_epsilon: f64,
) -> Option<EuclideanInterval> {
    if fractional_bounds_overlap(boundary, infill, radius + scaled_epsilon) {
        rounded_thick_segment_collision(boundary, infill, radius, scaled_epsilon)
    } else {
        None
    }
}

fn fractional_bounds_overlap(boundary: F64Segment, infill: F64Segment, delta: f64) -> bool {
    !(cpp_max(infill.a.x, infill.b.x) + delta < cpp_min(boundary.a.x, boundary.b.x)
        || cpp_min(infill.a.x, infill.b.x) - delta > cpp_max(boundary.a.x, boundary.b.x)
        || cpp_max(infill.a.y, infill.b.y) + delta < cpp_min(boundary.a.y, boundary.b.y)
        || cpp_min(infill.a.y, infill.b.y) - delta > cpp_max(boundary.a.y, boundary.b.y))
}

pub(super) fn rounded_thick_segment_collision(
    line: F64Segment,
    segment: F64Segment,
    offset: f64,
    scaled_epsilon: f64,
) -> Option<EuclideanInterval> {
    let line_vector = line.vector();
    let line_length_squared = line_vector.squared_norm();
    let segment_vector = segment.vector();
    let segment_length = segment_vector.norm();
    let offset_squared = offset * offset;

    if line_length_squared < scaled_epsilon * scaled_epsilon {
        let line_midpoint = line.a.midpoint(line.b);
        let distance_squared = if segment_length > scaled_epsilon {
            distance_to_segment_squared(segment, line_midpoint)
        } else {
            segment
                .a
                .midpoint(segment.b)
                .sub(line_midpoint)
                .squared_norm()
        };
        return (distance_squared < offset_squared).then_some(EuclideanInterval {
            start: 0.0,
            end: line_length_squared.sqrt(),
        });
    }

    let mut interval = (f64::MAX, -f64::MAX);
    if segment_length > scaled_epsilon {
        extend_circle_interval(&mut interval, segment.a, offset_squared, line, line_vector);
        extend_circle_interval(&mut interval, segment.b, offset_squared, line, line_vector);

        let direction_x = F64Point::new(
            segment_vector.x / segment_length,
            segment_vector.y / segment_length,
        );
        let direction_y = F64Point::new(-direction_x.y, direction_x.x);
        let line_from_segment = line.a.sub(segment.a);
        if let Some(rectangle) = liang_barsky_interval(
            F64Point::new(
                line_from_segment.dot(direction_x),
                line_from_segment.dot(direction_y),
            ),
            F64Point::new(line_vector.dot(direction_x), line_vector.dot(direction_y)),
            F64Point::new(0.0, -offset),
            F64Point::new(segment_length, offset),
        ) {
            extend_interval(&mut interval, rectangle.0, rectangle.1);
        }
    } else {
        // Preserve FillBase.cpp's active short-segment radius-vs-radius-squared quirk.
        extend_circle_interval(
            &mut interval,
            segment.a.midpoint(segment.b),
            offset,
            line,
            line_vector,
        );
    }

    (interval.0 <= interval.1).then(|| {
        let line_length = line_length_squared.sqrt();
        EuclideanInterval {
            start: interval.0 * line_length,
            end: interval.1 * line_length,
        }
    })
}

fn distance_to_segment_squared(segment: F64Segment, point: F64Point) -> f64 {
    let vector = segment.vector();
    let from_start = point.sub(segment.a);
    let length_squared = vector.squared_norm();
    if length_squared == 0.0 {
        return from_start.squared_norm();
    }
    let t = from_start.dot(vector) / length_squared;
    if t <= 0.0 {
        from_start.squared_norm()
    } else if t >= 1.0 {
        point.sub(segment.b).squared_norm()
    } else {
        vector.scale(t).sub(from_start).squared_norm()
    }
}

fn extend_circle_interval(
    interval: &mut (f64, f64),
    center: F64Point,
    radius_squared: f64,
    line: F64Segment,
    line_vector: F64Point,
) {
    let from_center = line.a.sub(center);
    let length_squared = line_vector.squared_norm();
    let a = line_vector.y;
    let b = -line_vector.x;
    let c = -line_vector.y * from_center.x + line_vector.x * from_center.y;
    let x0 = -a * c;
    let y0 = -b * c;
    let discriminant = radius_squared * length_squared - c * c;
    if discriminant < 0.0 {
        return;
    }
    let root = discriminant.sqrt();
    let first = F64Point::new(
        (x0 + b * root) / length_squared,
        (y0 - a * root) / length_squared,
    );
    let second = F64Point::new(
        (x0 - b * root) / length_squared,
        (y0 + a * root) / length_squared,
    );
    let mut start = first.sub(from_center).dot(line_vector) / length_squared;
    let mut end = second.sub(from_center).dot(line_vector) / length_squared;
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    start = cpp_max(start, 0.0);
    end = cpp_min(end, 1.0);
    if start <= end {
        extend_interval(interval, start, end);
    }
}

fn liang_barsky_interval(
    point: F64Point,
    vector: F64Point,
    min: F64Point,
    max: F64Point,
) -> Option<(f64, f64)> {
    let mut interval = (0.0, 1.0);
    (clip_side(&mut interval, -vector.x, -min.x + point.x)
        && clip_side(&mut interval, vector.x, max.x - point.x)
        && clip_side(&mut interval, -vector.y, -min.y + point.y)
        && clip_side(&mut interval, vector.y, max.y - point.y))
    .then_some(interval)
}

fn clip_side(interval: &mut (f64, f64), p: f64, q: f64) -> bool {
    if p == 0.0 {
        return q >= 0.0;
    }
    let ratio = q / p;
    if p < 0.0 {
        if ratio > interval.1 {
            return false;
        }
        if ratio > interval.0 {
            interval.0 = ratio;
        }
    } else {
        if ratio < interval.0 {
            return false;
        }
        if ratio < interval.1 {
            interval.1 = ratio;
        }
    }
    true
}

fn extend_interval(interval: &mut (f64, f64), start: f64, end: f64) {
    interval.0 = cpp_min(interval.0, start);
    interval.1 = cpp_max(interval.1, end);
}

fn cpp_min(left: f64, right: f64) -> f64 {
    if right < left { right } else { left }
}

fn cpp_max(left: f64, right: f64) -> f64 {
    if left < right { right } else { left }
}

pub(super) fn thick_trace_line(
    infill: F64Segment,
    radius: f64,
    negative_perpendicular: bool,
) -> Result<(Point, Point), ClipperError> {
    let direction = infill.vector();
    let length = direction.norm();
    let extension = F64Point::new(direction.x / length * radius, direction.y / length * radius);
    let perpendicular = F64Point::new(-extension.y, extension.x);
    let (start, end) = if negative_perpendicular {
        (
            infill.a.sub(extension).sub(perpendicular),
            infill.b.add(extension).sub(perpendicular),
        )
    } else {
        (
            infill.a.sub(extension).add(perpendicular),
            infill.b.add(extension).add(perpendicular),
        )
    };
    Ok((
        checked_truncating_point(start)?,
        checked_truncating_point(end)?,
    ))
}

fn checked_truncating_point(point: F64Point) -> Result<Point, ClipperError> {
    Ok(Point::new(
        coord_from_completed(point.x)?,
        coord_from_completed(point.y)?,
    ))
}

#[cfg(test)]
mod tests;
