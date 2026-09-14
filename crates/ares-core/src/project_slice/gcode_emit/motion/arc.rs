mod retained;
mod simplify;

pub(super) use retained::{clip_end as clip_fitting_end, from_fitting};
pub(in crate::project_slice) use simplify::{simplify_linear_points, simplify_points};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice::gcode_emit) struct Point {
    pub(super) x: f64,
    pub(super) y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ArcSegment {
    pub(super) start: Point,
    pub(super) end: Point,
    pub(super) center: Point,
    pub(super) radius: f64,
    pub(super) length: f64,
    pub(super) clockwise: bool,
}

#[derive(Clone, Copy)]
struct ArcSlice {
    center: Point,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    clockwise: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Segment {
    Line { end: Point, length: f64 },
    Arc(ArcSegment),
}

pub(super) fn fit(points: &[Point], tolerance: f64) -> Vec<Segment> {
    let ranges = simplify::fit_ranges(points, tolerance, 1_000_000.0);
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

fn try_arc(points: &[Point], tolerance: f64, units_per_mm: f64) -> Option<ArcSegment> {
    let (center, radius) = fit_circle(points, tolerance, units_per_mm)?;
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
    if !points_within_arc(
        points,
        ArcSlice {
            center,
            radius,
            start_angle,
            end_angle,
            clockwise,
        },
    ) {
        return None;
    }
    Some(ArcSegment {
        start,
        end,
        center,
        radius,
        length,
        clockwise,
    })
}

fn fit_circle(points: &[Point], tolerance: f64, units_per_mm: f64) -> Option<(Point, f64)> {
    let middle_index = points.len() / 2;
    let middle = if points.len() == 3 {
        points[middle_index]
    } else if points.len().is_multiple_of(2) {
        scaled_midpoint(points[middle_index], points[middle_index - 1], units_per_mm)
    } else {
        scaled_midpoint(
            points[middle_index - 1],
            points[middle_index + 1],
            units_per_mm,
        )
    };
    if let Some(circle) =
        circle_from_three(points[0], middle, points[points.len() - 1], units_per_mm)
        && deviation_sum(points, circle, tolerance, units_per_mm).is_some()
    {
        return Some(circle);
    }
    let mut best = None::<((Point, f64), f64)>;
    for (index, &candidate) in points[1..points.len() - 1].iter().enumerate() {
        if index + 1 == middle_index {
            continue;
        }
        let Some(circle) =
            circle_from_three(points[0], candidate, points[points.len() - 1], units_per_mm)
        else {
            continue;
        };
        let Some(deviation) = deviation_sum(points, circle, tolerance, units_per_mm) else {
            continue;
        };
        if best.is_none_or(|(_, best_deviation)| deviation < best_deviation) {
            best = Some((circle, deviation));
        }
    }
    best.map(|(circle, _)| circle)
}

fn circle_from_three(
    first: Point,
    middle: Point,
    last: Point,
    units_per_mm: f64,
) -> Option<(Point, f64)> {
    const PARALLEL_AREA_MM2: f64 = 1.0e-4;
    let scale = scaled_point(Point { x: 1.0, y: 0.0 }, units_per_mm).x;
    let parallel_area_threshold = PARALLEL_AREA_MM2 * scale * scale;
    let first = scaled_point(first, units_per_mm);
    let middle = scaled_point(middle, units_per_mm);
    let last = scaled_point(last, units_per_mm);
    let area =
        (first.y - middle.y) * (first.x - last.x) - (first.y - last.y) * (first.x - middle.x);
    if area.abs() <= parallel_area_threshold {
        return None;
    }
    let a = first.x * (middle.y - last.y) - first.y * (middle.x - last.x) + middle.x * last.y
        - last.x * middle.y;
    if a.abs() < 1.0 {
        return None;
    }
    let first_square = first.x * first.x + first.y * first.y;
    let middle_square = middle.x * middle.x + middle.y * middle.y;
    let last_square = last.x * last.x + last.y * last.y;
    let b = first_square * (last.y - middle.y)
        + middle_square * (first.y - last.y)
        + last_square * (middle.y - first.y);
    let c = first_square * (middle.x - last.x)
        + middle_square * (last.x - first.x)
        + last_square * (first.x - middle.x);
    let center_x = -b / (2.0 * a);
    let center_y = -c / (2.0 * a);
    let radius = (center_x - first.x).hypot(center_y - first.y) / scale;
    (radius <= 2_000.0).then_some((
        Point {
            x: center_x.round() * (1.0 / scale),
            y: center_y.round() * (1.0 / scale),
        },
        radius,
    ))
}

fn deviation_sum(
    points: &[Point],
    circle: (Point, f64),
    tolerance: f64,
    units_per_mm: f64,
) -> Option<f64> {
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
        if !((5.0e-6)..(1.0 - 5.0e-6)).contains(&parameter) {
            continue;
        }
        let closest = Point {
            x: quantize(pair[0].x + parameter * dx, units_per_mm),
            y: quantize(pair[0].y + parameter * dy, units_per_mm),
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
        } else if (0.0 <= middle && middle < start)
            || (end < middle && middle < std::f64::consts::TAU)
        {
            Some((true, start + std::f64::consts::TAU - end))
        } else {
            None
        }
    } else if start > end {
        if (start < middle && middle < std::f64::consts::TAU) || (0.0 < middle && middle < end) {
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

fn points_within_arc(points: &[Point], arc: ArcSlice) -> bool {
    let mut previous = arc.start_angle;
    let crosses_zero = if arc.clockwise {
        arc.start_angle < arc.end_angle
    } else {
        arc.start_angle > arc.end_angle
    };
    let mut crossed_zero = false;
    let start_direction = (
        (points[0].x - arc.center.x) / arc.radius,
        (points[0].y - arc.center.y) / arc.radius,
    );
    let end = points[points.len() - 1];
    let end_direction = (
        (end.x - arc.center.x) / arc.radius,
        (end.y - arc.center.y) / arc.radius,
    );
    for index in points.len() - 2..points.len() {
        let angle = if index + 1 == points.len() {
            arc.end_angle
        } else {
            polar(arc.center, points[index])
        };
        if index + 1 < points.len() && !angle_within_arc(angle, arc, crosses_zero) {
            return false;
        }
        if !advance_polar(
            previous,
            angle,
            arc.clockwise,
            crosses_zero,
            &mut crossed_zero,
        ) {
            return false;
        }
        if (index != 1
            && ray_intersects_segment(
                arc.center,
                start_direction,
                points[index - 1],
                points[index],
            ))
            || (index + 1 != points.len()
                && ray_intersects_segment(
                    arc.center,
                    end_direction,
                    points[index - 1],
                    points[index],
                ))
        {
            return false;
        }
        previous = angle;
    }
    crosses_zero == crossed_zero
}

fn angle_within_arc(angle: f64, arc: ArcSlice, crosses_zero: bool) -> bool {
    if arc.clockwise {
        if crosses_zero {
            angle < arc.start_angle || angle > arc.end_angle
        } else {
            arc.start_angle > angle && angle > arc.end_angle
        }
    } else if crosses_zero {
        angle > arc.start_angle || angle < arc.end_angle
    } else {
        arc.start_angle < angle && angle < arc.end_angle
    }
}

fn advance_polar(
    previous: f64,
    angle: f64,
    clockwise: bool,
    crosses_zero: bool,
    crossed_zero: &mut bool,
) -> bool {
    let wraps = if clockwise {
        previous < angle
    } else {
        previous > angle
    };
    if !wraps {
        return true;
    }
    if !crosses_zero || *crossed_zero {
        return false;
    }
    *crossed_zero = true;
    true
}

fn ray_intersects_segment(origin: Point, direction: (f64, f64), a: Point, b: Point) -> bool {
    let v1 = (origin.x - a.x, origin.y - a.y);
    let v2 = (b.x - a.x, b.y - a.y);
    let v3 = (-direction.1, direction.0);
    let dot = v2.0 * v3.0 + v2.1 * v3.1;
    if dot.abs() < 0.0001 {
        return false;
    }
    let t1 = (v2.0 * v1.1 - v2.1 * v1.0) / dot;
    let t2 = (v1.0 * v3.0 + v1.1 * v3.1) / dot;
    t1 >= 0.0 && (0.0..=1.0).contains(&t2)
}

fn relative_difference(left: f64, right: f64) -> f64 {
    ((left - right) / right).abs()
}

fn polar(center: Point, point: Point) -> f64 {
    (point.y - center.y)
        .atan2(point.x - center.x)
        .rem_euclid(std::f64::consts::TAU)
}

fn scaled_point(point: Point, units_per_mm: f64) -> Point {
    Point {
        x: (point.x * units_per_mm).round(),
        y: (point.y * units_per_mm).round(),
    }
}

fn scaled_midpoint(left: Point, right: Point, units_per_mm: f64) -> Point {
    let left = scaled_point(left, units_per_mm);
    let right = scaled_point(right, units_per_mm);
    Point {
        x: ((left.x as i64 + right.x as i64) / 2) as f64 / units_per_mm,
        y: ((left.y as i64 + right.y as i64) / 2) as f64 / units_per_mm,
    }
}

fn quantize(value: f64, units_per_mm: f64) -> f64 {
    (value * units_per_mm).trunc() / units_per_mm
}

fn distance(left: Point, right: Point) -> f64 {
    (left.x - right.x).hypot(left.y - right.y)
}

#[cfg(test)]
mod tests;
