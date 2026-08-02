use boostvoronoi::prelude::Diagram;

use super::{Annotations, CellCategory, EdgeCategory, VertexCategory};
use crate::geometry::medial_axis::{MedialAxisError, diagram};

pub(super) fn propagate_point_edges(
    vd: &Diagram,
    out: &mut Annotations,
) -> Result<(), MedialAxisError> {
    let mut queue = Vec::new();
    for index in 0..vd.num_edges() {
        if out.edges[index] != EdgeCategory::Unknown {
            continue;
        }
        let edge = diagram::edge_index(vd, index);
        let twin = diagram::twin(vd, edge)?;
        let left = diagram::cell(vd, edge)?;
        let right = diagram::cell(vd, twin)?;
        let mut category = known(out.cells[left.usize()]).or(known(out.cells[right.usize()]));
        if category.is_none()
            && let Some(vertex) = vd.edge_get_vertex0(edge).map_err(invariant)?
        {
            category = match out.vertices[vertex.usize()] {
                VertexCategory::Inside => Some(CellCategory::Inside),
                VertexCategory::Outside => Some(CellCategory::Outside),
                _ => None,
            };
        }
        if let Some(category) = category {
            mark_edge(vd, edge, category, out)?;
            enqueue_unknown_cells(out, &mut queue, [left, right], category);
        }
    }
    while let Some(cell) = queue.pop() {
        propagate_cell(vd, cell, out, &mut queue)?;
    }
    Ok(())
}

fn enqueue_unknown_cells<const N: usize>(
    out: &mut Annotations,
    queue: &mut Vec<boostvoronoi::prelude::CellIndex>,
    cells: [boostvoronoi::prelude::CellIndex; N],
    category: CellCategory,
) {
    for cell in cells {
        if out.cells[cell.usize()] == CellCategory::Unknown {
            out.cells[cell.usize()] = category;
            queue.push(cell);
        }
    }
}

fn propagate_cell(
    vd: &Diagram,
    cell: boostvoronoi::prelude::CellIndex,
    out: &mut Annotations,
    queue: &mut Vec<boostvoronoi::prelude::CellIndex>,
) -> Result<(), MedialAxisError> {
    let category = out.cells[cell.usize()];
    let Some(first) = vd.cell(cell).map_err(invariant)?.get_incident_edge() else {
        return Ok(());
    };
    let mut edge = first;
    loop {
        if out.edges[edge.usize()] == EdgeCategory::Unknown {
            propagate_edge(vd, edge, category, out, queue)?;
        }
        edge = vd.edge_get_next(edge).map_err(invariant)?;
        if edge == first {
            break;
        }
    }
    Ok(())
}

fn propagate_edge(
    vd: &Diagram,
    edge: boostvoronoi::prelude::EdgeIndex,
    category: CellCategory,
    out: &mut Annotations,
    queue: &mut Vec<boostvoronoi::prelude::CellIndex>,
) -> Result<(), MedialAxisError> {
    mark_edge(vd, edge, category, out)?;
    let other = diagram::cell(vd, diagram::twin(vd, edge)?)?;
    enqueue_unknown_cells(out, queue, [other], category);
    Ok(())
}

fn mark_edge(
    vd: &Diagram,
    edge: boostvoronoi::prelude::EdgeIndex,
    category: CellCategory,
    out: &mut Annotations,
) -> Result<(), MedialAxisError> {
    let twin = diagram::twin(vd, edge)?;
    let (edge_category, vertex_category) = match category {
        CellCategory::Inside => (EdgeCategory::Inside, VertexCategory::Inside),
        CellCategory::Outside => (EdgeCategory::Outside, VertexCategory::Outside),
        _ => return Err(MedialAxisError::ConstructionFailed),
    };
    out.edges[edge.usize()] = edge_category;
    out.edges[twin.usize()] = edge_category;
    for vertex in [
        vd.edge_get_vertex0(edge).map_err(invariant)?,
        vd.edge_get_vertex1(edge).map_err(invariant)?,
    ]
    .into_iter()
    .flatten()
    {
        if out.vertices[vertex.usize()] == VertexCategory::Unknown {
            out.vertices[vertex.usize()] = vertex_category;
        }
    }
    Ok(())
}

fn known(category: CellCategory) -> Option<CellCategory> {
    match category {
        CellCategory::Inside | CellCategory::Outside => Some(category),
        CellCategory::Unknown | CellCategory::Boundary => None,
    }
}

fn invariant(_: boostvoronoi::BvError) -> MedialAxisError {
    MedialAxisError::ConstructionFailed
}
