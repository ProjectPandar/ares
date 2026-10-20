use super::Diagram;
use crate::geometry::Line;
use boostvoronoi::prelude::{Cell, EdgeIndex};

// VoronoiUtils.cpp212-254,318-325: source endpoint equality after llround.
pub(super) fn segment(
    diagram: &Diagram,
    cell: &Cell,
    line: Line,
) -> Option<(EdgeIndex, EdgeIndex)> {
    let first = cell.get_incident_edge().unwrap();
    let mut edge_id = first;
    let mut begin = None;
    let mut end = None;
    let mut seen_start = false;
    let mut after_start = false;
    let mut end_before_start = false;
    loop {
        let edge = diagram.edge(edge_id).unwrap();
        let v0 = edge.vertex0();
        let v1 = diagram.edge_get_vertex1(edge_id).unwrap();
        if let (Some(v0), Some(v1)) = (v0, v1) {
            let v0 = diagram.position(v0).map(|v| v.round() as i64);
            let v1 = diagram.position(v1).map(|v| v.round() as i64);
            if v0 == [line.b.x(), line.b.y()] && !after_start {
                begin = Some(edge_id);
                seen_start = true;
            } else if seen_start {
                after_start = true;
            }
            if v1 == [line.a.x(), line.a.y()] && (end.is_none() || end_before_start) {
                end_before_start = !after_start;
                end = Some(edge_id);
            }
        }
        edge_id = edge.next().unwrap();
        if edge_id == first {
            break;
        }
    }
    match (begin, end) {
        (Some(begin), Some(end)) if begin != end => Some((begin, end)),
        _ => None,
    }
}
