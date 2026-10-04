use crate::geometry::{Line, Point};

use super::{Bounds, MmQuery, Node, NodeKind};

pub(super) fn classify(lines: &[Line], nodes: &[Node], point: Point) -> i32 {
    if nodes.is_empty() {
        return 1;
    }
    let x_counts = ray_counts(lines, nodes, point, 0);
    match classify_counts(x_counts) {
        Some(classification) => classification,
        None => classify_counts(ray_counts(lines, nodes, point, 1)).unwrap_or(0),
    }
}

pub(super) fn ray_counts(
    lines: &[Line],
    nodes: &[Node],
    point: Point,
    coordinate: usize,
) -> (i32, i32) {
    if nodes.is_empty() {
        return (0, 0);
    }
    count_recursive(lines, nodes, 0, point, coordinate)
}

fn count_recursive(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    point: Point,
    coordinate: usize,
) -> (i32, i32) {
    let node = nodes[node_index];
    match node.kind {
        NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
        NodeKind::Leaf(line_index) => leaf_counts(lines[line_index], point, coordinate),
        NodeKind::Inner => inner_counts(lines, nodes, node_index, point, coordinate),
    }
}

fn inner_counts(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    point: Point,
    coordinate: usize,
) -> (i32, i32) {
    let other_coordinate = (coordinate + 1) % 2;
    let left_index = node_index * 2 + 1;
    let right_index = left_index + 1;
    let mut counts = (0, 0);
    for child_index in [left_index, right_index] {
        if !contains_coordinate(nodes[child_index].bounds, point, other_coordinate) {
            continue;
        }
        let child = count_recursive(lines, nodes, child_index, point, coordinate);
        if child.0 < 0 || child.1 < 0 {
            return (-1, -1);
        }
        counts.0 += child.0;
        counts.1 += child.1;
    }
    counts
}

fn leaf_counts(line: Line, point: Point, coordinate: usize) -> (i32, i32) {
    let other_coordinate = (coordinate + 1) % 2;
    let point_other = point_coordinate(point, other_coordinate);
    let line_a_other = point_coordinate(line.a, other_coordinate);
    let line_b_other = point_coordinate(line.b, other_coordinate);
    if point_other < line_a_other.min(line_b_other) || point_other >= line_a_other.max(line_b_other)
    {
        return (0, 0);
    }

    let point_coordinate_value = point_coordinate(point, coordinate);
    let line_a_coordinate = point_coordinate(line.a, coordinate);
    let line_b_coordinate = point_coordinate(line.b, coordinate);
    let line_max = line_a_coordinate.max(line_b_coordinate);
    let line_min = line_a_coordinate.min(line_b_coordinate);
    if point_coordinate_value > line_max {
        return (1, 0);
    }
    if point_coordinate_value < line_min {
        return (0, 1);
    }

    let distance_other = (line_b_other - line_a_other) as f64;
    let t = (point_other - line_a_other) as f64 / distance_other;
    let intersection =
        line_a_coordinate as f64 + t * (line_b_coordinate - line_a_coordinate) as f64;
    let origin = point_coordinate_value as f64;
    if origin > intersection {
        (1, 0)
    } else if origin < intersection {
        (0, 1)
    } else {
        (-1, -1)
    }
}

fn contains_coordinate(bounds: Bounds, point: Point, coordinate: usize) -> bool {
    let value = point_coordinate(point, coordinate);
    value >= point_coordinate(bounds.min, coordinate)
        && value <= point_coordinate(bounds.max, coordinate)
}

fn point_coordinate(point: Point, coordinate: usize) -> i64 {
    if coordinate == 0 {
        point.x()
    } else {
        point.y()
    }
}

fn classify_counts(counts: (i32, i32)) -> Option<i32> {
    if counts.0 < 0 || counts.1 < 0 {
        Some(0)
    } else if counts.0 % 2 == 1 && counts.1 % 2 == 1 {
        Some(-1)
    } else if counts.0 % 2 == 0 && counts.1 % 2 == 0 {
        Some(1)
    } else {
        None
    }
}

