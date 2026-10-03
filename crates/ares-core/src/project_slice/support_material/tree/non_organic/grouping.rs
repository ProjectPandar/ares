//! Per-part grouping and MST construction in `TreeSupport.cpp:2773-2847`.

use std::collections::VecDeque;

use super::{
    movement,
    nodes::{NodeArena, NodeId},
    spanning_tree::MinimumSpanningTree,
};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, point_hash_map::PointHashMap};

pub(super) struct Group {
    pub(super) nodes: Vec<NodeId>,
    pub(super) tree: MinimumSpanningTree,
}

pub(super) fn group_nodes(
    arena: &NodeArena,
    nodes: &[NodeId],
    parts: &[ExPolygon],
    buildplate_only: bool,
    scale: CoordinateScale,
) -> (Vec<Group>, VecDeque<NodeId>) {
    let mut maps = (0..=parts.len())
        .map(|_| PointHashMap::new())
        .collect::<Vec<_>>();
    let mut unsupported = VecDeque::new();
    for &id in nodes {
        let node = &arena.nodes[id];
        if buildplate_only && !node.to_buildplate {
            unsupported.push_front(id);
            continue;
        }
        let group = if node.to_buildplate || parts.is_empty() {
            0
        } else {
            closest_part(parts, node.position, scale) + 1
        };
        maps[group].insert(node.position, id);
    }
    let groups = maps
        .into_iter()
        .map(|map| {
            let ordered = map.into_ordered();
            let points = ordered.iter().map(|&(point, _)| point).collect::<Vec<_>>();
            Group {
                nodes: ordered.into_iter().map(|(_, id)| id).collect(),
                tree: MinimumSpanningTree::new(&points, scale),
            }
        })
        .collect();
    (groups, unsupported)
}

fn closest_part(parts: &[ExPolygon], position: Point, scale: CoordinateScale) -> usize {
    let mut closest = 0;
    let mut minimum = f64::MAX;
    for (index, part) in parts.iter().enumerate() {
        if movement::is_inside(part, position) {
            return index;
        }
        let point = closest_vertex(part.contour(), position);
        let difference = Point::new(position.x() - point.x(), position.y() - point.y());
        let distance = movement::distance_squared(difference, scale);
        if distance < minimum {
            minimum = distance;
            closest = index;
        }
    }
    closest
}

fn closest_vertex(polygon: &Polygon, position: Point) -> Point {
    let points = polygon.points();
    let mut closest = points[0];
    let mut minimum = coordinate_distance(position, closest);
    for &point in &points[1..] {
        let distance = coordinate_distance(point, position);
        if distance < minimum {
            minimum = distance;
            closest = point;
        }
    }
    closest
}

fn coordinate_distance(first: Point, second: Point) -> f64 {
    let x = (first.x() - second.x()) as f64;
    let y = (first.y() - second.y()) as f64;
    (x * x + y * y).sqrt()
}
