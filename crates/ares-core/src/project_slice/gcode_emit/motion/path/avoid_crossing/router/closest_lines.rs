//! Closest boundary hits and endpoint extension (`AvoidCrossingPerimeters.cpp:150–294`).
use crate::geometry::fixed_gcc_sort_by;

use super::{Boundary, Coord, Intersection, Point, distance};

#[derive(Clone, Copy)]
pub(super) struct ClosestLine {
    pub(super) point: Point,
    contour: usize,
    segment: usize,
}

/// `get_closest_lines_in_radius` (:150-165) over the boundary grid.
pub(super) fn closest_lines_in_radius(
    boundary: &Boundary,
    center: Point,
    radius: f32,
) -> Vec<ClosestLine> {
    let radius_coordinate = radius.round() as Coord;
    let query_min = Point::new(
        center.x().saturating_sub(radius_coordinate),
        center.y().saturating_sub(radius_coordinate),
    );
    let query_max = Point::new(
        center.x().saturating_add(radius_coordinate),
        center.y().saturating_add(radius_coordinate),
    );
    let mut lines = Vec::new();
    let mut seen = std::collections::HashSet::new();
    boundary
        .grid
        .visit_cells_intersecting_box(query_min, query_max, |_, _, edges| {
            for &edge in edges {
                if !seen.insert((edge.contour_index, edge.segment_index)) {
                    continue;
                }
                let (segment_start, segment_end) = boundary.grid.segment(edge);
                let (closest, distance_squared) =
                    project_on_segment(center, segment_start, segment_end);
                if distance_squared <= f64::from(radius * radius) {
                    lines.push(ClosestLine {
                        point: closest,
                        contour: edge.contour_index,
                        segment: edge.segment_index,
                    });
                }
            }
            true
        });
    fixed_gcc_sort_by(&mut lines, |left, right| {
        squared_distance(center, left.point) < squared_distance(center, right.point)
    });
    lines
}

fn project_on_segment(point: Point, start: Point, end: Point) -> (Point, f64) {
    let segment = (
        end.x() as f64 - start.x() as f64,
        end.y() as f64 - start.y() as f64,
    );
    let from_start = (
        point.x() as f64 - start.x() as f64,
        point.y() as f64 - start.y() as f64,
    );
    let length_squared = segment.0 * segment.0 + segment.1 * segment.1;
    if length_squared == 0.0 {
        return (
            start,
            from_start.0 * from_start.0 + from_start.1 * from_start.1,
        );
    }
    let t = (from_start.0 * segment.0 + from_start.1 * segment.1) / length_squared;
    if t <= 0.0 {
        return (
            start,
            from_start.0 * from_start.0 + from_start.1 * from_start.1,
        );
    }
    let to_end = (
        point.x() as f64 - end.x() as f64,
        point.y() as f64 - end.y() as f64,
    );
    if t >= 1.0 {
        return (end, to_end.0 * to_end.0 + to_end.1 * to_end.1);
    }
    let projected_vector = (t * segment.0, t * segment.1);
    let projected = (
        start.x() as f64 + projected_vector.0,
        start.y() as f64 + projected_vector.1,
    );
    let distance_vector = (
        projected_vector.0 - from_start.0,
        projected_vector.1 - from_start.1,
    );
    (
        Point::new(projected.0 as Coord, projected.1 as Coord),
        distance_vector.0 * distance_vector.0 + distance_vector.1 * distance_vector.1,
    )
}

fn squared_distance(first: Point, second: Point) -> f64 {
    let dx = second.x() as f64 - first.x() as f64;
    let dy = second.y() as f64 - first.y() as f64;
    dx * dx + dy * dy
}

/// `extend_for_closest_lines` (:171-290): when the offset swallowed the
/// intersections, splice closest-line hits at the travel endpoints.
pub(super) fn extend_for_closest_lines(
    boundary: &Boundary,
    intersections: Vec<Intersection>,
    start: Point,
    end: Point,
    search_radius: f32,
) -> Vec<Intersection> {
    let start_lines = closest_lines_in_radius(boundary, start, search_radius);
    let end_lines = closest_lines_in_radius(boundary, end, search_radius);
    let distance_of = |line: &ClosestLine| -> f64 {
        let contour = boundary.contour(line.contour);
        boundary.lengths(line.contour)[line.segment] + distance(line.point, contour[line.segment])
    };
    // `endpoints_close_to_same_boundary` scans end candidates first and
    // considers every start contour, not only the nearest start line.
    let mut new_intersections = intersections;
    let start_contours = start_lines
        .iter()
        .map(|line| line.contour)
        .collect::<std::collections::HashSet<_>>();
    if let Some(end_line) = end_lines
        .iter()
        .find(|line| start_contours.contains(&line.contour))
    {
        let start_line = start_lines
            .iter()
            .find(|line| line.contour == end_line.contour)
            .expect("end contour is present among start candidates");
        return vec![
            intersection_of(start_line, distance_of),
            intersection_of(end_line, distance_of),
        ];
    }
    if !new_intersections.is_empty() && !start_lines.is_empty() {
        if let Some(line) = get_closer(&start_lines, new_intersections[0], start, search_radius) {
            new_intersections[0] = intersection_of(&line, distance_of);
        } else {
            let line = shared_contour_line(&start_lines, &new_intersections, true)
                .unwrap_or(start_lines[0]);
            new_intersections.insert(0, intersection_of(&line, distance_of));
        }
    }
    if !new_intersections.is_empty() && !end_lines.is_empty() {
        let last = new_intersections.len() - 1;
        if let Some(line) = get_closer(&end_lines, new_intersections[last], end, search_radius) {
            new_intersections[last] = intersection_of(&line, distance_of);
        } else {
            let line =
                shared_contour_line(&end_lines, &new_intersections, false).unwrap_or(end_lines[0]);
            new_intersections.push(intersection_of(&line, distance_of));
        }
    }
    new_intersections
}

fn intersection_of(line: &ClosestLine, distance_of: impl Fn(&ClosestLine) -> f64) -> Intersection {
    Intersection {
        point: line.point,
        contour: line.contour,
        segment: line.segment,
        distance: distance_of(line),
        do_not_remove: true,
    }
}

/// `get_closer` (:219–231): replacement requires the existing crossing to be
/// within the radius. Upstream casts coordinate differences to float before
/// squaring; failure means append/prepend, not replacement by another hit.
fn get_closer(
    lines: &[ClosestLine],
    current: Intersection,
    close_to: Point,
    search_radius: f32,
) -> Option<ClosestLine> {
    let squared_distance = |point: Point| {
        let dx = (close_to.x() - point.x()) as f32;
        let dy = (close_to.y() - point.y()) as f32;
        dx * dx + dy * dy
    };
    let old_distance = squared_distance(current.point);
    let radius = search_radius;
    lines.iter().copied().find(|line| {
        line.contour == current.contour
            && old_distance <= radius * radius
            && squared_distance(line.point) < old_distance
    })
}

fn shared_contour_line(
    lines: &[ClosestLine],
    intersections: &[Intersection],
    reverse: bool,
) -> Option<ClosestLine> {
    let contours = lines
        .iter()
        .map(|line| line.contour)
        .collect::<std::collections::HashSet<_>>();
    let found = if reverse {
        intersections
            .iter()
            .rev()
            .find(|intersection| contours.contains(&intersection.contour))
    } else {
        intersections
            .iter()
            .find(|intersection| contours.contains(&intersection.contour))
    };
    let shared = found?;
    lines
        .iter()
        .copied()
        .find(|line| line.contour == shared.contour)
}