/// `point_outside_closed_contours` on the unscaled point — upstream
/// passes the f64 `Vec` directly (`AABBTreeLines.hpp:241,325,342`).
/// The ray-count math runs in millimetres against `coord *
/// scale_factor` endpoints so sub-unit positions classify correctly.
pub(super) fn classify_mm(lines: &[Line], nodes: &[Node], point: [f64; 2], scale: f64) -> i32 {
    if nodes.is_empty() {
        return 1;
    }
    let x_counts = ray_counts_mm(lines, nodes, point, 0, scale);
    match classify_counts(x_counts) {
        Some(classification) => classification,
        None => classify_counts(ray_counts_mm(lines, nodes, point, 1, scale)).unwrap_or(0),
    }
}

fn ray_counts_mm(
    lines: &[Line],
    nodes: &[Node],
    point: [f64; 2],
    coordinate: usize,
    scale: f64,
) -> (i32, i32) {
    if nodes.is_empty() {
        return (0, 0);
    }
    count_mm(lines, nodes, 0, MmQuery { point, scale }, coordinate)
}

fn count_mm(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    query: MmQuery,
    coordinate: usize,
) -> (i32, i32) {
    let MmQuery { point, scale } = query;
    let node = nodes[node_index];
    match node.kind {
        NodeKind::Unused => unreachable!("tree traversal only reaches valid nodes"),
        NodeKind::Leaf(line_index) => leaf_counts_mm(lines[line_index], point, coordinate, scale),
        NodeKind::Inner => inner_counts_mm(lines, nodes, node_index, query, coordinate),
    }
}

fn inner_counts_mm(
    lines: &[Line],
    nodes: &[Node],
    node_index: usize,
    query: MmQuery,
    coordinate: usize,
) -> (i32, i32) {
    let MmQuery { point, scale } = query;
    let other_coordinate = (coordinate + 1) % 2;
    let left_index = node_index * 2 + 1;
    let right_index = left_index + 1;
    let mut counts = (0, 0);
    for child_index in [left_index, right_index] {
        let bounds = nodes[child_index].bounds;
        let minimum = point_coordinate(bounds.min, other_coordinate) as f64 * scale;
        let maximum = point_coordinate(bounds.max, other_coordinate) as f64 * scale;
        let value = point[other_coordinate];
        if value < minimum || value > maximum {
            continue;
        }
        let child = count_mm(lines, nodes, child_index, query, coordinate);
        if child.0 < 0 || child.1 < 0 {
            return (-1, -1);
        }
        counts.0 += child.0;
        counts.1 += child.1;
    }
    counts
}

fn leaf_counts_mm(line: Line, point: [f64; 2], coordinate: usize, scale: f64) -> (i32, i32) {
    let other_coordinate = (coordinate + 1) % 2;
    let point_other = point[other_coordinate];
    let line_a_other = point_coordinate(line.a, other_coordinate) as f64 * scale;
    let line_b_other = point_coordinate(line.b, other_coordinate) as f64 * scale;
    if point_other < line_a_other.min(line_b_other) || point_other >= line_a_other.max(line_b_other)
    {
        return (0, 0);
    }

    let point_coordinate_value = point[coordinate];
    let line_a_coordinate = point_coordinate(line.a, coordinate) as f64 * scale;
    let line_b_coordinate = point_coordinate(line.b, coordinate) as f64 * scale;
    let line_max = line_a_coordinate.max(line_b_coordinate);
    let line_min = line_a_coordinate.min(line_b_coordinate);
    if point_coordinate_value > line_max {
        return (1, 0);
    }
    if point_coordinate_value < line_min {
        return (0, 1);
    }

    let t = (point_other - line_a_other) / (line_b_other - line_a_other);
    let intersection = line_a_coordinate + t * (line_b_coordinate - line_a_coordinate);
    if point_coordinate_value > intersection {
        (1, 0)
    } else if point_coordinate_value < intersection {
        (0, 1)
    } else {
        (-1, -1)
    }
}
