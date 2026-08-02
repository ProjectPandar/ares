use boostvoronoi::prelude::{CellIndex, Diagram, EdgeIndex, SourceCategory};

use crate::geometry::Line;

use super::{MedialAxisError, diagram};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VertexCategory {
    Unknown,
    Inside,
    Outside,
    OnContour,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EdgeCategory {
    Unknown,
    Inside,
    Outside,
    ToContour,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CellCategory {
    Unknown,
    Inside,
    Outside,
    Boundary,
}

pub(crate) struct Annotations {
    pub(crate) vertices: Vec<VertexCategory>,
    pub(crate) edges: Vec<EdgeCategory>,
    pub(crate) cells: Vec<CellCategory>,
}

pub(crate) fn annotate(vd: &Diagram, lines: &[Line]) -> Result<Annotations, MedialAxisError> {
    let mut out = Annotations {
        vertices: vec![VertexCategory::Unknown; vd.num_vertices()],
        edges: vec![EdgeCategory::Unknown; vd.num_edges()],
        cells: vec![CellCategory::Unknown; vd.num_cells()],
    };
    mark_contour_vertices(vd, lines, &mut out)?;
    classify_segment_edges(vd, lines, &mut out)?;
    propagate::propagate_point_edges(vd, &mut out)?;
    Ok(out)
}

fn mark_contour_vertices(
    vd: &Diagram,
    lines: &[Line],
    out: &mut Annotations,
) -> Result<(), MedialAxisError> {
    for index in 0..vd.num_edges() {
        let edge = diagram::edge_index(vd, index);
        let Some(vertex) = vd.edge_get_vertex0(edge).map_err(invariant)? else {
            continue;
        };
        let cell = vd.cell(diagram::cell(vd, edge)?).map_err(invariant)?;
        let point = vd.vertex(vertex).map_err(invariant)?;
        let line = lines[cell.source_index().usize()];
        let on_site = match cell.source_category() {
            SourceCategory::SegmentStart => equals(point.x(), point.y(), line.a),
            SourceCategory::SegmentEnd => equals(point.x(), point.y(), line.b),
            SourceCategory::Segment => {
                equals(point.x(), point.y(), line.a) || equals(point.x(), point.y(), line.b)
            }
            SourceCategory::SinglePoint => false,
        };
        if on_site {
            out.vertices[vertex.usize()] = VertexCategory::OnContour;
        }
        if vd.edge(edge).map_err(invariant)?.is_secondary() {
            let twin = diagram::twin(vd, edge)?;
            let left = vd.cell(diagram::cell(vd, edge)?).map_err(invariant)?;
            let right = vd.cell(diagram::cell(vd, twin)?).map_err(invariant)?;
            let point_cell = if left.contains_point() { left } else { right };
            let source = lines[point_cell.source_index().usize()];
            let site = endpoint(source, point_cell.source_category())?;
            if equals(point.x(), point.y(), site) {
                out.vertices[vertex.usize()] = VertexCategory::OnContour;
            }
        }
    }
    Ok(())
}

fn classify_segment_edges(
    vd: &Diagram,
    lines: &[Line],
    out: &mut Annotations,
) -> Result<(), MedialAxisError> {
    for index in 0..vd.num_edges() {
        let edge = diagram::edge_index(vd, index);
        let twin = diagram::twin(vd, edge)?;
        let v0 = vd.edge_get_vertex0(edge).map_err(invariant)?;
        let v1 = vd.edge_get_vertex1(edge).map_err(invariant)?;
        if v1.is_none() {
            classify_infinite_edge(
                vd,
                InfiniteEdge {
                    edge,
                    twin,
                    index,
                    vertex: v0,
                },
                out,
            )?;
            continue;
        }
        let (Some(v0), Some(v1)) = (v0, v1) else {
            continue;
        };
        let left_id = diagram::cell(vd, edge)?;
        let right_id = diagram::cell(vd, twin)?;
        let left = vd.cell(left_id).map_err(invariant)?;
        let right = vd.cell(right_id).map_err(invariant)?;
        let (segment_cell, other_cell) = if left.contains_segment() {
            (left, right)
        } else if right.contains_segment() {
            (right, left)
        } else {
            continue;
        };
        let line = lines[segment_cell.source_index().usize()];
        let v1_point = vd.vertex(v1).map_err(invariant)?;
        let on_contour = out.vertices[v0.usize()] == VertexCategory::OnContour
            || out.vertices[v1.usize()] == VertexCategory::OnContour;
        if on_contour && out.vertices[v1.usize()] == VertexCategory::OnContour {
            out.edges[index] = EdgeCategory::ToContour;
            continue;
        }
        let side = (v1_point.x() - line.a.x() as f64) * (line.b.y() - line.a.y()) as f64
            - (v1_point.y() - line.a.y() as f64) * (line.b.x() - line.a.x()) as f64;
        let vertex_category = if side > 0.0 {
            VertexCategory::Outside
        } else {
            VertexCategory::Inside
        };
        let edge_category = if vertex_category == VertexCategory::Inside {
            EdgeCategory::Inside
        } else {
            EdgeCategory::Outside
        };
        out.vertices[v1.usize()] = vertex_category;
        out.edges[index] = edge_category;
        out.vertices[v0.usize()] = if on_contour {
            VertexCategory::OnContour
        } else {
            vertex_category
        };
        out.edges[twin.usize()] = if on_contour {
            EdgeCategory::ToContour
        } else {
            edge_category
        };
        annotate_cell(
            out,
            segment_cell.id(),
            if on_contour {
                CellCategory::Boundary
            } else {
                cell_for(vertex_category)
            },
        );
        annotate_cell(
            out,
            other_cell.id(),
            if on_contour && other_cell.contains_segment() {
                CellCategory::Boundary
            } else {
                cell_for(vertex_category)
            },
        );
    }
    Ok(())
}

struct InfiniteEdge {
    edge: EdgeIndex,
    twin: EdgeIndex,
    index: usize,
    vertex: Option<boostvoronoi::prelude::VertexIndex>,
}

fn classify_infinite_edge(
    vd: &Diagram,
    infinite: InfiniteEdge,
    out: &mut Annotations,
) -> Result<(), MedialAxisError> {
    let secondary = vd.edge(infinite.edge).map_err(invariant)?.is_secondary();
    out.edges[infinite.index] = EdgeCategory::Outside;
    out.edges[infinite.twin.usize()] = if secondary {
        EdgeCategory::ToContour
    } else {
        EdgeCategory::Outside
    };
    if let Some(vertex) = infinite.vertex {
        out.vertices[vertex.usize()] = if secondary {
            VertexCategory::OnContour
        } else {
            VertexCategory::Outside
        };
    }
    seed_infinite_cells(vd, infinite.edge, out)
}

fn seed_infinite_cells(
    vd: &Diagram,
    edge: EdgeIndex,
    out: &mut Annotations,
) -> Result<(), MedialAxisError> {
    let twin = diagram::twin(vd, edge)?;
    let left = vd.cell(diagram::cell(vd, edge)?).map_err(invariant)?;
    let right = vd.cell(diagram::cell(vd, twin)?).map_err(invariant)?;
    for cell in [left, right] {
        annotate_cell(
            out,
            cell.id(),
            if cell.contains_segment() {
                CellCategory::Boundary
            } else {
                CellCategory::Outside
            },
        );
    }
    Ok(())
}

fn annotate_cell(out: &mut Annotations, id: CellIndex, category: CellCategory) {
    let old = out.cells[id.usize()];
    out.cells[id.usize()] = merge_cell_category(old, category);
}

pub(crate) fn merge_cell_category(old: CellCategory, category: CellCategory) -> CellCategory {
    match (old, category) {
        (_, CellCategory::Boundary) | (CellCategory::Boundary, _) => CellCategory::Boundary,
        (CellCategory::Unknown, new) => new,
        (CellCategory::Inside, CellCategory::Outside)
        | (CellCategory::Outside, CellCategory::Inside) => CellCategory::Boundary,
        (old, _) => old,
    }
}

fn cell_for(category: VertexCategory) -> CellCategory {
    match category {
        VertexCategory::Inside => CellCategory::Inside,
        VertexCategory::Outside => CellCategory::Outside,
        _ => unreachable!(),
    }
}

fn endpoint(
    line: Line,
    category: SourceCategory,
) -> Result<crate::geometry::Point, MedialAxisError> {
    match category {
        SourceCategory::SegmentStart => Ok(line.a),
        SourceCategory::SegmentEnd => Ok(line.b),
        SourceCategory::Segment | SourceCategory::SinglePoint => {
            Err(MedialAxisError::ConstructionFailed)
        }
    }
}

fn equals(x: f64, y: f64, point: crate::geometry::Point) -> bool {
    vertex_equal_to_point(x, point.x() as f64) && vertex_equal_to_point(y, point.y() as f64)
}

pub(crate) fn vertex_equal_to_point(left: f64, right: f64) -> bool {
    ordered_bits(left).abs_diff(ordered_bits(right)) <= 64
}

fn ordered_bits(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits < 0x8000_0000_0000_0000 {
        0x8000_0000_0000_0000 - bits
    } else {
        bits
    }
}

fn invariant(_: boostvoronoi::BvError) -> MedialAxisError {
    MedialAxisError::ConstructionFailed
}

mod propagate;
