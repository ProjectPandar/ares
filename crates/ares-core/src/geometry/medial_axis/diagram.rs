use boostvoronoi::prelude::{
    Builder, CellIndex, Diagram, EdgeIndex, Line as BvLine, SourceCategory,
};

use crate::geometry::Line;

use super::MedialAxisError;

pub(crate) fn build(lines: &[Line]) -> Result<Diagram, MedialAxisError> {
    if lines.iter().any(|line| line.a == line.b) {
        return Err(MedialAxisError::ConstructionFailed);
    }
    let sites = lines
        .iter()
        .map(|line| BvLine::from([line.a.x(), line.a.y(), line.b.x(), line.b.y()]))
        .collect::<Vec<_>>();
    let diagram = Builder::<i64>::default()
        .with_segments(&sites)
        .and_then(Builder::build)
        .map_err(|_| MedialAxisError::ConstructionFailed)?;
    validate_topology(&diagram, lines)?;
    Ok(diagram)
}

fn validate_topology(diagram: &Diagram, lines: &[Line]) -> Result<(), MedialAxisError> {
    if !diagram.num_edges().is_multiple_of(2) {
        return Err(MedialAxisError::ConstructionFailed);
    }
    for cell in diagram.cells() {
        if cell.source_index().usize() >= lines.len()
            || !match cell.source_category() {
                SourceCategory::SegmentStart | SourceCategory::SegmentEnd => cell.contains_point(),
                SourceCategory::Segment => cell.contains_segment(),
                SourceCategory::SinglePoint => false,
            }
        {
            return Err(MedialAxisError::ConstructionFailed);
        }
    }
    for pair in (0..diagram.num_edges()).step_by(2) {
        let even = edge_index(diagram, pair);
        let odd = edge_index(diagram, pair + 1);
        if diagram.edge_get_twin(even).ok() != Some(odd)
            || diagram.edge_get_twin(odd).ok() != Some(even)
        {
            return Err(MedialAxisError::ConstructionFailed);
        }
        for edge in [even, odd] {
            let cell = diagram.edge_get_cell(edge).map_err(invariant)?;
            diagram.cell(cell).map_err(invariant)?;
            validate_face_cycle(diagram, edge)?;
            let twin = diagram.edge_get_twin(edge).map_err(invariant)?;
            let value = diagram.edge(edge).map_err(invariant)?;
            if value.is_secondary() != diagram.edge(twin).map_err(invariant)?.is_secondary()
                || diagram.edge_get_vertex0(edge).map_err(invariant)?
                    != diagram.edge_get_vertex1(twin).map_err(invariant)?
                || diagram.edge_get_vertex1(edge).map_err(invariant)?
                    != diagram.edge_get_vertex0(twin).map_err(invariant)?
            {
                return Err(MedialAxisError::ConstructionFailed);
            }
            if value.is_secondary() {
                validate_secondary_site(diagram, edge, lines)?;
            }
            if let Some(vertex) = diagram.edge_get_vertex0(edge).map_err(invariant)? {
                diagram.vertex(vertex).map_err(invariant)?;
                validate_rotation_cycle(diagram, edge, vertex)?;
            }
        }
    }
    for cell in diagram.cells() {
        if let Some(edge) = cell.get_incident_edge() {
            validate_face_cycle(diagram, edge)?;
        }
    }
    Ok(())
}

fn validate_face_cycle(diagram: &Diagram, first: EdgeIndex) -> Result<(), MedialAxisError> {
    let cell = diagram.edge_get_cell(first).map_err(invariant)?;
    let mut edge = first;
    for _ in 0..=diagram.num_edges() {
        if diagram.edge_get_cell(edge).map_err(invariant)? != cell {
            return Err(MedialAxisError::ConstructionFailed);
        }
        edge = diagram.edge_get_next(edge).map_err(invariant)?;
        if edge == first {
            return Ok(());
        }
    }
    Err(MedialAxisError::ConstructionFailed)
}

fn validate_rotation_cycle(
    diagram: &Diagram,
    first: EdgeIndex,
    vertex: boostvoronoi::prelude::VertexIndex,
) -> Result<(), MedialAxisError> {
    let mut edge = first;
    for _ in 0..=diagram.num_edges() {
        if diagram.edge_get_vertex0(edge).map_err(invariant)? != Some(vertex) {
            return Err(MedialAxisError::ConstructionFailed);
        }
        edge = diagram
            .edge_rot_next(edge)
            .ok_or(MedialAxisError::ConstructionFailed)?;
        if edge == first {
            return Ok(());
        }
    }
    Err(MedialAxisError::ConstructionFailed)
}

fn validate_secondary_site(
    diagram: &Diagram,
    edge: EdgeIndex,
    lines: &[Line],
) -> Result<(), MedialAxisError> {
    let twin = diagram.edge_get_twin(edge).map_err(invariant)?;
    let left = diagram
        .cell(diagram.edge_get_cell(edge).map_err(invariant)?)
        .map_err(invariant)?;
    let right = diagram
        .cell(diagram.edge_get_cell(twin).map_err(invariant)?)
        .map_err(invariant)?;
    if left.contains_point() == right.contains_point() {
        return Err(MedialAxisError::ConstructionFailed);
    }
    let (point, segment) = if left.contains_point() {
        (left, right)
    } else {
        (right, left)
    };
    if !segment.contains_segment() {
        return Err(MedialAxisError::ConstructionFailed);
    }
    let point_line = lines[point.source_index().usize()];
    let endpoint = match point.source_category() {
        SourceCategory::SegmentStart => point_line.a,
        SourceCategory::SegmentEnd => point_line.b,
        SourceCategory::Segment | SourceCategory::SinglePoint => {
            return Err(MedialAxisError::ConstructionFailed);
        }
    };
    let segment_line = lines[segment.source_index().usize()];
    if endpoint != segment_line.a && endpoint != segment_line.b {
        return Err(MedialAxisError::ConstructionFailed);
    }
    Ok(())
}

pub(crate) fn edge_index(diagram: &Diagram, index: usize) -> EdgeIndex {
    diagram.edge_index_unchecked(index)
}

pub(crate) fn cell(diagram: &Diagram, edge: EdgeIndex) -> Result<CellIndex, MedialAxisError> {
    diagram.edge_get_cell(edge).map_err(invariant)
}

pub(crate) fn twin(diagram: &Diagram, edge: EdgeIndex) -> Result<EdgeIndex, MedialAxisError> {
    diagram.edge_get_twin(edge).map_err(invariant)
}

pub(crate) fn vertex0(
    diagram: &Diagram,
    edge: EdgeIndex,
) -> Result<Option<(f64, f64)>, MedialAxisError> {
    vertex(diagram, diagram.edge_get_vertex0(edge).map_err(invariant)?)
}

pub(crate) fn vertex1(
    diagram: &Diagram,
    edge: EdgeIndex,
) -> Result<Option<(f64, f64)>, MedialAxisError> {
    vertex(diagram, diagram.edge_get_vertex1(edge).map_err(invariant)?)
}

fn vertex(
    diagram: &Diagram,
    id: Option<boostvoronoi::prelude::VertexIndex>,
) -> Result<Option<(f64, f64)>, MedialAxisError> {
    id.map(|id| {
        diagram
            .vertex(id)
            .map(|vertex| (vertex.x(), vertex.y()))
            .map_err(invariant)
    })
    .transpose()
}

fn invariant(_: boostvoronoi::BvError) -> MedialAxisError {
    MedialAxisError::ConstructionFailed
}
