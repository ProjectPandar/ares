//! VoronoiVisualUtils.hpp236–290, MultiMaterialSegmentation.cpp1507–1523.

use super::math::*;
use crate::geometry::{Line, voronoi::Diagram};
use boostvoronoi::prelude::EdgeIndex;

pub(super) fn position(d: &Diagram, id: boostvoronoi::prelude::VertexIndex) -> [f64; 2] {
    let v = d.vertex(id).unwrap();
    [v.x(), v.y()]
}
pub(super) fn infinite(d: &Diagram, id: EdgeIndex, lines: &[Line], extent: f64) -> Option<Line> {
    let a = d.cell(d.edge_get_cell(id).unwrap()).unwrap();
    let twin = d.edge_get_twin(id).unwrap();
    let b = d.cell(d.edge_get_cell(twin).unwrap()).unwrap();
    if !a.contains_point() && !b.contains_point() {
        return None;
    }
    let v0 = d.edge_get_vertex0(id).unwrap();
    let v1 = d.edge_get_vertex1(id).unwrap();
    let direction = if a.contains_point() && b.contains_point() {
        let retrieve = |cell: &boostvoronoi::prelude::Cell| {
            let line = lines[cell.source_index().usize()];
            point(if cell.contains_segment_endpoint() {
                line.b
            } else {
                line.a
            })
        };
        let mut p = retrieve(a);
        let mut q = retrieve(b);
        if v0.is_none() {
            std::mem::swap(&mut p, &mut q);
        }
        [p[1] - q[1], q[0] - p[0]]
    } else {
        let source = if a.contains_segment() {
            a.source_index().usize()
        } else {
            b.source_index().usize()
        };
        let line = lines[source];
        [
            (line.b.y() - line.a.y()) as f64,
            (line.a.x() - line.b.x()) as f64,
        ]
    };
    let factor = extent / direction[0].abs().max(direction[1].abs());
    let base = position(d, v0.or(v1).unwrap());
    let far = [
        base[0] + direction[0] * factor,
        base[1] + direction[1] * factor,
    ];
    Some(if v0.is_none() {
        Line::new(truncated(far), truncated(base))
    } else {
        Line::new(truncated(base), truncated(far))
    })
}
pub(super) fn finite(d: &Diagram, id: EdgeIndex, bbox: Bounds) -> Line {
    let mut a = position(d, d.edge_get_vertex0(id).unwrap().unwrap());
    let mut b = position(d, d.edge_get_vertex1(id).unwrap().unwrap());
    if bbox.contains_f64(a) != bbox.contains_f64(b) {
        let vector = unit(sub(b, a));
        let distance = norm(bbox.size());
        if !bbox.contains_f64(a) {
            a = [b[0] - vector[0] * distance, b[1] - vector[1] * distance];
        } else {
            b = [a[0] + vector[0] * distance, a[1] + vector[1] * distance];
        }
    }
    Line::new(truncated(a), truncated(b))
}
