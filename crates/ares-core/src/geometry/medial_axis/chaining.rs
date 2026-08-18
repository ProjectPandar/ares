use boostvoronoi::prelude::{Diagram, EdgeIndex};

use crate::geometry::{Point, ThickPolyline};

use super::{MedialAxisError, diagram, validate::EdgeData};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NeighborSelection {
    None,
    One(usize),
    Multiple,
}

pub(crate) fn chain(
    vd: &Diagram,
    data: &mut [EdgeData],
) -> Result<Vec<ThickPolyline>, MedialAxisError> {
    let mut output = Vec::new();
    let mut reverse = ThickPolyline::default();
    for index in (0..vd.num_edges()).step_by(2) {
        if !data[index / 2].active {
            continue;
        }
        data[index / 2].active = false;
        let edge = diagram::edge_index(vd, index);
        let start = point(diagram::vertex0(vd, edge)?.ok_or(MedialAxisError::ConstructionFailed)?);
        let end = point(diagram::vertex1(vd, edge)?.ok_or(MedialAxisError::ConstructionFailed)?);
        let edge_data = data[index / 2];
        let mut polyline = ThickPolyline {
            points: vec![start, end],
            width: vec![edge_data.width_start, edge_data.width_end],
            endpoints: (false, false),
        };
        grow(vd, edge, data, &mut polyline)?;
        reverse.clear();
        grow(vd, diagram::twin(vd, edge)?, data, &mut reverse)?;
        polyline.points.splice(0..0, reverse.points.drain(..).rev());
        polyline.width.splice(0..0, reverse.width.drain(..).rev());
        polyline.endpoints.0 = reverse.endpoints.1;
        suppress_closed_endpoints(&mut polyline);
        output.push(polyline);
    }
    Ok(output)
}

fn grow(
    vd: &Diagram,
    mut edge: EdgeIndex,
    data: &mut [EdgeData],
    polyline: &mut ThickPolyline,
) -> Result<(), MedialAxisError> {
    loop {
        let twin = diagram::twin(vd, edge)?;
        let mut neighbor = vd
            .edge_rot_next(twin)
            .ok_or(MedialAxisError::ConstructionFailed)?;
        let mut selection = NeighborSelection::None;
        while neighbor != twin {
            selection = select_active_neighbor(selection, neighbor.usize(), data);
            neighbor = vd
                .edge_rot_next(neighbor)
                .ok_or(MedialAxisError::ConstructionFailed)?;
        }
        match selection {
            NeighborSelection::One(index) => {
                let next = diagram::edge_index(vd, index);
                append_neighbor(vd, next, &mut data[index / 2], polyline)?;
                edge = next;
                continue;
            }
            NeighborSelection::None => polyline.endpoints.1 = true,
            NeighborSelection::Multiple => {}
        }
        break;
    }
    Ok(())
}

fn append_neighbor(
    vd: &Diagram,
    edge: EdgeIndex,
    stored: &mut EdgeData,
    polyline: &mut ThickPolyline,
) -> Result<(), MedialAxisError> {
    stored.active = false;
    let end = diagram::vertex1(vd, edge)?.ok_or(MedialAxisError::ConstructionFailed)?;
    polyline.points.push(point(end));
    let widths = directed_widths(edge.usize(), *stored);
    polyline.width.extend(widths);
    Ok(())
}

pub(crate) fn select_active_neighbor(
    selection: NeighborSelection,
    index: usize,
    data: &[EdgeData],
) -> NeighborSelection {
    if !data[index / 2].active {
        return selection;
    }
    match selection {
        NeighborSelection::None => NeighborSelection::One(index),
        NeighborSelection::One(_) | NeighborSelection::Multiple => NeighborSelection::Multiple,
    }
}

pub(crate) fn directed_widths(edge_index: usize, stored: EdgeData) -> [f64; 2] {
    if edge_index.is_multiple_of(2) {
        [stored.width_start, stored.width_end]
    } else {
        [stored.width_end, stored.width_start]
    }
}

pub(crate) fn suppress_closed_endpoints(polyline: &mut ThickPolyline) {
    if polyline.points.first() == polyline.points.last() {
        polyline.endpoints = (false, false);
    }
}

fn point((x, y): (f64, f64)) -> Point {
    super::validate::integer_point(x, y)
}
