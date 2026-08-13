use crate::geometry::{Line, Point, Polygon};

struct TracedPolygon {
    lows: Vec<Point>,
    highs: Vec<Point>,
}

pub(super) fn trace_sections(sections: &[Vec<Line>], spacing: i64) -> Vec<Polygon> {
    let mut output = Vec::new();
    let mut current = Vec::<TracedPolygon>::new();
    for slice in sections {
        let mut used = vec![false; slice.len()];
        for trace in &mut current {
            let (begin, end) = candidate_range(
                slice,
                *trace.lows.last().expect("active trace has a low point"),
                *trace.highs.last().expect("active trace has a high point"),
            );
            let candidate = (begin..end).find(|&index| !used[index]);
            if let Some(index) = candidate {
                let segment = slice[index];
                append_connected(&mut trace.lows, segment.a, spacing);
                append_connected(&mut trace.highs, segment.b, spacing);
                used[index] = true;
            } else {
                close_trace(trace, spacing, &mut output);
            }
        }
        current.retain(|trace| !trace.lows.is_empty());
        for (index, &segment) in slice.iter().enumerate() {
            if !used[index] {
                let half_spacing = spacing / 2;
                current.push(TracedPolygon {
                    lows: vec![
                        Point::new(segment.a.x() - half_spacing, segment.a.y()),
                        segment.a,
                    ],
                    highs: vec![
                        Point::new(segment.b.x() - half_spacing, segment.b.y()),
                        segment.b,
                    ],
                });
            }
        }
    }
    for mut trace in current {
        emit_trace(&mut trace, &mut output);
    }
    output
}

fn candidate_range(slice: &[Line], low: Point, high: Point) -> (usize, usize) {
    let begin = slice
        .iter()
        .position(|segment| segment.b.y() > low.y())
        .unwrap_or(slice.len());
    let end = slice
        .iter()
        .position(|segment| segment.a.y() > high.y())
        .unwrap_or(slice.len());
    (begin, end)
}

fn append_connected(points: &mut Vec<Point>, candidate: Point, spacing: i64) {
    let previous = *points.last().expect("active trace has an endpoint");
    let distance_squared = distance_squared(previous, candidate);
    let threshold = 36.0 * spacing as f64 * spacing as f64;
    if distance_squared < threshold {
        points.push(candidate);
    } else {
        let half_spacing = spacing / 2;
        points.push(Point::new(previous.x() + half_spacing, previous.y()));
        points.push(Point::new(candidate.x() - half_spacing, candidate.y()));
        points.push(candidate);
    }
}

fn distance_squared(left: Point, right: Point) -> f64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    dx * dx + dy * dy
}

fn close_trace(trace: &mut TracedPolygon, spacing: i64, output: &mut Vec<Polygon>) {
    let half_spacing = spacing / 2;
    let low = *trace.lows.last().expect("active trace has a low point");
    let high = *trace.highs.last().expect("active trace has a high point");
    trace.lows.push(Point::new(low.x() + half_spacing, low.y()));
    trace
        .highs
        .push(Point::new(high.x() + half_spacing, high.y()));
    emit_trace(trace, output);
}

fn emit_trace(trace: &mut TracedPolygon, output: &mut Vec<Polygon>) {
    let mut points = std::mem::take(&mut trace.lows);
    points.extend(trace.highs.drain(..).rev());
    output.push(Polygon::new(points));
}

#[cfg(test)]
pub(super) fn candidate_range_for_test(slice: &[Line], low: Point, high: Point) -> (usize, usize) {
    candidate_range(slice, low, high)
}

#[cfg(test)]
pub(super) fn distance_squared_bits_for_test(left: Point, right: Point) -> u64 {
    distance_squared(left, right).to_bits()
}

#[cfg(test)]
pub(super) fn connect_points_for_test(
    previous: Point,
    candidate: Point,
    spacing: i64,
) -> Vec<Point> {
    let mut points = vec![previous];
    append_connected(&mut points, candidate, spacing);
    points
}
