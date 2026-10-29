//! FillAdaptive.cpp:654-668,670-778,1034-1044: finite hook collision queries.
use super::geometry::{normalize, segment_line_intersection, shift_from_thick_line};
use super::{Joint, create_offset_line, pair::other_hook};
use crate::geometry::{ExPolygon, Line, LineDistanceTree, Point};

pub(super) struct HookParams<'a> {
    pub(super) source: &'a [Line],
    pub(super) segments: &'a [Line],
    pub(super) index: LineDistanceTree<'a>,
    pub(super) offset: f64,
    pub(super) length: f64,
    pub(super) maximum_length: f64,
    pub(super) trim: f64,
    pub(super) epsilon: f64,
}

// mk_rtree_seg uses float endpoints, including the contour and all holes.
pub(super) fn collision_segments(source: &[Line], boundary: &ExPolygon) -> Vec<Line> {
    let mut segments = source.iter().copied().map(float_line).collect::<Vec<_>>();
    for polygon in std::iter::once(boundary.contour()).chain(boundary.holes()) {
        let mut previous = *polygon.points().last().unwrap();
        for &point in polygon.points() {
            segments.push(float_line(Line::new(previous, point)));
            previous = point;
        }
    }
    segments
}

fn float_line(line: Line) -> Line {
    let point = |p: Point| Point::new(p.x() as f32 as i64, p.y() as f32 as i64);
    Line::new(point(line.a), point(line.b))
}

pub(super) fn add_hook(joint: &mut Joint, working: &mut [Vec<Point>], params: &HookParams<'_>) {
    let crossing = params.source[joint.closest_line];
    let (dx, dy) = normalize(
        crossing.b.x() - crossing.a.x(),
        crossing.b.y() - crossing.a.y(),
    )
    .unwrap();
    let start = segment_line_intersection(
        params.source[joint.intersect_line],
        create_offset_line(crossing.a, crossing.b, crossing, joint.left, params.offset),
    )
    .unwrap();
    let extension = params.length + 1.16 * params.trim;
    let vector = Point::new((extension * dx) as i64, (extension * dy) as i64);
    let forward_query = Line::new(
        start,
        Point::new(start.x() + vector.x(), start.y() + vector.y()),
    );
    let other = other_hook(joint, &working[joint.intersect_pl]);
    let forward = max_hook_length(params, joint.intersect_line, forward_query, (dx, dy), other);
    let backward = if forward < params.length - params.epsilon {
        let query = Line::new(
            start,
            Point::new(start.x() - vector.x(), start.y() - vector.y()),
        );
        max_hook_length(params, joint.intersect_line, query, (-dx, -dy), other)
    } else {
        0.0
    };
    // :767-768 selects backward on a tie, then casts displacement before addition.
    let length = if forward > backward {
        forward
    } else {
        -backward
    };
    let end = Point::new(
        start.x() + (length * dx) as i64,
        start.y() + (length * dy) as i64,
    );
    let points = &mut working[joint.intersect_pl];
    if joint.front {
        points[0] = start;
        points.insert(0, end);
    } else {
        *points.last_mut().unwrap() = start;
        points.push(end);
    }
}

fn max_hook_length(
    params: &HookParams<'_>,
    excluded: usize,
    query: Line,
    (dx, dy): (f64, f64),
    other: Option<Line>,
) -> f64 {
    let mut maximum = params.length;
    // The accelerated finite-segment query replaces bgi::intersects; only
    // the minimum distance is consumed, so tree visitation order is immaterial.
    for (_, index) in params.index.intersections_sorted(float_line(query)) {
        if index == excluded {
            continue;
        }
        let segment = params.segments[index];
        let sx = (segment.b.x() - segment.a.x()) as f64;
        let sy = (segment.b.y() - segment.a.y()) as f64;
        let ax = (segment.a.x() - query.a.x()) as f64;
        let ay = (segment.a.y() - query.a.y()) as f64;
        let mut distance = (ax * sy - ay * sx) / (dx * sy - dy * sx);
        // :742-744: contour/hole segments are not thick infill centerlines.
        if index < params.source.len() {
            distance -= shift_from_thick_line(dx, dy, &segment, params.trim);
        }
        if distance < maximum {
            maximum = distance;
        }
    }
    if let Some(line) = other
        && let Some(point) = segment_line_intersection(line, query)
    {
        let distance = (point.x() - query.a.x()) as f64 * dx
            + (point.y() - query.a.y()) as f64 * dy
            - shift_from_thick_line(dx, dy, &line, params.trim);
        maximum = maximum.min(distance);
    }
    maximum.max(0.0)
}
