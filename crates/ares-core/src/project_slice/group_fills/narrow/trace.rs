use crate::geometry::{Line, Point, Polygon};

struct TracedPolygon {
    lows: Vec<Point>,
    highs: Vec<Point>,
}

pub(super) fn reconstruct(sections: &[Vec<Line>], spacing: i64) -> Vec<Polygon> {
    let y_half = (0.5 * spacing as f64).round() as i64;
    let x_half = spacing / 2;
    let widened = sections
        .iter()
        .map(|section| {
            section
                .iter()
                .map(|line| {
                    Line::new(
                        Point::new(line.a.x(), line.a.y() - y_half),
                        Point::new(line.b.x(), line.b.y() + y_half),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let reconnect_limit = 2.0 * spacing as f64;
    let reconnect_limit_squared = reconnect_limit * reconnect_limit;
    let mut active: Vec<TracedPolygon> = Vec::new();
    let mut output = Vec::new();

    for section in &widened {
        let mut used = vec![false; section.len()];
        for traced in &mut active {
            let low = *traced.lows.last().unwrap();
            let high = *traced.highs.last().unwrap();
            let begin = section.partition_point(|segment| segment.b.y() <= low.y());
            let end = section.partition_point(|segment| segment.a.y() <= high.y());
            let available = (begin..end).find(|&candidate| !used[candidate]);
            if available.is_some() {
                // Preserve Fill.cpp:720-743: candidate identity advances, while
                // geometry and the used marker always refer to candidates_begin.
                let segment = section[begin];
                append_endpoint(&mut traced.lows, segment.a, x_half, reconnect_limit_squared);
                append_endpoint(
                    &mut traced.highs,
                    segment.b,
                    x_half,
                    reconnect_limit_squared,
                );
                used[begin] = true;
            } else {
                close_with_cap(traced, x_half, &mut output);
            }
        }
        active.retain(|traced| !traced.lows.is_empty());

        for (index, segment) in section.iter().enumerate() {
            if used[index] {
                continue;
            }
            active.push(TracedPolygon {
                lows: vec![Point::new(segment.a.x() - x_half, segment.a.y()), segment.a],
                highs: vec![Point::new(segment.b.x() - x_half, segment.b.y()), segment.b],
            });
        }
    }

    for traced in active {
        output.push(finish(traced));
    }
    output
}

fn append_endpoint(
    points: &mut Vec<Point>,
    next: Point,
    x_half: i64,
    reconnect_limit_squared: f64,
) {
    let previous = *points.last().unwrap();
    if squared_distance(previous, next) < reconnect_limit_squared {
        points.push(next);
    } else {
        points.push(Point::new(previous.x() + x_half, previous.y()));
        points.push(Point::new(next.x() - x_half, next.y()));
        points.push(next);
    }
}

fn squared_distance(left: Point, right: Point) -> f64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    dx * dx + dy * dy
}

fn close_with_cap(traced: &mut TracedPolygon, x_half: i64, output: &mut Vec<Polygon>) {
    let low = *traced.lows.last().unwrap();
    let high = *traced.highs.last().unwrap();
    traced.lows.push(Point::new(low.x() + x_half, low.y()));
    traced.highs.push(Point::new(high.x() + x_half, high.y()));
    let complete = TracedPolygon {
        lows: std::mem::take(&mut traced.lows),
        highs: std::mem::take(&mut traced.highs),
    };
    output.push(finish(complete));
}

fn finish(mut traced: TracedPolygon) -> Polygon {
    traced.lows.extend(traced.highs.into_iter().rev());
    Polygon::new(traced.lows)
}
