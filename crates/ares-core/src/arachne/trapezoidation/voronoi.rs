use boostvoronoi::prelude::{Builder, CellIndex, Diagram, EdgeIndex, Line as BvLine};

use crate::geometry::{Point, Polygon};

use super::{TrapezoidationError, index::PolygonSegmentIndex};

pub(super) struct CellRange {
    pub(super) source_start: Point,
    pub(super) source_end: Point,
    pub(super) edge_begin: EdgeIndex,
    pub(super) edge_end: EdgeIndex,
}

pub(super) fn build(
    polygons: &[Polygon],
    segments: &[PolygonSegmentIndex],
) -> Result<Diagram, TrapezoidationError> {
    if polygons.iter().any(|polygon| polygon.points().len() < 3) {
        return Err(TrapezoidationError::EmptyPolygon);
    }
    let lines = segments
        .iter()
        .map(|segment| segment.line(polygons))
        .map(|line| {
            let coordinate =
                |value| i32::try_from(value).map_err(|_| TrapezoidationError::VoronoiConstruction);
            Ok(BvLine::from([
                coordinate(line.a.x())?,
                coordinate(line.a.y())?,
                coordinate(line.b.x())?,
                coordinate(line.b.y())?,
            ]))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Builder::<i32>::default()
        .with_segments(&lines)
        .and_then(Builder::build)
        .map_err(|_| TrapezoidationError::VoronoiConstruction)
}

pub(super) fn cell_range(
    vd: &Diagram,
    cell_id: CellIndex,
    polygons: &[Polygon],
    segments: &[PolygonSegmentIndex],
) -> Result<Option<CellRange>, TrapezoidationError> {
    let cell = vd.cell(cell_id).map_err(invalid)?;
    let Some(incident) = cell.get_incident_edge() else {
        return Ok(None);
    };
    let source = segments
        .get(cell.source_index().usize())
        .copied()
        .ok_or(TrapezoidationError::InvalidTopology)?;
    if cell.contains_point() {
        let Some(source_point) = source.source_point(polygons, cell.source_category()) else {
            return Err(TrapezoidationError::InvalidTopology);
        };
        let Some(source_index) = source.source_point_index(polygons, cell.source_category()) else {
            return Err(TrapezoidationError::InvalidTopology);
        };
        point_cell_range(vd, incident, source_point, source_index, polygons)
    } else if cell.contains_segment() {
        segment_cell_range(vd, incident, source, polygons)
    } else {
        Err(TrapezoidationError::InvalidTopology)
    }
}

fn point_cell_range(
    vd: &Diagram,
    incident: EdgeIndex,
    source: Point,
    source_index: PolygonSegmentIndex,
    polygons: &[Polygon],
) -> Result<Option<CellRange>, TrapezoidationError> {
    if !edge_in_range(vd, incident)? {
        return Ok(None);
    }
    let v0 = vertex0(vd, incident)?.ok_or(TrapezoidationError::InvalidTopology)?;
    let v1 = vertex1(vd, incident)?.ok_or(TrapezoidationError::InvalidTopology)?;
    let query = if v0 == source { v1 } else { v0 };
    if !inside_corner(source_index, query, polygons) {
        return Ok(None);
    }
    let mut edge = incident;
    let mut begin = None;
    let mut end = None;
    loop {
        if vertex1(vd, edge)? == Some(source) {
            begin = Some(next(vd, edge)?);
            end = Some(edge);
        }
        edge = next(vd, edge)?;
        if edge == incident {
            break;
        }
    }
    Ok(begin.zip(end).map(|(edge_begin, edge_end)| CellRange {
        source_start: source,
        source_end: source,
        edge_begin,
        edge_end,
    }))
}

fn segment_cell_range(
    vd: &Diagram,
    incident: EdgeIndex,
    source: PolygonSegmentIndex,
    polygons: &[Polygon],
) -> Result<Option<CellRange>, TrapezoidationError> {
    let from = source.from(polygons);
    let to = source.to(polygons);
    let mut begin = None;
    let mut end = None;
    let mut seen_possible_start = false;
    let mut after_start = false;
    let mut end_before_start = false;
    let mut edge = incident;
    loop {
        if vd.edge_is_finite(edge).map_err(invalid)? {
            let v0 = vertex0(vd, edge)?.ok_or(TrapezoidationError::InvalidTopology)?;
            let v1 = vertex1(vd, edge)?.ok_or(TrapezoidationError::InvalidTopology)?;
            if v0 == to && !after_start {
                begin = Some(edge);
                seen_possible_start = true;
            } else if seen_possible_start {
                after_start = true;
            }
            if v1 == from && (end.is_none() || end_before_start) {
                end_before_start = !after_start;
                end = Some(edge);
            }
        }
        edge = next(vd, edge)?;
        if edge == incident {
            break;
        }
    }
    Ok(begin.zip(end).map(|(edge_begin, edge_end)| CellRange {
        source_start: to,
        source_end: from,
        edge_begin,
        edge_end,
    }))
}

pub(super) fn vertex0(vd: &Diagram, edge: EdgeIndex) -> Result<Option<Point>, TrapezoidationError> {
    vertex(vd, vd.edge_get_vertex0(edge).map_err(invalid)?)
}

pub(super) fn vertex1(vd: &Diagram, edge: EdgeIndex) -> Result<Option<Point>, TrapezoidationError> {
    vertex(vd, vd.edge_get_vertex1(edge).map_err(invalid)?)
}

fn vertex(
    vd: &Diagram,
    vertex: Option<boostvoronoi::prelude::VertexIndex>,
) -> Result<Option<Point>, TrapezoidationError> {
    vertex
        .map(|id| {
            let vertex = vd.vertex(id).map_err(invalid)?;
            if !vertex.x().is_finite() || !vertex.y().is_finite() {
                return Err(TrapezoidationError::InvalidTopology);
            }
            Ok(Point::new(vertex.x() as i64, vertex.y() as i64))
        })
        .transpose()
}

pub(super) fn next(vd: &Diagram, edge: EdgeIndex) -> Result<EdgeIndex, TrapezoidationError> {
    vd.edge_get_next(edge).map_err(invalid)
}

pub(super) fn twin(vd: &Diagram, edge: EdgeIndex) -> Result<EdgeIndex, TrapezoidationError> {
    vd.edge_get_twin(edge).map_err(invalid)
}

pub(super) fn source_point(
    vd: &Diagram,
    edge: EdgeIndex,
    polygons: &[Polygon],
    segments: &[PolygonSegmentIndex],
) -> Result<Option<Point>, TrapezoidationError> {
    let cell = vd
        .cell(vd.edge_get_cell(edge).map_err(invalid)?)
        .map_err(invalid)?;
    Ok(segments[cell.source_index().usize()].source_point(polygons, cell.source_category()))
}

pub(super) fn source_segment(
    vd: &Diagram,
    edge: EdgeIndex,
    segments: &[PolygonSegmentIndex],
) -> Result<Option<PolygonSegmentIndex>, TrapezoidationError> {
    let cell = vd
        .cell(vd.edge_get_cell(edge).map_err(invalid)?)
        .map_err(invalid)?;
    Ok(cell
        .contains_segment()
        .then(|| segments[cell.source_index().usize()]))
}

pub(super) fn cell_contains_point(
    vd: &Diagram,
    edge: EdgeIndex,
) -> Result<bool, TrapezoidationError> {
    let cell = vd
        .cell(vd.edge_get_cell(edge).map_err(invalid)?)
        .map_err(invalid)?;
    Ok(cell.contains_point())
}

pub(super) fn is_secondary(vd: &Diagram, edge: EdgeIndex) -> Result<bool, TrapezoidationError> {
    vd.edge(edge)
        .map(|edge| edge.is_secondary())
        .map_err(invalid)
}

fn edge_in_range(vd: &Diagram, edge: EdgeIndex) -> Result<bool, TrapezoidationError> {
    Ok(vd.edge_is_finite(edge).map_err(invalid)?
        && vertex0(vd, edge)?.is_some()
        && vertex1(vd, edge)?.is_some())
}

fn inside_corner(index: PolygonSegmentIndex, query: Point, polygons: &[Polygon]) -> bool {
    let points = polygons[index.polygon_index].points();
    let current = points[index.point_index];
    let previous = points[(index.point_index + points.len() - 1) % points.len()];
    let next = points[(index.point_index + 1) % points.len()];
    let ba = normalized(previous, current);
    let bc = normalized(next, current);
    let bq = normalized(query, current);
    let left_normal = (-bq.1, bq.0);
    let a_on_normal = dot(ba, left_normal);
    let c_on_normal = dot(bc, left_normal);
    if (a_on_normal > 0.0 && c_on_normal <= 0.0) || (a_on_normal <= 0.0 && c_on_normal > 0.0) {
        a_on_normal > 0.0
    } else {
        let a_on_query = dot(ba, bq);
        let c_on_query = dot(bc, bq);
        (a_on_normal > 0.0 && c_on_query < a_on_query)
            || (a_on_normal <= 0.0 && c_on_query >= a_on_query)
    }
}

fn normalized(point: Point, origin: Point) -> (f64, f64) {
    let x = point.x() as f64 - origin.x() as f64;
    let y = point.y() as f64 - origin.y() as f64;
    let length = (x * x + y * y).sqrt();
    (x / length, y / length)
}

fn dot(left: (f64, f64), right: (f64, f64)) -> f64 {
    left.0 * right.0 + left.1 * right.1
}

fn invalid(_: boostvoronoi::BvError) -> TrapezoidationError {
    TrapezoidationError::InvalidTopology
}
