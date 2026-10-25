//! Native `AABBTreeLines.hpp:165-224,300-345` over `Linef` / `ExtrusionLine`.

use super::NodeKind;

mod build;
mod intersections;
mod query;
mod scalar;

pub(super) use scalar::NativeFloat;

#[derive(Default)]
pub(crate) struct FloatLineDistanceTree<F = f64> {
    lines: Vec<[[F; 2]; 2]>,
    nodes: Vec<Node<F>>,
}

#[derive(Clone, Copy)]
struct Bounds<F> {
    min: [F; 2],
    max: [F; 2],
}

#[derive(Clone, Copy)]
struct Node<F> {
    kind: NodeKind,
    bounds: Bounds<F>,
}

pub(crate) struct FloatLineHit<F = f64> {
    pub(crate) index: usize,
    pub(crate) squared_distance: F,
    pub(crate) point: [F; 2],
}

impl<F: NativeFloat> FloatLineDistanceTree<F> {
    pub(crate) fn new(lines: Vec<[[F; 2]; 2]>) -> Self {
        let nodes = build::build(&lines);
        Self { lines, nodes }
    }

    pub(crate) fn nearest_within(
        &self,
        point: [F; 2],
        squared_limit: F,
    ) -> Option<FloatLineHit<F>> {
        query::nearest(self, point, squared_limit)
    }

    pub(crate) fn intersections_sorted(&self, line: [[F; 2]; 2]) -> Vec<([F; 2], usize)> {
        intersections::sorted(self, line)
    }
}
