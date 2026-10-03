//! Native `AABBTreeLines.hpp` over `Linef` / `Tree<2, double>`.

use super::NodeKind;

mod build;
mod query;

pub(crate) struct FloatLineDistanceTree {
    lines: Vec<[[f64; 2]; 2]>,
    nodes: Vec<Node>,
}

#[derive(Clone, Copy)]
struct Bounds {
    min: [f64; 2],
    max: [f64; 2],
}

#[derive(Clone, Copy)]
struct Node {
    kind: NodeKind,
    bounds: Bounds,
}

pub(crate) struct FloatLineHit {
    pub(crate) squared_distance: f64,
    pub(crate) point: [f64; 2],
}

impl FloatLineDistanceTree {
    pub(crate) fn new(lines: Vec<[[f64; 2]; 2]>) -> Self {
        let nodes = build::build(&lines);
        Self { lines, nodes }
    }

    pub(crate) fn nearest_within(
        &self,
        point: [f64; 2],
        squared_limit: f64,
    ) -> Option<FloatLineHit> {
        query::nearest(self, point, squared_limit)
    }
}
