#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Point {
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
    if points.len() < 2 {
        return Vec::new();
    }
    let mut segments = Vec::with_capacity(points.len());
    let mut front = 0;
    let mut last_arc = None;
    for back in 0..points.len() {
        if back - front < 2 {
            continue;
        }
        let candidate = try_arc(&points[front..=back], tolerance);
        if let Some(arc) = candidate {
            last_arc = Some(arc);
            if back + 1 == points.len() {
                segments.push(Segment::Arc(arc));
                front = back;
            }
        } else {
            if back - front > 2 {
                segments.push(Segment::Arc(
                    last_arc.expect("a preceding point span fitted"),
                ));
            } else {
                append_line(&mut segments, points[front], points[front + 1]);
            }
            front = back - 1;
            last_arc = None;
        }
    }
    if front + 1 < points.len() {
        for index in front..points.len() - 1 {
            append_line(&mut segments, points[index], points[index + 1]);
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
    let start = *points.first()?;
    let end = *points.last()?;
    let middle = points[points.len() / 2];
    let determinant = 2.0
        * (start.x * (middle.y - end.y)
            + middle.x * (end.y - start.y)
            + end.x * (start.y - middle.y));
    if determinant.abs() <= f64::EPSILON {
        return None;
    }
    let start_square = start.x * start.x + start.y * start.y;
    let middle_square = middle.x * middle.x + middle.y * middle.y;
    let end_square = end.x * end.x + end.y * end.y;
    let center = Point {
        x: (start_square * (middle.y - end.y)
            + middle_square * (end.y - start.y)
            + end_square * (start.y - middle.y))
            / determinant,
        y: (start_square * (end.x - middle.x)
            + middle_square * (start.x - end.x)
            + end_square * (middle.x - start.x))
            / determinant,
    };
    let radius = distance(center, start);
    if radius <= f64::EPSILON || radius > 2_000.0 {
        return None;
    }
    if points
        .iter()
        .any(|point| (distance(center, *point) - radius).abs() > tolerance)
    {
        return None;
    }
    let cross = cross(sub(middle, start), sub(end, middle));
    if cross.abs() <= f64::EPSILON {
        return None;
    }
    let clockwise = cross < 0.0;
    let start_angle = (start.y - center.y).atan2(start.x - center.x);
    let end_angle = (end.y - center.y).atan2(end.x - center.x);
    let mut angle = if clockwise {
        start_angle - end_angle
    } else {
        end_angle - start_angle
    };
    while angle < 0.0 {
        angle += std::f64::consts::TAU;
    }
    let length = radius * angle;
    let chord_length = points
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum::<f64>();
    if length <= f64::EPSILON || (length - chord_length).abs() > length * 0.05 {
        return None;
    }
    Some(ArcSegment {
        end,
        center,
        length,
        clockwise,
    })
}

fn sub(left: Point, right: Point) -> Point {
    Point {
        x: left.x - right.x,
        y: left.y - right.y,
    }
}

fn cross(left: Point, right: Point) -> f64 {
    left.x * right.y - left.y * right.x
}

fn distance(left: Point, right: Point) -> f64 {
    (left.x - right.x).hypot(left.y - right.y)
}

#[cfg(test)]
mod tests;
