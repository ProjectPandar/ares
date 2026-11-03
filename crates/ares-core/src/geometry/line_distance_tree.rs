mod build;
mod floating;
mod intersections;
mod intersections_mm;
mod outside;
mod query;
mod radius;
pub(crate) mod segment;

pub(crate) use floating::FloatLineDistanceTree;

use super::{Line, Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NearestLine {
    pub(crate) line_index: usize,
    pub(crate) squared_distance: f64,
    pub(crate) nearest_point: [f64; 2],
}

pub(crate) struct LineDistanceTree<'a> {
    lines: &'a [Line],
    nodes: Vec<Node>,
}

#[derive(Clone, Copy)]
struct MmQuery {
    point: [f64; 2],
    scale: f64,
}

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub(super) min: Point,
    pub(super) max: Point,
}

#[derive(Clone, Copy)]
pub(super) enum NodeKind {
    Unused,
    Inner,
    Leaf(usize),
}

#[derive(Clone, Copy)]
pub(super) struct Node {
    pub(super) kind: NodeKind,
    pub(super) bounds: Bounds,
}

impl Node {
    pub(super) const UNUSED: Self = Self {
        kind: NodeKind::Unused,
        bounds: Bounds {
            min: Point::new(i64::MAX, i64::MAX),
            max: Point::new(i64::MIN, i64::MIN),
        },
    };
}

impl<'a> LineDistanceTree<'a> {
    pub(crate) fn new(lines: &'a [Line]) -> Self {
        Self {
            lines,
            nodes: build::build(lines),
        }
    }

    pub(crate) fn all_lines_in_radius(&self, point: Point, radius: f64) -> Vec<usize> {
        radius::query(self, point, radius * radius)
    }

    pub(crate) fn nearest(&self, point: Point) -> Option<NearestLine> {
        query::nearest(self.lines, &self.nodes, point)
    }

    /// Nearest line measured in unscaled millimetres
    /// (`LinesDistancer<Linef>` semantics — f64 endpoints at `coord *
    /// 1e-6`, f64 segment distance). `point` is unscaled millimetres.
    pub(crate) fn nearest_mm(
        &self,
        point: [f64; 2],
        scale_factor: f64,
    ) -> Option<query::mm::NearestMm> {
        query::mm::nearest_mm(self.lines, &self.nodes, point, scale_factor)
    }

    pub(crate) fn intersections_sorted(&self, line: Line) -> Vec<(Point, usize)> {
        intersections::sorted(self.lines, &self.nodes, line)
    }

    pub(crate) fn intersections_sorted_mm(
        &self,
        line: [[f64; 2]; 2],
        scale_factor: f64,
    ) -> Vec<([f64; 2], usize)> {
        intersections_mm::sorted(self.lines, &self.nodes, line, scale_factor)
    }

    pub(crate) fn outside(&self, point: Point) -> i32 {
        outside::classify(self.lines, &self.nodes, point)
    }

    /// Inside/outside classification at the unscaled f64 point
    /// (`point_outside_closed_contours` on the double-precision query).
    pub(crate) fn outside_mm(&self, point: [f64; 2], scale_factor: f64) -> i32 {
        outside::classify_mm(self.lines, &self.nodes, point, scale_factor)
    }
}
