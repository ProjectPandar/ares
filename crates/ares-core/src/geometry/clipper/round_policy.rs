use super::{predicates, types::Edge};
use crate::geometry::Point;

#[derive(Clone, Copy)]
pub(crate) struct IntersectionEdge {
    pub(crate) current: Point,
    pub(crate) bottom: Point,
    pub(crate) top: Point,
    pub(crate) dx: f64,
}

impl From<Edge> for IntersectionEdge {
    fn from(edge: Edge) -> Self {
        Self {
            current: edge.current.xy,
            bottom: edge.bottom.xy,
            top: edge.top.xy,
            dx: edge.dx,
        }
    }
}

impl IntersectionEdge {
    pub(super) fn is_horizontal(self) -> bool {
        self.bottom.y() == self.top.y()
    }
}

pub(crate) trait RoundPolicy {
    const REMOVE_INPUT_COLLINEAR: bool = true;

    fn intersection(first: IntersectionEdge, second: IntersectionEdge, top_y: i64) -> Point;
}

pub(super) struct Clipper1Round;

impl RoundPolicy for Clipper1Round {
    fn intersection(first: IntersectionEdge, second: IntersectionEdge, top_y: i64) -> Point {
        let point = predicates::intersect_point(first, second);
        if point.y() < top_y {
            Point::new(predicates::top_x(first, top_y), top_y)
        } else {
            point.xy
        }
    }
}
