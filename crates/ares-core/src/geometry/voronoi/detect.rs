use super::{Diagram, Issue, cell_range, planarity};
use crate::geometry::Line;

// Voronoi.cpp166-240: edge, cell, then angular-planarity precedence.
pub(super) fn issues(diagram: &Diagram, lines: &[Line]) -> Issue {
    for edge in diagram.edges() {
        let (Some(a), Some(b)) = (edge.vertex0(), diagram.edge_get_vertex1(edge.id()).unwrap())
        else {
            continue;
        };
        if diagram
            .position(a)
            .into_iter()
            .chain(diagram.position(b))
            .any(|v| !v.is_finite())
        {
            return Issue::NonFiniteVertex;
        }
        if !edge.is_linear()
            && !diagram.cell(edge.cell().unwrap()).unwrap().contains_point()
            && !diagram
                .cell(diagram.edge(edge.twin().unwrap()).unwrap().cell().unwrap())
                .unwrap()
                .contains_point()
        {
            return Issue::ParabolaWithoutFocus;
        }
    }
    for cell in diagram.cells() {
        if cell.is_degenerate() || !cell.contains_segment() {
            continue;
        }
        let line = lines[cell.source_index().usize()];
        let Some((mut edge_id, end)) = cell_range::segment(diagram, cell, line) else {
            return Issue::MissingVertex;
        };
        let from = [line.a.x() as f64, line.a.y() as f64];
        let vector = [line.b.x() as f64 - from[0], line.b.y() as f64 - from[1]];
        while edge_id != end {
            let edge = diagram.edge(edge_id).unwrap();
            let (Some(_), Some(v1)) = (edge.vertex0(), diagram.edge_get_vertex1(edge_id).unwrap())
            else {
                return Issue::MissingVertex;
            };
            let point = diagram.position(v1);
            if vector[0] * (point[1] - from[1]) - vector[1] * (point[0] - from[0]) < 0.0 {
                return Issue::IntersectingSegment;
            }
            edge_id = edge.next().unwrap();
        }
    }
    if !planarity::angular(diagram, lines) {
        return Issue::NonPlanar;
    }
    Issue::None
}
