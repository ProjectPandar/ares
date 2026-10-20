//! VoronoiUtilsCgal.cpp188-332: source tangent orientation and CCW triplets.
use super::{
    Diagram,
    predicates::{self, Parabola},
};
use crate::geometry::{Line, Point};
use boostvoronoi::prelude::{EdgeIndex, SourceCategory};

fn point(p: Point) -> [f64; 2] {
    [p.x() as f64, p.y() as f64]
}
fn ends(diagram: &Diagram, edge: EdgeIndex) -> ([f64; 2], [f64; 2]) {
    (
        diagram.position(diagram.edge_get_vertex0(edge).unwrap().unwrap()),
        diagram.position(diagram.edge_get_vertex1(edge).unwrap().unwrap()),
    )
}
fn parabola(diagram: &Diagram, id: EdgeIndex, lines: &[Line]) -> Parabola {
    let edge = diagram.edge(id).unwrap();
    let left = diagram.cell(edge.cell().unwrap()).unwrap();
    let right = diagram
        .cell(diagram.edge_get_cell(edge.twin().unwrap()).unwrap())
        .unwrap();
    let (focus, directrix) = if left.contains_point() {
        (left, right)
    } else {
        (right, left)
    };
    let line = lines[focus.source_index().usize()];
    let focus = point(match focus.source_category() {
        SourceCategory::SegmentStart => line.a,
        SourceCategory::SegmentEnd => line.b,
        SourceCategory::SinglePoint | SourceCategory::Segment => {
            unreachable!("segment-only source point")
        }
    });
    let line = lines[directrix.source_index().usize()];
    let (p, q) = ends(diagram, id);
    Parabola {
        focus,
        from: point(line.a),
        to: point(line.b),
        side: -predicates::orientation(p, q, focus),
    }
}
fn orientation(diagram: &Diagram, a: EdgeIndex, b: EdgeIndex, lines: &[Line]) -> i8 {
    let linear_a = diagram.edge(a).unwrap().is_linear();
    let linear_b = diagram.edge(b).unwrap().is_linear();
    let (p, q) = ends(diagram, a);
    match (linear_a, linear_b) {
        (true, true) => predicates::orientation(p, q, ends(diagram, b).1),
        (false, false) => {
            predicates::tangents(p, parabola(diagram, a, lines), parabola(diagram, b, lines))
        }
        (false, true) => {
            predicates::tangent_line(p, ends(diagram, b).1, parabola(diagram, a, lines))
        }
        (true, false) => -predicates::tangent_line(p, q, parabola(diagram, b, lines)),
    }
}
fn three_ccw(diagram: &Diagram, a: EdgeIndex, b: EdgeIndex, c: EdgeIndex, lines: &[Line]) -> bool {
    match orientation(diagram, a, b, lines) {
        0 => orientation(diagram, a, c, lines) == -1,
        1 => orientation(diagram, a, c, lines) != 1 || orientation(diagram, b, c, lines) != -1,
        -1 => orientation(diagram, a, c, lines) == -1 || orientation(diagram, b, c, lines) == 1,
        _ => unreachable!(),
    }
}
pub(super) fn angular(diagram: &Diagram, lines: &[Line]) -> bool {
    for vertex in diagram.vertices() {
        let first = vertex.get_incident_edge().unwrap();
        let mut edge = first;
        let mut edges = Vec::new();
        loop {
            if let (Some(a), Some(b)) = (
                diagram.edge_get_vertex0(edge).unwrap(),
                diagram.edge_get_vertex1(edge).unwrap(),
            ) && diagram
                .position(a)
                .into_iter()
                .chain(diagram.position(b))
                .all(f64::is_finite)
            {
                edges.push(edge);
            }
            edge = diagram.edge_rot_next(edge).unwrap();
            if edge == first {
                break;
            }
        }
        if edges.len() <= 2 {
            continue;
        }
        for i in 0..edges.len() {
            if !three_ccw(
                diagram,
                edges[(i + edges.len() - 1) % edges.len()],
                edges[i],
                edges[(i + 1) % edges.len()],
                lines,
            ) {
                return false;
            }
        }
    }
    true
}
