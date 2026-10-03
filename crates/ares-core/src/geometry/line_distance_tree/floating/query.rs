//! Containing-box order and bounded strict hits from `AABBTreeIndirect.hpp`.

use super::super::segment::distance_squared;
use super::{Bounds, FloatLineDistanceTree, FloatLineHit, NodeKind};

pub(super) fn nearest(
    tree: &FloatLineDistanceTree,
    origin: [f64; 2],
    limit: f64,
) -> Option<FloatLineHit> {
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

struct Query<'a> {
    tree: &'a FloatLineDistanceTree,
    origin: [f64; 2],
    limit: f64,
    hit: Option<FloatLineHit>,
}

impl Query<'_> {
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

fn contains(bounds: Bounds, origin: [f64; 2]) -> bool {
    (0..2).all(|axis| bounds.min[axis] <= origin[axis] && origin[axis] <= bounds.max[axis])
}

fn exterior(bounds: Bounds, origin: [f64; 2]) -> f64 {
    let mut distance = 0.0;
    for (axis, value) in origin.into_iter().enumerate() {
        let delta = if value < bounds.min[axis] {
            bounds.min[axis] - value
        } else if value > bounds.max[axis] {
            value - bounds.max[axis]
        } else {
            0.0
        };
        distance += delta * delta;
    }
    distance
}
