//! Floating intersections for `AABBTreeLines::LinesDistancer<Linef3>` queries
//! in `ExtrusionProcessor.hpp`; keep sub-micron positions until path scaling.

use super::{Node, NodeKind};
use crate::geometry::Line;

struct Hit {
    squared_distance: f64,
    point: [f64; 2],
    line_index: usize,
}

pub(super) fn sorted(
    lines: &[Line],
    nodes: &[Node],
    line: [[f64; 2]; 2],
    scale_factor: f64,
) -> Vec<([f64; 2], usize)> {
    if nodes.is_empty() {
        return Vec::new();
    }
    let mut collector = Collector {
        lines,
        nodes,
        line,
        scale_factor,
        minimum: [line[0][0].min(line[1][0]), line[0][1].min(line[1][1])],
        maximum: [line[0][0].max(line[1][0]), line[0][1].max(line[1][1])],
        hits: Vec::new(),
    };
    collector.visit(0);
    collector
        .hits
        .sort_by(|left, right| left.squared_distance.total_cmp(&right.squared_distance));
    collector
        .hits
        .into_iter()
        .map(|hit| (hit.point, hit.line_index))
        .collect()
}

struct Collector<'a> {
    lines: &'a [Line],
    nodes: &'a [Node],
    line: [[f64; 2]; 2],
    scale_factor: f64,
    minimum: [f64; 2],
    maximum: [f64; 2],
    hits: Vec<Hit>,
}

impl Collector<'_> {
    fn visit(&mut self, index: usize) {
        match self.nodes[index].kind {
            NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
            NodeKind::Leaf(line_index) => {
                let line = self.lines[line_index];
                let unscale = |point: crate::geometry::Point| {
                    [
                        point.x() as f64 * self.scale_factor,
                        point.y() as f64 * self.scale_factor,
                    ]
                };
                if let Some(point) = intersection(self.line, [unscale(line.a), unscale(line.b)]) {
                    let dx = point[0] - self.line[0][0];
                    let dy = point[1] - self.line[0][1];
                    self.hits.push(Hit {
                        squared_distance: dx * dx + dy * dy,
                        point,
                        line_index,
                    });
                }
            }
            NodeKind::Inner => {
                let left = index * 2 + 1;
                let right = left + 1;
                if self.overlaps(left) {
                    self.visit(left);
                }
                if self.overlaps(right) {
                    self.visit(right);
                }
            }
        }
    }

    fn overlaps(&self, index: usize) -> bool {
        let bounds = self.nodes[index].bounds;
        bounds.min.x() as f64 * self.scale_factor <= self.maximum[0]
            && bounds.max.x() as f64 * self.scale_factor >= self.minimum[0]
            && bounds.min.y() as f64 * self.scale_factor <= self.maximum[1]
            && bounds.max.y() as f64 * self.scale_factor >= self.minimum[1]
    }
}

// `Line.hpp::line_alg::intersection`: double endpoints, cross2 and EPSILON.
fn intersection(first: [[f64; 2]; 2], second: [[f64; 2]; 2]) -> Option<[f64; 2]> {
    let v1 = [first[1][0] - first[0][0], first[1][1] - first[0][1]];
    let v2 = [second[1][0] - second[0][0], second[1][1] - second[0][1]];
    let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
    let denominator = cross(v1, v2);
    if denominator.abs() < 1e-4 {
        return None;
    }
    let v12 = [first[0][0] - second[0][0], first[0][1] - second[0][1]];
    let t1 = cross(v2, v12) / denominator;
    let t2 = cross(v1, v12) / denominator;
    if (0.0..=1.0).contains(&t1) && (0.0..=1.0).contains(&t2) {
        Some([first[0][0] + t1 * v1[0], first[0][1] + t1 * v1[1]])
    } else {
        None
    }
}
