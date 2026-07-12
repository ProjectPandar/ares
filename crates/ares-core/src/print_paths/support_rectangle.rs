use super::{PrintPath, PrintPathRole};
use crate::Point2;

pub(crate) const EPSILON: f64 = 1e-9;

#[derive(Clone, Copy)]
pub(crate) struct RectangleBounds {
    pub(crate) min_x: f64,
    pub(crate) min_y: f64,
    pub(crate) max_x: f64,
    pub(crate) max_y: f64,
}

pub(crate) fn rectangle_bounds(points: &[Point2]) -> Option<RectangleBounds> {
    if points.len() != 4 {
        return None;
    }

    let min_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::NEG_INFINITY, f64::max);
    if max_x - min_x <= EPSILON || max_y - min_y <= EPSILON {
        return None;
    }
    let corners = [
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ];
    let reversed = [corners[0], corners[3], corners[2], corners[1]];
    (cyclic_matches(points, corners) || cyclic_matches(points, reversed)).then_some(
        RectangleBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        },
    )
}

pub(crate) fn rectangle_points(bounds: RectangleBounds) -> Vec<Point2> {
    vec![
        Point2::new(bounds.min_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.max_y),
        Point2::new(bounds.min_x, bounds.max_y),
    ]
}

pub(crate) fn rebuild_path(
    path: &PrintPath,
    role: PrintPathRole,
    points: Vec<Point2>,
    closed: bool,
) -> PrintPath {
    rebuild_path_with_extrusion_role(path, role, points, closed, path.extrusion_role())
}

pub(crate) fn rebuild_path_without_extrusion_role(
    path: &PrintPath,
    role: PrintPathRole,
    points: Vec<Point2>,
    closed: bool,
) -> PrintPath {
    rebuild_path_with_extrusion_role(path, role, points, closed, None)
}

fn rebuild_path_with_extrusion_role(
    path: &PrintPath,
    role: PrintPathRole,
    points: Vec<Point2>,
    closed: bool,
    extrusion_role: Option<PrintPathRole>,
) -> PrintPath {
    let rebuilt = PrintPath::new(role, points)
        .expect("existing print path points are non-empty")
        .with_effective_line_width_mm(path.effective_line_width_mm())
        .with_unsupported_span_mm(path.unsupported_span_mm())
        .with_seam_gap_mm(path.seam_gap_mm())
        .with_closed(closed);
    let rebuilt = if let Some(height) = path.effective_layer_height_mm() {
        rebuilt.with_effective_layer_height_mm(height)
    } else {
        rebuilt
    };
    if let Some(role) = extrusion_role {
        rebuilt.with_extrusion_role(role)
    } else {
        rebuilt
    }
}

fn cyclic_matches(points: &[Point2], corners: [Point2; 4]) -> bool {
    (0..4).any(|start| {
        points
            .iter()
            .enumerate()
            .all(|(index, point)| points_eq(*point, corners[(start + index) % 4]))
    })
}

fn points_eq(left: Point2, right: Point2) -> bool {
    scalar_eq(left.x(), right.x()) && scalar_eq(left.y(), right.y())
}

fn scalar_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= EPSILON
}
