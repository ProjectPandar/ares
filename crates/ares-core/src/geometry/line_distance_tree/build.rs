use crate::geometry::{Line, Point};

use super::{Bounds, Node, NodeKind};

#[derive(Clone, Copy)]
struct Input {
    index: usize,
    bounds: Bounds,
    centroid: Point,
}

pub(super) fn build(lines: &[Line]) -> Vec<Node> {
    if lines.is_empty() {
        return Vec::new();
    }
    let mut input = lines
        .iter()
        .enumerate()
        .map(|(index, line)| Input {
            index,
            bounds: line_bounds(*line),
            centroid: line_centroid(*line),
        })
        .collect::<Vec<_>>();
    let mut nodes = vec![Node::UNUSED; input.len().next_power_of_two() * 2 - 1];
    let right = input.len() - 1;
    build_recursive(&mut input, &mut nodes, 0, 0, right);
    nodes
}

fn build_recursive(
    input: &mut [Input],
    nodes: &mut [Node],
    node_index: usize,
    left: usize,
    right: usize,
) {
    if left == right {
        let item = input[left];
        nodes[node_index] = Node {
            kind: NodeKind::Leaf(item.index),
            bounds: item.bounds,
        };
        return;
    }

    let bounds = range_bounds(input, left, right);
    let x_span = i128::from(bounds.max.x()) - i128::from(bounds.min.x());
    let y_span = i128::from(bounds.max.y()) - i128::from(bounds.min.y());
    let dimension = usize::from(y_span > x_span);
    let center = (left + right) / 2;
    partition_input(input, dimension, left, right, center);
    nodes[node_index] = Node {
        kind: NodeKind::Inner,
        bounds,
    };
    build_recursive(input, nodes, node_index * 2 + 1, left, center);
    build_recursive(input, nodes, node_index * 2 + 2, center + 1, right);
}

fn partition_input(
    input: &mut [Input],
    dimension: usize,
    mut left: usize,
    mut right: usize,
    k: usize,
) {
    while left < right {
        let center = (left + right) / 2;
        let mut left_value = coordinate(input[left].centroid, dimension);
        let mut center_value = coordinate(input[center].centroid, dimension);
        let mut right_value = coordinate(input[right].centroid, dimension);
        if left_value > center_value {
            input.swap(left, center);
            std::mem::swap(&mut left_value, &mut center_value);
        }
        if left_value > right_value {
            input.swap(left, right);
            right_value = left_value;
        }
        if center_value > right_value {
            input.swap(center, right);
            center_value = right_value;
        }
        let pivot = center_value;
        if right <= left + 2 {
            break;
        }

        let mut i = left;
        let mut j = right - 1;
        input.swap(center, j);
        loop {
            i += 1;
            while coordinate(input[i].centroid, dimension) < pivot {
                i += 1;
            }
            j -= 1;
            while coordinate(input[j].centroid, dimension) > pivot && i < j {
                j -= 1;
            }
            if i >= j {
                break;
            }
            input.swap(i, j);
        }
        input.swap(i, right - 1);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}

fn range_bounds(input: &[Input], left: usize, right: usize) -> Bounds {
    let mut bounds = input[left].bounds;
    for item in &input[left + 1..=right] {
        bounds.min = Point::new(
            bounds.min.x().min(item.bounds.min.x()),
            bounds.min.y().min(item.bounds.min.y()),
        );
        bounds.max = Point::new(
            bounds.max.x().max(item.bounds.max.x()),
            bounds.max.y().max(item.bounds.max.y()),
        );
    }
    bounds
}

fn line_bounds(line: Line) -> Bounds {
    Bounds {
        min: Point::new(line.a.x().min(line.b.x()), line.a.y().min(line.b.y())),
        max: Point::new(line.a.x().max(line.b.x()), line.a.y().max(line.b.y())),
    }
}

fn line_centroid(line: Line) -> Point {
    Point::new(
        midpoint(line.a.x(), line.b.x()),
        midpoint(line.a.y(), line.b.y()),
    )
}

fn midpoint(left: i64, right: i64) -> i64 {
    ((i128::from(left) + i128::from(right)) as f64 * 0.5) as i64
}

#[cfg(test)]
pub(super) fn centroids_for_test(lines: &[Line]) -> Vec<Point> {
    lines.iter().copied().map(line_centroid).collect()
}

fn coordinate(point: Point, dimension: usize) -> i64 {
    if dimension == 0 { point.x() } else { point.y() }
}
