use crate::geometry::{IntersectionEdge, Point, RoundPolicy};

pub(super) struct Clipper2Truncate;

impl RoundPolicy for Clipper2Truncate {
    // clipper.engine.cpp:2350-2382 AddNewIntersectNode, restricted to XY.
    fn intersection(first: IntersectionEdge, second: IntersectionEdge, top_y: i64) -> Point {
        let point =
            segment_intersection(first, second).unwrap_or(Point::new(first.current.x(), top_y));
        // BuildIntersectList updates only current.x; current.y remains Clipper2's bot_y_.
        if point.y() <= first.current.y() && point.y() >= top_y {
            return point;
        }
        let dx1 = first.dx.abs();
        let dx2 = second.dx.abs();
        let steeper = if dx1 > dx2 { first } else { second };
        if dx1 > 100.0 && dx2 > 100.0 {
            return closest(point, steeper);
        }
        if dx1 > 100.0 {
            return closest(point, first);
        }
        if dx2 > 100.0 {
            return closest(point, second);
        }
        let y = if point.y() < top_y {
            top_y
        } else {
            first.current.y()
        };
        let flatter = if dx1 < dx2 { first } else { second };
        Point::new(top_x(flatter, y), y)
    }
}

// clipper.core.h:898-917 default GetSegmentIntersectPt (not HI_PRECISION).
fn segment_intersection(first: IntersectionEdge, second: IntersectionEdge) -> Option<Point> {
    let dx1 = (first.top.x() - first.bottom.x()) as f64;
    let dy1 = (first.top.y() - first.bottom.y()) as f64;
    let dx2 = (second.top.x() - second.bottom.x()) as f64;
    let dy2 = (second.top.y() - second.bottom.y()) as f64;
    let determinant = dy1 * dx2 - dy2 * dx1;
    if determinant == 0.0 {
        return None;
    }
    let t = ((first.bottom.x() - second.bottom.x()) as f64 * dy2
        - (first.bottom.y() - second.bottom.y()) as f64 * dx2)
        / determinant;
    Some(if t <= 0.0 {
        first.bottom
    } else if t >= 1.0 {
        first.top
    } else {
        Point::new(
            (first.bottom.x() as f64 + t * dx1) as i64,
            (first.bottom.y() as f64 + t * dy1) as i64,
        )
    })
}

// clipper.engine.cpp:131-138 TopX (nearbyint with the default rounding mode).
fn top_x(edge: IntersectionEdge, y: i64) -> i64 {
    if y == edge.top.y() || edge.top.x() == edge.bottom.x() {
        edge.top.x()
    } else if y == edge.bottom.y() {
        edge.bottom.x()
    } else {
        edge.bottom.x() + (edge.dx * (y - edge.bottom.y()) as f64).round_ties_even() as i64
    }
}

// clipper.core.h:971-990 GetClosestPointOnSegment, integral Point64 branch.
fn closest(point: Point, edge: IntersectionEdge) -> Point {
    if edge.bottom == edge.top {
        return edge.bottom;
    }
    let dx = (edge.top.x() - edge.bottom.x()) as f64;
    let dy = (edge.top.y() - edge.bottom.y()) as f64;
    let q = (((point.x() - edge.bottom.x()) as f64 * dx
        + (point.y() - edge.bottom.y()) as f64 * dy)
        / (dx * dx + dy * dy))
        .clamp(0.0, 1.0);
    Point::new(
        edge.bottom.x() + (q * dx).round_ties_even() as i64,
        edge.bottom.y() + (q * dy).round_ties_even() as i64,
    )
}
