//! MultiMaterialSegmentation.cpp::build_graph1586–1720 (contour attachment).

use super::super::painted_lines::ColoredLine;
use super::{Graph, math::*, voronoi_edge};
use crate::geometry::{Line, Point, voronoi::Diagram};
use boostvoronoi::prelude::EdgeIndex;

pub(super) struct Attach<'a> {
    pub(super) graph: &'a mut Graph,
    pub(super) diagram: &'a Diagram,
    pub(super) processed: &'a mut [bool],
    pub(super) mapping: &'a [usize],
    pub(super) lines: &'a [ColoredLine],
    pub(super) segments: &'a [Line],
    pub(super) force: &'a mut [bool],
    pub(super) bbox: Bounds,
    pub(super) epsilon: f64,
}
impl Attach<'_> {
    pub(super) fn mark(&mut self, id: EdgeIndex) {
        self.processed[id.usize()] = true;
        self.processed[self.diagram.edge_get_twin(id).unwrap().usize()] = true;
    }
    pub(super) fn nodes(&self, id: EdgeIndex) -> (usize, usize) {
        (
            self.mapping[self.diagram.edge_get_vertex0(id).unwrap().unwrap().usize()],
            self.mapping[self.diagram.edge_get_vertex1(id).unwrap().unwrap().usize()],
        )
    }
    fn target(&self, border: usize, p: Point) -> usize {
        let arc = self.graph.arcs[border];
        if at_start(self.lines[border].line, p) {
            arc.from
        } else {
            arc.to
        }
    }
    fn intersection(&self, border: usize, line: Line) -> Option<Point> {
        intersect(self.lines[border].line, line, 15.0 * self.epsilon)
    }
    pub(super) fn infinite(&mut self, id: EdgeIndex, border: usize) {
        let size = self.bbox.size();
        let Some(line) =
            voronoi_edge::infinite(self.diagram, id, self.segments, size[0].max(size[1]))
        else {
            return;
        };
        let Some(p) = self.intersection(border, line) else {
            return;
        };
        let vertex = self
            .diagram
            .edge_get_vertex1(id)
            .unwrap()
            .or(self.diagram.edge_get_vertex0(id).unwrap())
            .unwrap();
        let from = self.mapping[vertex.usize()];
        let to = self.target(border, p);
        if from != to && from < self.graph.nodes.len() && to < self.graph.nodes.len() {
            self.graph.connect(from, to);
            self.mark(id);
        }
    }
    pub(super) fn outside(&mut self, id: EdgeIndex, from: usize) {
        let line = voronoi_edge::finite(self.diagram, id, self.bbox);
        let twin = self.diagram.edge_get_twin(id).unwrap();
        let a = self
            .diagram
            .cell(self.diagram.edge_get_cell(id).unwrap())
            .unwrap()
            .source_index()
            .usize();
        let b = self
            .diagram
            .cell(self.diagram.edge_get_cell(twin).unwrap())
            .unwrap()
            .source_index()
            .usize();
        if let Some(p) = self.intersection(b, line) {
            self.graph.connect(from, self.target(b, p));
        } else if let Some(p) = self.intersection(a, line) {
            self.graph.connect(from, self.target(a, p));
        }
        self.mark(id);
    }
    pub(super) fn finite(&mut self, id: EdgeIndex, source: usize) {
        let (a, b) = self.nodes(id);
        let border = self.graph.border_count;
        if (a < border && b < border) || a == b {
            return;
        }
        let count = self.graph.nodes.len();
        if a >= count || b >= count {
            if a < count && a >= border {
                self.outside(id, a);
            }
            if b < count && b >= border {
                self.outside(id, b);
            }
        } else if a < border || b < border {
            self.on_contour(id, source);
        } else {
            self.crossing(id, source);
        }
    }
    fn neighbors(&self, border: usize) -> (ColoredLine, ColoredLine) {
        let line = self.lines[border];
        let offset = self.graph.offsets[line.contour];
        let n = self.graph.sizes[line.contour];
        (
            self.lines[offset + (line.segment + n - 1) % n],
            self.lines[offset + (line.segment + 1) % n],
        )
    }
    pub(super) fn on_contour(&mut self, id: EdgeIndex, border: usize) {
        self.mark(id);
        let (a, b) = self.nodes(id);
        let edge = voronoi_edge::finite(self.diagram, id, self.bbox);
        let (p, q) = if a < self.graph.border_count {
            (edge.a, edge.b)
        } else {
            (edge.b, edge.a)
        };
        let line = self.lines[border];
        let (previous, next) = self.neighbors(border);
        let (neighbor, first, second) = if at_start(line.line, p) {
            (previous, previous.line, line.line)
        } else {
            (next, line.line, next.line)
        };
        if (neighbor.color != line.color || self.force[line.contour]) && inside(first, second, q) {
            self.graph.connect(a, b);
            self.force[line.contour] = false;
        }
    }
    pub(super) fn crossing(&mut self, id: EdgeIndex, border: usize) {
        let edge = voronoi_edge::finite(self.diagram, id, self.bbox);
        let Some(p) = self.intersection(border, edge) else {
            return;
        };
        self.mark(id);
        let line = self.lines[border];
        let (previous, next) = self.neighbors(border);
        let (neighbor, first, second) = if at_start(line.line, p) {
            (previous, previous.line, line.line)
        } else {
            (next, line.line, next.line)
        };
        if neighbor.color == line.color {
            return;
        }
        let (a, b) = self.nodes(id);
        let target = self.target(border, p);
        for from in [a, b] {
            if inside(first, second, truncated(self.graph.nodes[from].point)) {
                self.graph.connect(from, target);
            }
        }
    }
}
