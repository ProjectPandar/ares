use crate::geometry::{Point, Polygon};

/// WallToolPaths.cpp:331–369; separate from the intervening angular cleanup.
pub(super) fn remove(polygons: &mut Vec<Polygon>) {
    polygons.retain_mut(|polygon| {
        let points = polygon.points();
        let mut result = Vec::with_capacity(points.len());
        let mut changed = false;
        for (index, &point) in points.iter().enumerate() {
            let previous = result.last().copied().unwrap_or(points[points.len() - 1]);
            if index + 1 == points.len() && result.is_empty() {
                break;
            }
            let next = if index + 1 == points.len() {
                result[0]
            } else {
                points[index + 1]
            };
            if !is_degenerate(previous, point, next) {
                result.push(point);
                continue;
            }
            changed = true;
            while result.len() > 1
                && is_degenerate(result[result.len() - 2], result[result.len() - 1], next)
            {
                result.pop();
            }
        }
        if changed {
            if result.len() <= 2 {
                return false;
            }
            *polygon = Polygon::new(result);
        }
        true
    });
}

fn is_degenerate(previous: Point, point: Point, next: Point) -> bool {
    let incoming = (point.x() - previous.x(), point.y() - previous.y());
    let outgoing = (next.x() - point.x(), next.y() - point.y());
    let incoming_length =
        ((incoming.0 * incoming.0 + incoming.1 * incoming.1) as f64).sqrt() as i64;
    let outgoing_length =
        ((outgoing.0 * outgoing.0 + outgoing.1 * outgoing.1) as f64).sqrt() as i64;
    incoming.0 * outgoing.0 + incoming.1 * outgoing.1 == -incoming_length * outgoing_length
}
