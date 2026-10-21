//! MultiMaterialSegmentation.cpp::project_line_on_line491–512 / visitor522–588.

use super::{
    math::{dot, vector},
    painted_lines::{PaintedLine, distance2},
};
use crate::geometry::{ClipperError, EdgeGrid, Line, Point};
use std::collections::HashSet;

fn project_on_line(line: Line, projected: Line) -> Option<Line> {
    let v = vector(line.a, line.b);
    let length2 = dot(v, v);
    if length2 == 0.0 {
        return None;
    }
    let project = |p| {
        let t = (dot(vector(line.a, p), v) / length2).clamp(0.0, 1.0);
        Point::new(
            line.a.x() + (t * v[0]) as i64,
            line.a.y() + (t * v[1]) as i64,
        )
    };
    Some(Line::new(project(projected.a), project(projected.b)))
}

pub(super) fn visit(
    grid: &EdgeGrid,
    line: Line,
    color: usize,
    threshold: f64,
    output: &mut Vec<PaintedLine>,
) -> Result<(), ClipperError> {
    let mut seen = HashSet::new();
    seen.reserve(16);
    let v1 = vector(line.a, line.b);
    let length2 = dot(v1, v1);
    let heuristic = line.length() + threshold;
    let threshold2 = threshold * threshold;
    let cos2 = (std::f64::consts::PI * 30.0 / 180.0).cos().powi(2);
    grid.visit_cells_intersecting_line(line.a, line.b, |_, _, edges| {
        for edge in edges {
            let (a, b) = grid.segment(*edge);
            let contour = Line::new(a, b);
            let v2 = vector(a, b);
            let limit = (heuristic + contour.length()).powi(2);
            if distance2(a, line.a) > limit
                || distance2(b, line.a) > limit
                || distance2(a, line.b) > limit
                || distance2(b, line.b) > limit
            {
                continue;
            }
            let key = (edge.contour_index, edge.segment_index);
            if !(dot(v1, v2).powi(2) > cos2 * length2 * dot(v2, v2)
                && !seen.contains(&key)
                && (contour.distance_to_squared(line.a) < threshold2
                    || contour.distance_to_squared(line.b) < threshold2
                    || line.distance_to_squared(a) < threshold2
                    || line.distance_to_squared(b) < threshold2))
            {
                continue;
            }
            let mut projected = project_on_line(contour, line).unwrap();
            if distance2(projected.a, a) > distance2(projected.b, a) {
                std::mem::swap(&mut projected.a, &mut projected.b);
            }
            seen.insert(key);
            output.push(PaintedLine {
                contour: key.0,
                segment: key.1,
                line: projected,
                color,
            });
        }
        true
    })
}
