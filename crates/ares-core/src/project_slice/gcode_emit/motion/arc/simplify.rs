use super::{ArcSegment, Point, try_arc};
use crate::project_slice::perimeters::classic::materialize::{FittedArc, FittedMove};

#[derive(Clone, Copy)]
pub(super) struct FittedRange {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) arc: Option<ArcSegment>,
}

pub(super) fn fit_and_simplify(points: &[Point], tolerance: f64) -> (Vec<Point>, Vec<FittedRange>) {
    if points.len() < 2 {
        return (points.to_vec(), Vec::new());
    }
    let ranges = if tolerance.abs() > 0.0001 {
        fit_ranges(points, tolerance)
    } else {
        vec![FittedRange {
            start: 0,
            end: points.len() - 1,
            arc: None,
        }]
    };
    let mut simplified = Vec::with_capacity(points.len());
    simplified.push(points[0]);
    let mut simplified_ranges = Vec::with_capacity(ranges.len());
    for range in ranges {
        let part = douglas_peucker(&points[range.start..=range.end], tolerance);
        let start = simplified.len() - 1;
        simplified.extend_from_slice(&part[1..]);
        simplified_ranges.push(FittedRange {
            start,
            end: simplified.len() - 1,
            arc: range.arc,
        });
    }
    (simplified, simplified_ranges)
}

pub(in crate::project_slice) fn simplify_points(
    points: &mut Vec<(f64, f64)>,
    tolerance: f64,
) -> Vec<FittedMove> {
    let converted = points
        .iter()
        .map(|&(x, y)| Point { x, y })
        .collect::<Vec<_>>();
    let (converted, ranges) = fit_and_simplify(&converted, tolerance);
    let fitting = ranges
        .into_iter()
        .map(|range| FittedMove {
            start: range.start,
            end: range.end,
            arc: range.arc.map(|arc| FittedArc {
                center: (arc.center.x, arc.center.y),
                radius: arc.radius,
                length: arc.length,
                clockwise: arc.clockwise,
            }),
        })
        .collect();
    points.clear();
    points.extend(converted.into_iter().map(|point| (point.x, point.y)));
    fitting
}

pub(super) fn fit_ranges(points: &[Point], tolerance: f64) -> Vec<FittedRange> {
    if points.len() < 3 {
        return vec![FittedRange {
            start: 0,
            end: points.len() - 1,
            arc: None,
        }];
    }
    let mut ranges = Vec::with_capacity(points.len() / 2);
    let mut front = 0;
    let mut last_arc = None;
    for back in 0..points.len() {
        if back - front < 2 {
            continue;
        }
        if let Some(arc) = try_arc(&points[front..=back], tolerance) {
            last_arc = Some(arc);
            if back + 1 == points.len() {
                ranges.push(FittedRange {
                    start: front,
                    end: back,
                    arc: Some(arc),
                });
                front = back;
            }
        } else {
            if back - front > 2 {
                ranges.push(FittedRange {
                    start: front,
                    end: back - 1,
                    arc: Some(last_arc.expect("the preceding point span fitted as an arc")),
                });
            } else {
                append_linear_range(&mut ranges, front, front + 1);
            }
            front = back - 1;
            last_arc = None;
        }
    }
    if front + 1 < points.len() {
        append_linear_range(&mut ranges, front, points.len() - 1);
    }
    ranges
}

fn append_linear_range(ranges: &mut Vec<FittedRange>, start: usize, end: usize) {
    if let Some(last) = ranges.last_mut()
        && last.arc.is_none()
    {
        last.end = end;
    } else {
        ranges.push(FittedRange {
            start,
            end,
            arc: None,
        });
    }
}

fn douglas_peucker(points: &[Point], tolerance: f64) -> Vec<Point> {
    let Some(&first) = points.first() else {
        return Vec::new();
    };
    let mut result = Vec::with_capacity(points.len());
    result.push(first);
    if points.len() == 1 {
        return result;
    }

    let tolerance_squared = (tolerance * COORDINATE_UNITS_PER_MILLIMETER).powi(2);
    let mut anchor = 0;
    let mut floater = points.len() - 1;
    let mut endpoints = Vec::with_capacity(points.len());
    endpoints.push(floater);
    loop {
        let mut maximum = 0.0;
        let mut farthest = anchor;
        for index in anchor + 1..floater {
            let distance =
                point_segment_distance_squared(points[index], points[anchor], points[floater]);
            if distance > maximum {
                maximum = distance;
                farthest = index;
            }
        }
        if maximum <= tolerance_squared {
            result.push(points[floater]);
            anchor = floater;
            endpoints.pop();
            let Some(&next) = endpoints.last() else {
                break;
            };
            floater = next;
        } else {
            floater = farthest;
            endpoints.push(floater);
        }
    }
    result
}

const COORDINATE_UNITS_PER_MILLIMETER: f64 = 1_000_000.0;

fn point_segment_distance_squared(point: Point, start: Point, end: Point) -> f64 {
    let [point_x, point_y] = scaled_coordinates(point);
    let [start_x, start_y] = scaled_coordinates(start);
    let [end_x, end_y] = scaled_coordinates(end);
    let vector_x = (end_x - start_x) as f64;
    let vector_y = (end_y - start_y) as f64;
    let point_x = (point_x - start_x) as f64;
    let point_y = (point_y - start_y) as f64;
    let length_squared = vector_x * vector_x + vector_y * vector_y;
    if length_squared == 0.0 {
        return point_x * point_x + point_y * point_y;
    }
    let projection = (point_x * vector_x + point_y * vector_y) / length_squared;
    if projection <= 0.0 {
        point_x * point_x + point_y * point_y
    } else if projection >= 1.0 {
        let point_x = (point_x + start_x as f64) - end_x as f64;
        let point_y = (point_y + start_y as f64) - end_y as f64;
        point_x * point_x + point_y * point_y
    } else {
        let distance_x = projection * vector_x - point_x;
        let distance_y = projection * vector_y - point_y;
        distance_x * distance_x + distance_y * distance_y
    }
}

fn scaled_coordinates(point: Point) -> [i64; 2] {
    [
        (point.x * COORDINATE_UNITS_PER_MILLIMETER).round() as i64,
        (point.y * COORDINATE_UNITS_PER_MILLIMETER).round() as i64,
    ]
}
