mod build;
mod floating;
mod intersections;
mod intersections_mm;
mod outside;
mod query;
mod segment;

pub(crate) use floating::FloatLineDistanceTree;

use super::{Line, Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NearestLine {
    pub(crate) line_index: usize,
    pub(crate) squared_distance: f64,
    pub(crate) nearest_point: [f64; 2],
}

pub(crate) struct LineDistanceTree<'a> {
    lines: &'a [Line],
    nodes: Vec<Node>,
}

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub(super) min: Point,
    pub(super) max: Point,
}

#[derive(Clone, Copy)]
pub(super) enum NodeKind {
    Unused,
    Inner,
    Leaf(usize),
}

#[derive(Clone, Copy)]
pub(super) struct Node {
    pub(super) kind: NodeKind,
    pub(super) bounds: Bounds,
}

impl Node {
    pub(super) const UNUSED: Self = Self {
        kind: NodeKind::Unused,
        bounds: Bounds {
            min: Point::new(i64::MAX, i64::MAX),
            max: Point::new(i64::MIN, i64::MIN),
        },
    };
}

impl<'a> LineDistanceTree<'a> {
    pub(crate) fn new(lines: &'a [Line]) -> Self {
        Self {
            lines,
            nodes: build::build(lines),
        }
    }

    pub(crate) fn nearest(&self, point: Point) -> Option<NearestLine> {
        query::nearest(self.lines, &self.nodes, point)
    }

    /// Nearest line measured in unscaled millimetres
    /// (`LinesDistancer<Linef>` semantics — f64 endpoints at `coord *
    /// 1e-6`, f64 segment distance). `point` is unscaled millimetres.
    pub(crate) fn nearest_mm(
        &self,
        point: [f64; 2],
        scale_factor: f64,
    ) -> Option<query::mm::NearestMm> {
        query::mm::nearest_mm(self.lines, &self.nodes, point, scale_factor)
    }

    pub(crate) fn intersections_sorted(&self, line: Line) -> Vec<(Point, usize)> {
        intersections::sorted(self.lines, &self.nodes, line)
    }

    pub(crate) fn intersections_sorted_mm(
        &self,
        line: [[f64; 2]; 2],
        scale_factor: f64,
    ) -> Vec<([f64; 2], usize)> {
        intersections_mm::sorted(self.lines, &self.nodes, line, scale_factor)
    }

    pub(crate) fn outside(&self, point: Point) -> i32 {
        outside::classify(self.lines, &self.nodes, point)
    }

    /// Inside/outside classification at the unscaled f64 point
    /// (`point_outside_closed_contours` on the double-precision query).
    pub(crate) fn outside_mm(&self, point: [f64; 2], scale_factor: f64) -> i32 {
        outside::classify_mm(self.lines, &self.nodes, point, scale_factor)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeStateForTest {
    Unused,
    Inner,
    Leaf(usize),
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NodeSnapshotForTest {
    pub(crate) state: NodeStateForTest,
    pub(crate) min: Point,
    pub(crate) max: Point,
}

#[cfg(test)]
impl LineDistanceTree<'_> {
    pub(crate) fn intersections_in_traversal_order_for_test(
        &self,
        line: Line,
    ) -> Vec<(Point, usize)> {
        intersections::in_traversal_order(self.lines, &self.nodes, line)
    }

    pub(crate) fn squared_intersection_sort_key_bits_for_test(origin: Point, point: Point) -> u64 {
        intersections::squared_distance_bits_for_test(origin, point)
    }

    pub(crate) fn ray_counts_for_test(&self, point: Point, coordinate: usize) -> (i32, i32) {
        outside::ray_counts(self.lines, &self.nodes, point, coordinate)
    }

    pub(crate) fn centroids_for_test(&self) -> Vec<Point> {
        build::centroids_for_test(self.lines)
    }

    pub(crate) fn exterior_distance_squared_for_test(
        minimum: Point,
        maximum: Point,
        origin: [f64; 2],
    ) -> f64 {
        query::exterior_distance_squared(
            Bounds {
                min: minimum,
                max: maximum,
            },
            origin,
        )
    }

    pub(crate) fn node_snapshots(&self) -> Vec<NodeSnapshotForTest> {
        self.nodes
            .iter()
            .map(|node| NodeSnapshotForTest {
                state: match node.kind {
                    NodeKind::Unused => NodeStateForTest::Unused,
                    NodeKind::Inner => NodeStateForTest::Inner,
                    NodeKind::Leaf(index) => NodeStateForTest::Leaf(index),
                },
                min: node.bounds.min,
                max: node.bounds.max,
            })
            .collect()
    }
}
