use crate::{BrimOptions, BrimPath, Point2, SliceError};

pub(super) fn brim_ear_paths(
    contour: &crate::Contour,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<Vec<BrimPath>, SliceError> {
    let mut paths = Vec::new();
    for corner in ear_candidate_vertices(
        contour,
        options.brim_ears_max_angle_degrees(),
        options.brim_ears_detection_length_mm(),
    ) {
        for loop_index in 0..loop_count {
            let brim_offset =
                (f64::from(loop_index + 1) * effective_line_width).min(options.width_mm());
            paths.push(square_ear_path(
                corner,
                options.object_gap_mm() + brim_offset,
            )?);
        }
    }
    Ok(paths)
}

fn ear_candidate_vertices(
    contour: &crate::Contour,
    max_angle_degrees: f64,
    detection_length_mm: f64,
) -> Vec<Point2> {
    if max_angle_degrees <= 0.0 {
        return Vec::new();
    }
    let points = simplified_closed_points(contour.points(), detection_length_mm);
    ear_candidate_points(&points, max_angle_degrees)
}

fn ear_candidate_points(points: &[Point2], max_angle_degrees: f64) -> Vec<Point2> {
    if points.len() < 3 {
        return Vec::new();
    }

    let winding = contour_signed_area(points).signum();
    let mut candidates = Vec::new();
    for index in 0..points.len() {
        let previous = points[(index + points.len() - 1) % points.len()];
        let current = points[index];
        let next = points[(index + 1) % points.len()];
        if interior_angle_degrees(previous, current, next, winding) <= max_angle_degrees {
            candidates.push(current);
        }
    }
    candidates
}

fn interior_angle_degrees(previous: Point2, current: Point2, next: Point2, winding: f64) -> f64 {
    let ax = previous.x() - current.x();
    let ay = previous.y() - current.y();
    let bx = next.x() - current.x();
    let by = next.y() - current.y();
    let a_len = ax.hypot(ay);
    let b_len = bx.hypot(by);
    if a_len == 0.0 || b_len == 0.0 {
        return 180.0;
    }
    let cosine = ((ax * bx + ay * by) / (a_len * b_len)).clamp(-1.0, 1.0);
    let smaller_angle = cosine.acos().to_degrees();
    let cross = ax * by - ay * bx;
    if winding != 0.0 && cross.signum() == winding {
        360.0 - smaller_angle
    } else {
        smaller_angle
    }
}

fn contour_signed_area(points: &[Point2]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(a, b)| a.x() * b.y() - b.x() * a.y())
        .sum::<f64>()
        * 0.5
}

fn simplified_closed_points(points: &[Point2], tolerance: f64) -> Vec<Point2> {
    if tolerance <= 0.0 || points.len() < 4 {
        return points.to_vec();
    }
    let (start, end) = farthest_vertex_pair(points);
    let mut first = douglas_peucker(&ring_chain(points, start, end), tolerance);
    let mut second = douglas_peucker(&ring_chain(points, end, start), tolerance);
    first.pop();
    second.pop();
    first.extend(second);
    if first.len() < 4 {
        points.to_vec()
    } else {
        first
    }
}

fn farthest_vertex_pair(points: &[Point2]) -> (usize, usize) {
    let mut pair = (0, 1);
    let mut max_distance = 0.0;
    for first in 0..points.len() {
        for second in (first + 1)..points.len() {
            let distance = squared_distance(points[first], points[second]);
            if distance > max_distance {
                max_distance = distance;
                pair = (first, second);
            }
        }
    }
    pair
}

fn ring_chain(points: &[Point2], start: usize, end: usize) -> Vec<Point2> {
    let mut chain = vec![points[start]];
    let mut index = start;
    while index != end {
        index = (index + 1) % points.len();
        chain.push(points[index]);
    }
    chain
}

fn douglas_peucker(points: &[Point2], tolerance: f64) -> Vec<Point2> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let (index, distance) = farthest_point_from_segment(points);
    if distance <= tolerance {
        vec![points[0], points[points.len() - 1]]
    } else {
        let mut first = douglas_peucker(&points[..=index], tolerance);
        let second = douglas_peucker(&points[index..], tolerance);
        first.pop();
        first.extend(second);
        first
    }
}

fn farthest_point_from_segment(points: &[Point2]) -> (usize, f64) {
    let start = points[0];
    let end = points[points.len() - 1];
    (1..points.len() - 1)
        .map(|index| (index, point_segment_distance(points[index], start, end)))
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap()
}

fn point_segment_distance(point: Point2, start: Point2, end: Point2) -> f64 {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return squared_distance(point, start).sqrt();
    }
    let t = (((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / length_squared)
        .clamp(0.0, 1.0);
    let projection = Point2::new(start.x() + t * dx, start.y() + t * dy);
    squared_distance(point, projection).sqrt()
}

fn squared_distance(first: Point2, second: Point2) -> f64 {
    let dx = first.x() - second.x();
    let dy = first.y() - second.y();
    dx * dx + dy * dy
}

fn square_ear_path(center: Point2, radius: f64) -> Result<BrimPath, SliceError> {
    BrimPath::new(vec![
        Point2::new(center.x() - radius, center.y() - radius),
        Point2::new(center.x() + radius, center.y() - radius),
        Point2::new(center.x() + radius, center.y() + radius),
        Point2::new(center.x() - radius, center.y() + radius),
    ])
}
