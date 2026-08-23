use crate::geometry::{Line, Point};

use super::{Bounds, NearestLine, NearestLineF32, Node, NodeKind};

pub(super) fn nearest(lines: &[Line], nodes: &[Node], point: Point) -> Option<NearestLine> {
    if nodes.is_empty() {
        return None;
    }
    let origin = [point.x() as f64, point.y() as f64];
    let rounded = Point::new(origin[0] as i64, origin[1] as i64);
    let mut winner = None;
    visit(lines, nodes, 0, origin, rounded, &mut winner);
    winner.map(|winner: Winner| NearestLine {
        line_index: winner.line_index,
        squared_distance: winner.squared_distance,
        nearest_point: [
            winner.nearest_point.x() as f64,
            winner.nearest_point.y() as f64,
        ],
    })
}

#[derive(Clone, Copy)]
struct Winner {
    line_index: usize,
    squared_distance: f64,
    nearest_point: Point,
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source traversal state"
)]
fn visit(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    origin: [f64; 2],
    rounded: Point,
    winner: &mut Option<Winner>,
) {
    let node = nodes[node_index];
    match node.kind {
        NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
        NodeKind::Leaf(line_index) => {
            let (squared_distance, nearest_point) = segment_distance(lines[line_index], rounded);
            if squared_distance < winner_distance(*winner) {
                *winner = Some(Winner {
                    line_index,
                    squared_distance,
                    nearest_point,
                });
            }
        }
        NodeKind::Inner => {
            let left_index = node_index * 2 + 1;
            let right_index = left_index + 1;
            let left = nodes[left_index];
            let right = nodes[right_index];
            let mut looked_left = false;
            let mut looked_right = false;

            if contains(left.bounds, rounded) {
                visit(lines, nodes, left_index, origin, rounded, winner);
                looked_left = true;
            }
            if contains(right.bounds, rounded) {
                visit(lines, nodes, right_index, origin, rounded, winner);
                looked_right = true;
            }

            let left_distance = exterior_distance_squared(left.bounds, origin);
            let right_distance = exterior_distance_squared(right.bounds, origin);
            if left_distance < right_distance {
                visit_if_closer(
                    lines,
                    nodes,
                    left_index,
                    origin,
                    rounded,
                    winner,
                    looked_left,
                    left_distance,
                );
                visit_if_closer(
                    lines,
                    nodes,
                    right_index,
                    origin,
                    rounded,
                    winner,
                    looked_right,
                    right_distance,
                );
            } else {
                visit_if_closer(
                    lines,
                    nodes,
                    right_index,
                    origin,
                    rounded,
                    winner,
                    looked_right,
                    right_distance,
                );
                visit_if_closer(
                    lines,
                    nodes,
                    left_index,
                    origin,
                    rounded,
                    winner,
                    looked_left,
                    left_distance,
                );
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source traversal state"
)]
fn visit_if_closer(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    origin: [f64; 2],
    rounded: Point,
    winner: &mut Option<Winner>,
    already_visited: bool,
    lower_bound: f64,
) {
    if !already_visited && lower_bound < winner_distance(*winner) {
        visit(lines, nodes, node_index, origin, rounded, winner);
    }
}

fn winner_distance(winner: Option<Winner>) -> f64 {
    winner.map_or(f64::INFINITY, |winner| winner.squared_distance)
}

fn contains(bounds: Bounds, point: Point) -> bool {
    point.x() >= bounds.min.x()
        && point.x() <= bounds.max.x()
        && point.y() >= bounds.min.y()
        && point.y() <= bounds.max.y()
}

pub(super) fn exterior_distance_squared(bounds: Bounds, origin: [f64; 2]) -> f64 {
    let x = exterior_delta(bounds.min.x(), bounds.max.x(), origin[0]);
    let y = exterior_delta(bounds.min.y(), bounds.max.y(), origin[1]);
    (i128::from(x) * i128::from(x) + i128::from(y) * i128::from(y)) as f64
}

fn exterior_delta(minimum: i64, maximum: i64, origin: f64) -> i64 {
    if (minimum as f64) > origin {
        (minimum as f64 - origin) as i64
    } else if (maximum as f64) < origin {
        (origin - maximum as f64) as i64
    } else {
        0
    }
}

fn segment_distance(line: Line, point: Point) -> (f64, Point) {
    let vx = (i128::from(line.b.x()) - i128::from(line.a.x())) as f64;
    let vy = (i128::from(line.b.y()) - i128::from(line.a.y())) as f64;
    let vax = (i128::from(point.x()) - i128::from(line.a.x())) as f64;
    let vay = (i128::from(point.y()) - i128::from(line.a.y())) as f64;
    let length_squared = vx * vx + vy * vy;
    if length_squared == 0.0 {
        return (vax * vax + vay * vay, line.a);
    }

    let t = (vax * vx + vay * vy) / length_squared;
    if t <= 0.0 {
        (vax * vax + vay * vay, line.a)
    } else if t >= 1.0 {
        let vbx = (i128::from(point.x()) - i128::from(line.b.x())) as f64;
        let vby = (i128::from(point.y()) - i128::from(line.b.y())) as f64;
        (vbx * vbx + vby * vby, line.b)
    } else {
        let nearest = Point::new(
            (line.a.x() as f64 + t * vx) as i64,
            (line.a.y() as f64 + t * vy) as i64,
        );
        let dx = t * vx - vax;
        let dy = t * vy - vay;
        (dx * dx + dy * dy, nearest)
    }
}

pub(super) fn nearest_f32(
    lines: &[Line],
    nodes: &[Node],
    origin: [f32; 2],
    scale: f32,
) -> Option<NearestLineF32> {
    if nodes.is_empty() {
        return None;
    }
    let mut winner = None;
    visit_f32(lines, nodes, 0, origin, scale, &mut winner);
    winner.map(|winner: FloatWinner| NearestLineF32 {
        line_index: winner.line_index,
        squared_distance: winner.squared_distance,
        nearest_point: winner.nearest_point,
    })
}

#[derive(Clone, Copy)]
struct FloatWinner {
    line_index: usize,
    squared_distance: f32,
    nearest_point: [f32; 2],
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source traversal state"
)]
fn visit_f32(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    origin: [f32; 2],
    scale: f32,
    winner: &mut Option<FloatWinner>,
) {
    let node = nodes[node_index];
    match node.kind {
        NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
        NodeKind::Leaf(line_index) => {
            let (squared_distance, nearest_point) =
                segment_distance_f32(lines[line_index], origin, scale);
            if squared_distance < float_winner_distance(*winner) {
                *winner = Some(FloatWinner {
                    line_index,
                    squared_distance,
                    nearest_point,
                });
            }
        }
        NodeKind::Inner => {
            let left_index = node_index * 2 + 1;
            let right_index = left_index + 1;
            let left = nodes[left_index];
            let right = nodes[right_index];
            let mut looked_left = false;
            let mut looked_right = false;

            if contains_f32(left.bounds, origin, scale) {
                visit_f32(lines, nodes, left_index, origin, scale, winner);
                looked_left = true;
            }
            if contains_f32(right.bounds, origin, scale) {
                visit_f32(lines, nodes, right_index, origin, scale, winner);
                looked_right = true;
            }

            let left_distance = exterior_distance_squared_f32(left.bounds, origin, scale);
            let right_distance = exterior_distance_squared_f32(right.bounds, origin, scale);
            if left_distance < right_distance {
                visit_f32_if_closer(
                    lines,
                    nodes,
                    left_index,
                    origin,
                    scale,
                    winner,
                    looked_left,
                    left_distance,
                );
                visit_f32_if_closer(
                    lines,
                    nodes,
                    right_index,
                    origin,
                    scale,
                    winner,
                    looked_right,
                    right_distance,
                );
            } else {
                visit_f32_if_closer(
                    lines,
                    nodes,
                    right_index,
                    origin,
                    scale,
                    winner,
                    looked_right,
                    right_distance,
                );
                visit_f32_if_closer(
                    lines,
                    nodes,
                    left_index,
                    origin,
                    scale,
                    winner,
                    looked_left,
                    left_distance,
                );
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "mirrors the source traversal state"
)]
fn visit_f32_if_closer(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    origin: [f32; 2],
    scale: f32,
    winner: &mut Option<FloatWinner>,
    already_visited: bool,
    lower_bound: f32,
) {
    if !already_visited && lower_bound < float_winner_distance(*winner) {
        visit_f32(lines, nodes, node_index, origin, scale, winner);
    }
}

fn float_winner_distance(winner: Option<FloatWinner>) -> f32 {
    winner.map_or(f32::INFINITY, |winner| winner.squared_distance)
}

fn contains_f32(bounds: Bounds, point: [f32; 2], scale: f32) -> bool {
    point[0] >= scaled_f32(bounds.min.x(), scale)
        && point[0] <= scaled_f32(bounds.max.x(), scale)
        && point[1] >= scaled_f32(bounds.min.y(), scale)
        && point[1] <= scaled_f32(bounds.max.y(), scale)
}

fn exterior_distance_squared_f32(bounds: Bounds, origin: [f32; 2], scale: f32) -> f32 {
    let delta = |minimum: i64, maximum: i64, value: f32| {
        let minimum = scaled_f32(minimum, scale);
        let maximum = scaled_f32(maximum, scale);
        if minimum > value {
            minimum - value
        } else if maximum < value {
            value - maximum
        } else {
            0.0
        }
    };
    let x = delta(bounds.min.x(), bounds.max.x(), origin[0]);
    let y = delta(bounds.min.y(), bounds.max.y(), origin[1]);
    x * x + y * y
}

fn segment_distance_f32(line: Line, point: [f32; 2], scale: f32) -> (f32, [f32; 2]) {
    let a = [scaled_f32(line.a.x(), scale), scaled_f32(line.a.y(), scale)];
    let b = [scaled_f32(line.b.x(), scale), scaled_f32(line.b.y(), scale)];
    let v = [(b[0] - a[0]) as f64, (b[1] - a[1]) as f64];
    let va = [(point[0] - a[0]) as f64, (point[1] - a[1]) as f64];
    let length_squared = v[0] * v[0] + v[1] * v[1];
    if length_squared == 0.0 {
        return ((va[0] * va[0] + va[1] * va[1]) as f32, a);
    }

    let t = (va[0] * v[0] + va[1] * v[1]) / length_squared;
    if t <= 0.0 {
        ((va[0] * va[0] + va[1] * va[1]) as f32, a)
    } else if t >= 1.0 {
        let vb = [(point[0] - b[0]) as f64, (point[1] - b[1]) as f64];
        ((vb[0] * vb[0] + vb[1] * vb[1]) as f32, b)
    } else {
        let nearest = [
            (f64::from(a[0]) + t * v[0]) as f32,
            (f64::from(a[1]) + t * v[1]) as f32,
        ];
        let delta = [t * v[0] - va[0], t * v[1] - va[1]];
        ((delta[0] * delta[0] + delta[1] * delta[1]) as f32, nearest)
    }
}

fn scaled_f32(coordinate: i64, scale: f32) -> f32 {
    coordinate as f32 * scale
}
