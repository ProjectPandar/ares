use super::ClosedClipper;
use super::types::EdgeId;
use crate::geometry::{Point, Polygon};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct IntBounds {
    pub(crate) left: i64,
    pub(crate) top: i64,
    pub(crate) right: i64,
    pub(crate) bottom: i64,
}

impl ClosedClipper {
    pub(crate) fn bounds(&self) -> IntBounds {
        let Some(first_minimum) = self.minima.first() else {
            return IntBounds::default();
        };
        let first = self.edges.edge(first_minimum.left).bottom;
        let mut result = IntBounds {
            left: first.x(),
            top: first.y(),
            right: first.x(),
            bottom: first.y(),
        };

        for minimum in &self.minima {
            result.bottom = result.bottom.max(self.edges.edge(minimum.left).bottom.y());
            self.extend_bounds_along_lml(minimum.left, &mut result);
            self.extend_bounds_along_lml(minimum.right, &mut result);
        }
        result
    }

    fn extend_bounds_along_lml(&self, mut edge: EdgeId, result: &mut IntBounds) {
        while let Some(next) = self.edges.edge(edge).next_in_lml {
            let point = self.edges.edge(edge).bottom;
            result.left = result.left.min(point.x());
            result.right = result.right.max(point.x());
            edge = next;
        }

        let bound_end = self.edges.edge(edge);
        result.left = result.left.min(bound_end.bottom.x());
        result.right = result.right.max(bound_end.bottom.x());
        result.left = result.left.min(bound_end.top.x());
        result.right = result.right.max(bound_end.top.x());
        result.top = result.top.min(bound_end.top.y());
    }
}

pub(crate) fn negative_outer(bounds: IntBounds) -> Polygon {
    Polygon::new(vec![
        Point::new(bounds.left - 10, bounds.bottom + 10),
        Point::new(bounds.right + 10, bounds.bottom + 10),
        Point::new(bounds.right + 10, bounds.top - 10),
        Point::new(bounds.left - 10, bounds.top - 10),
    ])
}
