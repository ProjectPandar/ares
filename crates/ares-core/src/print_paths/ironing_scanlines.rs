use crate::Point2;

const EPSILON: f64 = 1e-9;

#[derive(Clone, Copy)]
pub(super) struct RectangleBounds {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

pub(super) fn rectilinear_scanlines(
    bounds: RectangleBounds,
    spacing_mm: f64,
    angle_degrees: f64,
) -> Vec<Vec<Point2>> {
    let angle = angle_degrees.to_radians();
    let direction = Vector2::new(angle.cos(), angle.sin());
    let normal = Vector2::new(-angle.sin(), angle.cos());
    let corners = bounds.corners();
    let min_offset = corners
        .iter()
        .map(|point| normal.project(*point))
        .fold(f64::INFINITY, f64::min);
    let max_offset = corners
        .iter()
        .map(|point| normal.project(*point))
        .fold(f64::NEG_INFINITY, f64::max);

    let mut lines = Vec::new();
    let mut offset = min_offset;
    while offset <= max_offset + EPSILON {
        if let Some(points) = clipped_scanline(bounds, direction, normal, offset) {
            lines.push(points);
        }
        let next_offset = offset + spacing_mm;
        if next_offset <= offset {
            break;
        }
        offset = next_offset;
    }
    lines
}

fn clipped_scanline(
    bounds: RectangleBounds,
    direction: Vector2,
    normal: Vector2,
    offset: f64,
) -> Option<Vec<Point2>> {
    let mut intersections = Vec::new();
    push_vertical_intersection(&mut intersections, bounds, normal, offset, bounds.min_x);
    push_vertical_intersection(&mut intersections, bounds, normal, offset, bounds.max_x);
    push_horizontal_intersection(&mut intersections, bounds, normal, offset, bounds.min_y);
    push_horizontal_intersection(&mut intersections, bounds, normal, offset, bounds.max_y);
    intersections.sort_by(|left, right| {
        direction
            .project(*left)
            .partial_cmp(&direction.project(*right))
            .expect("scanline projections are finite")
    });
    intersections.dedup_by(|left, right| point_eq(*left, *right));
    if intersections.len() < 2 {
        return None;
    }
    Some(vec![
        intersections[0],
        intersections[intersections.len() - 1],
    ])
}

fn push_vertical_intersection(
    intersections: &mut Vec<Point2>,
    bounds: RectangleBounds,
    normal: Vector2,
    offset: f64,
    x: f64,
) {
    if normal.y.abs() <= EPSILON {
        return;
    }
    let y = (offset - normal.x * x) / normal.y;
    if y >= bounds.min_y - EPSILON && y <= bounds.max_y + EPSILON {
        intersections.push(Point2::new(clean_zero(x), clean_zero(y)));
    }
}

fn push_horizontal_intersection(
    intersections: &mut Vec<Point2>,
    bounds: RectangleBounds,
    normal: Vector2,
    offset: f64,
    y: f64,
) {
    if normal.x.abs() <= EPSILON {
        return;
    }
    let x = (offset - normal.y * y) / normal.x;
    if x >= bounds.min_x - EPSILON && x <= bounds.max_x + EPSILON {
        intersections.push(Point2::new(clean_zero(x), clean_zero(y)));
    }
}

fn point_eq(left: Point2, right: Point2) -> bool {
    (left.x() - right.x()).abs() <= EPSILON && (left.y() - right.y()).abs() <= EPSILON
}

fn clean_zero(value: f64) -> f64 {
    if value.abs() <= EPSILON { 0.0 } else { value }
}

impl RectangleBounds {
    fn corners(self) -> [Point2; 4] {
        [
            Point2::new(self.min_x, self.min_y),
            Point2::new(self.max_x, self.min_y),
            Point2::new(self.max_x, self.max_y),
            Point2::new(self.min_x, self.max_y),
        ]
    }
}

#[derive(Clone, Copy)]
struct Vector2 {
    x: f64,
    y: f64,
}

impl Vector2 {
    const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn project(self, point: Point2) -> f64 {
        self.x * point.x() + self.y * point.y()
    }
}
