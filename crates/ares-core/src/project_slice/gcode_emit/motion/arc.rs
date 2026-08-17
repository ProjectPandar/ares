mod simplify;

pub(super) use simplify::simplify_points;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice::gcode_emit) struct Point {
    pub(super) x: f64,
    pub(super) y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ArcSegment {
    pub(super) end: Point,
    pub(super) center: Point,
    pub(super) length: f64,
    pub(super) clockwise: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Segment {
    Line { end: Point, length: f64 },
    Arc(ArcSegment),
}

pub(super) fn fit(points: &[Point], tolerance: f64) -> Vec<Segment> {
    let (points, ranges) = simplify::fit_and_simplify(points, tolerance);
    let mut segments = Vec::with_capacity(points.len());
    for range in ranges {
        if let Some(arc) = range.arc {
            segments.push(Segment::Arc(arc));
        } else {
            for pair in points[range.start..=range.end].windows(2) {
                append_line(&mut segments, pair[0], pair[1]);
            }
        }
    }
    segments
}

fn append_line(segments: &mut Vec<Segment>, start: Point, end: Point) {
    segments.push(Segment::Line {
        end,
        length: distance(start, end),
    });
}

fn try_arc(points: &[Point], tolerance: f64) -> Option<ArcSegment> {
    let (center, radius) = fit_circle(points, tolerance)?;
    let start = points[0];
    let end = points[points.len() - 1];
    let middle = points[((points.len() - 2) / 2) + 1];
    let start_angle = polar(center, start);
    let middle_angle = polar(center, middle);
    let end_angle = polar(center, end);
    let (mut clockwise, mut angle) = arc_direction(start_angle, middle_angle, end_angle)?;
    let approximate_length = points
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum::<f64>();
    let mut length = radius * angle;
    if relative_difference(length, approximate_length) >= 0.05 {
        angle = std::f64::consts::TAU - angle;
        length = radius * angle;
        if relative_difference(length, approximate_length) >= 0.05 {
            return None;
        }
        clockwise = !clockwise;
    }
    if !angles_are_monotonic(points, center, start_angle, angle, clockwise) {
        return None;
    }
    Some(ArcSegment {
        end,
        center,
        length,
        clockwise,
    })
}

fn fit_circle(points: &[Point], tolerance: f64) -> Option<(Point, f64)> {
    let middle_index = points.len() / 2;
    let middle = if points.len() == 3 {
        points[middle_index]
    } else if points.len().is_multiple_of(2) {
        midpoint(points[middle_index], points[middle_index - 1])
    } else {
        midpoint(points[middle_index - 1], points[middle_index + 1])
    };
    if let Some(circle) = circle_from_three(points[0], middle, points[points.len() - 1])
        && deviation_sum(points, circle, tolerance).is_some()
    {
        return Some(circle);
    }
    let mut best = None::<((Point, f64), f64)>;
    for &candidate in &points[1..points.len() - 1] {
        let Some(circle) = circle_from_three(points[0], candidate, points[points.len() - 1]) else {
            continue;
        };
        let Some(deviation) = deviation_sum(points, circle, tolerance) else {
            continue;
        };
        if best.is_none_or(|(_, best_deviation)| deviation < best_deviation) {
            best = Some((circle, deviation));
        }
    }
    best.map(|(circle, _)| circle)
}

fn circle_from_three(first: Point, middle: Point, last: Point) -> Option<(Point, f64)> {
    let area =
        (first.y - middle.y) * (first.x - last.x) - (first.y - last.y) * (first.x - middle.x);
    if area.abs() <= 0.0001 {
        return None;
    }
    let determinant = 2.0
        * (first.x * (middle.y - last.y)
            + middle.x * (last.y - first.y)
            + last.x * (first.y - middle.y));
    let first_square = first.x * first.x + first.y * first.y;
    let middle_square = middle.x * middle.x + middle.y * middle.y;
    let last_square = last.x * last.x + last.y * last.y;
    let center = Point {
        x: (first_square * (middle.y - last.y)
            + middle_square * (last.y - first.y)
            + last_square * (first.y - middle.y))
            / determinant,
        y: (first_square * (last.x - middle.x)
            + middle_square * (first.x - last.x)
            + last_square * (middle.x - first.x))
            / determinant,
    };
    let radius = distance(center, first);
    (radius <= 2_000.0).then_some((center, radius))
}

fn deviation_sum(points: &[Point], circle: (Point, f64), tolerance: f64) -> Option<f64> {
    let (center, radius) = circle;
    let mut total = 0.0;
    for point in &points[1..points.len() - 1] {
        let deviation = (distance(center, *point) - radius).abs();
        if deviation > tolerance {
            return None;
        }
        total += deviation * deviation;
    }
    for pair in points.windows(2) {
        let dx = pair[1].x - pair[0].x;
        let dy = pair[1].y - pair[0].y;
        let denominator = dx * dx + dy * dy;
        let parameter = ((center.x - pair[0].x) * dx + (center.y - pair[0].y) * dy) / denominator;
        if !(f64::EPSILON..1.0 - f64::EPSILON).contains(&parameter) {
            continue;
        }
        let closest = Point {
            x: pair[0].x + parameter * dx,
            y: pair[0].y + parameter * dy,
        };
        let deviation = (distance(center, closest) - radius).abs();
        if deviation > tolerance {
            return None;
        }
        total += deviation * deviation;
    }
    Some(total)
}

fn arc_direction(start: f64, middle: f64, end: f64) -> Option<(bool, f64)> {
    if end > start {
        if start < middle && middle < end {
            Some((false, end - start))
        } else if middle < start || end < middle {
            Some((true, start + std::f64::consts::TAU - end))
        } else {
            None
        }
    } else if start > end {
        if start < middle || middle < end {
            Some((false, end + std::f64::consts::TAU - start))
        } else if end < middle && middle < start {
            Some((true, start - end))
        } else {
            None
        }
    } else {
        None
    }
}

fn angles_are_monotonic(
    points: &[Point],
    center: Point,
    start_angle: f64,
    total_angle: f64,
    clockwise: bool,
) -> bool {
    let mut previous = 0.0;
    for point in &points[1..points.len() - 1] {
        let angle = polar(center, *point);
        let delta = if clockwise {
            (start_angle - angle).rem_euclid(std::f64::consts::TAU)
        } else {
            (angle - start_angle).rem_euclid(std::f64::consts::TAU)
        };
        if delta <= previous || delta >= total_angle {
            return false;
        }
        previous = delta;
    }
    true
}

fn relative_difference(left: f64, right: f64) -> f64 {
    ((left - right) / right).abs()
}

fn polar(center: Point, point: Point) -> f64 {
    (point.y - center.y)
        .atan2(point.x - center.x)
        .rem_euclid(std::f64::consts::TAU)
}

fn midpoint(left: Point, right: Point) -> Point {
    Point {
        x: (left.x + right.x) * 0.5,
        y: (left.y + right.y) * 0.5,
    }
}

fn distance(left: Point, right: Point) -> f64 {
    (left.x - right.x).hypot(left.y - right.y)
}

#[cfg(test)]
mod tests;
