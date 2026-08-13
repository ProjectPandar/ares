use crate::geometry::{Line, Point, fixed_msvc_sort_by};

use super::{Bounds, Node, NodeKind};

#[derive(Clone, Copy)]
struct KeyedHit {
    squared_distance: f64,
    hit: (Point, usize),
}

pub(super) fn sorted(lines: &[Line], nodes: &[Node], line: Line) -> Vec<(Point, usize)> {
    let hits = in_traversal_order(lines, nodes, line);
    let mut keyed = hits
        .into_iter()
        .map(|hit| KeyedHit {
            squared_distance: squared_distance(line.a, hit.0),
            hit,
        })
        .collect::<Vec<_>>();
    fixed_msvc_sort_by(&mut keyed, |left, right| {
        left.squared_distance < right.squared_distance
    });
    keyed.into_iter().map(|item| item.hit).collect()
}

pub(super) fn in_traversal_order(
    lines: &[Line],
    nodes: &[Node],
    line: Line,
) -> Vec<(Point, usize)> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let mut collector = IntersectionCollector {
        lines,
        nodes,
        line,
        line_bounds: line_bounds(line),
        intersections: Vec::new(),
    };
    collector.visit(0);
    collector.intersections
}

struct IntersectionCollector<'a> {
    lines: &'a [Line],
    nodes: &'a [Node],
    line: Line,
    line_bounds: Bounds,
    intersections: Vec<(Point, usize)>,
}

impl IntersectionCollector<'_> {
    fn visit(&mut self, node_index: usize) {
        let node = self.nodes[node_index];
        match node.kind {
            NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
            NodeKind::Leaf(line_index) => {
                if let Some(point) = self.line.intersection(self.lines[line_index]) {
                    self.intersections.push((point, line_index));
                }
            }
            NodeKind::Inner => {
                let left_index = node_index * 2 + 1;
                let right_index = left_index + 1;
                if bounds_intersect(self.nodes[left_index].bounds, self.line_bounds) {
                    self.visit(left_index);
                }
                if bounds_intersect(self.nodes[right_index].bounds, self.line_bounds) {
                    self.visit(right_index);
                }
            }
        }
    }
}

fn bounds_intersect(left: Bounds, right: Bounds) -> bool {
    left.min.x() <= right.max.x()
        && left.max.x() >= right.min.x()
        && left.min.y() <= right.max.y()
        && left.max.y() >= right.min.y()
}

fn line_bounds(line: Line) -> Bounds {
    Bounds {
        min: Point::new(line.a.x().min(line.b.x()), line.a.y().min(line.b.y())),
        max: Point::new(line.a.x().max(line.b.x()), line.a.y().max(line.b.y())),
    }
}

fn squared_distance(origin: Point, point: Point) -> f64 {
    let dx = (point.x() - origin.x()) as f64;
    let dy = (point.y() - origin.y()) as f64;
    dx * dx + dy * dy
}

#[cfg(test)]
pub(super) fn squared_distance_bits_for_test(origin: Point, point: Point) -> u64 {
    squared_distance(origin, point).to_bits()
}
