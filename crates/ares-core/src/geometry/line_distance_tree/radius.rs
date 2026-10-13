// `AABBTreeIndirect.hpp:635-663`: preserve left/right traversal order and
// strict squared-distance admission for LineSplit's clip-vertex resolver.
use super::{
    LineDistanceTree, NodeKind,
    query::{exterior_distance_squared, segment_distance},
};
use crate::geometry::Point;

pub(super) fn query(tree: &LineDistanceTree, point: Point, limit: f64) -> Vec<usize> {
    let mut found = Vec::new();
    if !tree.nodes.is_empty() {
        visit(tree, 0, point, limit, &mut found);
    }
    found
}

fn visit(tree: &LineDistanceTree, index: usize, point: Point, limit: f64, found: &mut Vec<usize>) {
    match tree.nodes[index].kind {
        NodeKind::Unused => unreachable!("radius traversal reaches valid nodes"),
        NodeKind::Leaf(line) => {
            if segment_distance(tree.lines[line], point).0 < limit {
                found.push(line);
            }
        }
        NodeKind::Inner => {
            let origin = [point.x() as f64, point.y() as f64];
            for child in [index * 2 + 1, index * 2 + 2] {
                if exterior_distance_squared(tree.nodes[child].bounds, origin) < limit {
                    visit(tree, child, point, limit, found);
                }
            }
        }
    }
}
