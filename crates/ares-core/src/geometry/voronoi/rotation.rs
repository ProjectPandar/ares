use super::{Diagram, cell_range};
use crate::geometry::{Line, Point};

// Point.hpp236-247; no truncation, FMA, or different angle constants.
pub(super) fn sites(lines: &[Line], angle: f64) -> Vec<Line> {
    let cos = angle.cos();
    let sin = angle.sin();
    let rotate = |p: Point| {
        let x = p.x() as f64;
        let y = p.y() as f64;
        Point::new(
            (cos * x - sin * y).round() as i64,
            (cos * y + sin * x).round() as i64,
        )
    };
    lines
        .iter()
        .map(|line| Line::new(rotate(line.a), rotate(line.b)))
        .collect()
}

// Voronoi.cpp263-283,316-361. Mark ownership follows source cell order.
pub(super) fn remap(diagram: &mut Diagram, rotated: &[Line], original: &[Line], angle: f64) {
    let mut marks = vec![0_usize; diagram.vertices().len()];
    for cell in diagram.cells() {
        if cell.is_degenerate() || !cell.contains_segment() {
            continue;
        }
        let source = cell.source_index().usize();
        if let Some((begin, end)) = cell_range::segment(diagram, cell, rotated[source]) {
            let low = diagram.edge_get_vertex1(end).unwrap().unwrap().usize();
            let high = diagram.edge_get_vertex0(begin).unwrap().unwrap().usize();
            if marks[low] == 0 {
                marks[low] = (source + 1) << 1;
            }
            if marks[high] == 0 {
                marks[high] = ((source + 1) << 1) | 1;
            }
        }
    }
    let cos = (-angle).cos();
    let sin = (-angle).sin();
    for (position, mark) in diagram.local.as_mut().unwrap().iter_mut().zip(&mut marks) {
        if *mark == 0 {
            let [x, y] = *position;
            *position = [cos * x - sin * y, cos * y + sin * x];
        } else {
            let line = original[(*mark >> 1) - 1];
            let point = if *mark & 1 != 0 { line.b } else { line.a };
            *position = [point.x() as f64, point.y() as f64];
        }
        *mark = 0;
    }
}
