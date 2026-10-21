//! MultiMaterialSegmentation.cpp:59–201 (active MMU_Graph storage).

mod attach;
mod build;
mod cleanup;
mod extract;
use super::math;
mod point_lookup;
mod vertices;
mod voronoi_edge;

use super::painted_lines::ColoredLine;
use crate::{
    SliceError,
    geometry::{CoordinateScale, ExPolygon},
};
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq)]
enum ArcType {
    Border,
    Interior,
}
#[derive(Clone, Copy)]
struct Arc {
    from: usize,
    to: usize,
    color: usize,
    kind: ArcType,
}
struct Node {
    point: [f64; 2],
    arcs: Vec<usize>,
}
struct Graph {
    nodes: Vec<Node>,
    arcs: Vec<Arc>,
    border_count: usize,
    offsets: Vec<usize>,
    sizes: Vec<usize>,
}
impl Graph {
    fn new(contours: &[Vec<ColoredLine>]) -> Self {
        let mut nodes = Vec::new();
        let mut offsets = Vec::new();
        let mut sizes = Vec::new();
        for lines in contours {
            offsets.push(nodes.len());
            sizes.push(lines.len());
            for line in lines {
                nodes.push(Node {
                    point: math::point(line.line.a),
                    arcs: Vec::new(),
                });
            }
        }
        let mut graph = Self {
            border_count: nodes.len(),
            nodes,
            arcs: Vec::new(),
            offsets,
            sizes,
        };
        for (contour, lines) in contours.iter().enumerate() {
            for (index, line) in lines.iter().enumerate() {
                graph.append(
                    graph.offsets[contour] + index,
                    graph.offsets[contour] + (index + 1) % lines.len(),
                    line.color,
                    ArcType::Border,
                );
            }
        }
        graph
    }
    fn append(&mut self, from: usize, to: usize, color: usize, kind: ArcType) {
        if self.nodes[from].arcs.iter().any(|&a| self.arcs[a].to == to)
            || self.nodes[to].arcs.iter().any(|&a| self.arcs[a].to == from)
        {
            return;
        }
        self.nodes[from].arcs.push(self.arcs.len());
        self.arcs.push(Arc {
            from,
            to,
            color,
            kind,
        });
        if kind == ArcType::Interior {
            self.nodes[to].arcs.push(self.arcs.len());
            self.arcs.push(Arc {
                from: to,
                to: from,
                color,
                kind,
            });
        }
    }
    fn connect(&mut self, from: usize, to: usize) {
        self.append(from, to, usize::MAX, ArcType::Interior);
    }
    fn remove_edge(&mut self, from: usize, to: usize) {
        for (a, b) in [(from, to), (to, from)] {
            if let Some(index) = self.nodes[a]
                .arcs
                .iter()
                .position(|&id| self.arcs[id].to == b)
            {
                self.nodes[a].arcs.remove(index);
            }
        }
    }
    fn prune(&mut self) {
        let mut pending = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| (i >= self.border_count && n.arcs.len() == 1).then_some(i))
            .collect::<VecDeque<_>>();
        while let Some(from) = pending.pop_front() {
            if self.nodes[from].arcs.is_empty() {
                continue;
            }
            let to = self.arcs[self.nodes[from].arcs[0]].to;
            self.remove_edge(from, to);
            if to >= self.border_count && self.nodes[to].arcs.len() == 1 {
                pending.push_back(to);
            }
        }
    }
}

pub(super) fn segment(
    contours: &[Vec<ColoredLine>],
    states: usize,
    scale: CoordinateScale,
) -> Result<Vec<Vec<ExPolygon>>, SliceError> {
    let mut graph = build::build(contours, scale)?;
    cleanup::remove_multiple(&mut graph, contours);
    graph.prune();
    Ok(extract::extract(&graph, states))
}
