use crate::{InfillOptions, Point2, options::InfillLayerRole};

const EPSILON: f64 = 1e-9;

pub(super) fn filter_contours(
    role: InfillLayerRole,
    contours: Vec<Vec<Point2>>,
    options: &InfillOptions,
) -> Vec<Vec<Point2>> {
    let min_width_mm = options.min_width_top_surface_mm();
    if role != InfillLayerRole::TopSurface || min_width_mm <= EPSILON {
        return contours;
    }
    contours
        .into_iter()
        .filter(|points| match rectangle_width(points) {
            Some(width) => width + EPSILON >= min_width_mm,
            None => true,
        })
        .collect()
}

fn rectangle_width(points: &[Point2]) -> Option<f64> {
    let bounds = rectangle_bounds(points)?;
    is_cyclic_rectangle(points, bounds)
        .then_some((bounds.max_x - bounds.min_x).min(bounds.max_y - bounds.min_y))
}

fn rectangle_bounds(points: &[Point2]) -> Option<RectangleBounds> {
    if points.len() != 4 {
        return None;
    }
    let min_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::INFINITY, f64::min);
    let min_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::NEG_INFINITY, f64::max);
    if max_x - min_x <= EPSILON || max_y - min_y <= EPSILON {
        return None;
    }
    Some(RectangleBounds {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

#[derive(Clone, Copy)]
struct RectangleBounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

fn is_cyclic_rectangle(points: &[Point2], bounds: RectangleBounds) -> bool {
    let corners = [
        Point2::new(bounds.min_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.min_y),
        Point2::new(bounds.max_x, bounds.max_y),
        Point2::new(bounds.min_x, bounds.max_y),
    ];
    let reversed = [corners[0], corners[3], corners[2], corners[1]];
    cyclic_matches(points, corners) || cyclic_matches(points, reversed)
}

fn cyclic_matches(points: &[Point2], corners: [Point2; 4]) -> bool {
    (0..4).any(|start| {
        points
            .iter()
            .enumerate()
            .all(|(index, point)| point_eq(*point, corners[(start + index) % 4]))
    })
}

fn point_eq(left: Point2, right: Point2) -> bool {
    (left.x() - right.x()).abs() <= EPSILON && (left.y() - right.y()).abs() <= EPSILON
}
