//! Containing-box order and strict hits: `AABBTreeIndirect.hpp:552-632`.

use super::super::segment::distance_squared;
use super::{Bounds, FloatLineDistanceTree, FloatLineHit, NativeFloat, NodeKind};

pub(super) fn nearest<F: NativeFloat>(
    tree: &FloatLineDistanceTree<F>,
    origin: [F; 2],
    limit: F,
) -> Option<FloatLineHit<F>> {
    if tree.nodes.is_empty() {
        return None;
    }
    let mut query = Query {
        tree,
        origin,
        limit,
        hit: None,
    };
    query.visit(0);
    query.hit
}

/// AABBTreeIndirect.hpp:639–667 visits left then right with strict limits.
pub(super) fn radius<F: NativeFloat>(
    tree: &FloatLineDistanceTree<F>,
    origin: [F; 2],
    limit: F,
) -> Vec<usize> {
    let mut found = Vec::new();
    if !tree.nodes.is_empty() {
        radius_visit(tree, origin, limit, 0, &mut found);
    }
    found
}
fn radius_visit<F: NativeFloat>(
    tree: &FloatLineDistanceTree<F>,
    origin: [F; 2],
    limit: F,
    index: usize,
    found: &mut Vec<usize>,
) {
    match tree.nodes[index].kind {
        NodeKind::Unused => unreachable!("radius traversal reaches valid tree nodes"),
        NodeKind::Leaf(line) => {
            let [a, b] = tree.lines[line];
            if distance_squared(a, b, origin).0 < limit {
                found.push(line);
            }
        }
        NodeKind::Inner => {
            for child in [index * 2 + 1, index * 2 + 2] {
                if exterior(tree.nodes[child].bounds, origin) < limit {
                    radius_visit(tree, origin, limit, child, found);
                }
            }
        }
    }
}

struct Query<'a, F> {
    tree: &'a FloatLineDistanceTree<F>,
    origin: [F; 2],
    limit: F,
    hit: Option<FloatLineHit<F>>,
}

impl<F: NativeFloat> Query<'_, F> {
    fn visit(&mut self, index: usize) {
        let node = self.tree.nodes[index];
        match node.kind {
            NodeKind::Unused => unreachable!("traversal reaches only valid native tree nodes"),
            NodeKind::Leaf(line) => {
                let [a, b] = self.tree.lines[line];
                let (distance, point) = distance_squared(a, b, self.origin);
                if distance < self.limit {
                    self.limit = distance;
                    self.hit = Some(FloatLineHit {
                        index: line,
                        squared_distance: distance,
                        point,
                    });
                }
            }
            NodeKind::Inner => self.visit_children(index),
        }
    }

    fn visit_children(&mut self, index: usize) {
        let left = index * 2 + 1;
        let right = left + 1;
        let left_bounds = self.tree.nodes[left].bounds;
        let right_bounds = self.tree.nodes[right].bounds;
        let looked_left = contains(left_bounds, self.origin);
        let looked_right = contains(right_bounds, self.origin);
        if looked_left {
            self.visit(left);
        }
        if looked_right {
            self.visit(right);
        }
        let left_distance = exterior(left_bounds, self.origin);
        let right_distance = exterior(right_bounds, self.origin);
        let ordered = if left_distance < right_distance {
            [
                (left, left_distance, looked_left),
                (right, right_distance, looked_right),
            ]
        } else {
            [
                (right, right_distance, looked_right),
                (left, left_distance, looked_left),
            ]
        };
        for (child, distance, looked) in ordered {
            if !looked && distance < self.limit {
                self.visit(child);
            }
        }
    }
}

fn contains<F: NativeFloat>(bounds: Bounds<F>, origin: [F; 2]) -> bool {
    (0..2).all(|axis| bounds.min[axis] <= origin[axis] && origin[axis] <= bounds.max[axis])
}

fn exterior<F: NativeFloat>(bounds: Bounds<F>, origin: [F; 2]) -> F {
    let mut distance = F::ZERO;
    for (axis, value) in origin.into_iter().enumerate() {
        let delta = if value < bounds.min[axis] {
            bounds.min[axis] - value
        } else if value > bounds.max[axis] {
            value - bounds.max[axis]
        } else {
            F::ZERO
        };
        distance = distance + delta * delta;
    }
    distance
}
