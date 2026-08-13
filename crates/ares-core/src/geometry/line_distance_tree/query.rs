use crate::geometry::{Line, Point};

use super::{Bounds, NearestLine, Node, NodeKind};

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
