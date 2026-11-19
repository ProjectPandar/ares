//! `TreeSupport.cpp:3172-3250::smooth_nodes`.

use super::nodes::{NodeArena, NodeId, NodeType};
use crate::geometry::Point;

pub(super) fn smooth_nodes(arena: &mut NodeArena, layers: &[Vec<NodeId>]) {
    for &id in layers.iter().flatten() {
        arena.nodes[id].is_processed = false;
    }

    for &id in layers.iter().flatten() {
        if arena.nodes[id].is_processed {
            continue;
        }
        smooth_branch(arena, id);
    }
}

fn smooth_branch(arena: &mut NodeArena, start: NodeId) {
    let mut points = Vec::new();
    let mut radii = Vec::new();
    let mut branch = Vec::new();

    if let Some(child) = arena.nodes[start].child
        && arena.nodes[child].node_type != NodeType::Polygon
    {
        let node = &arena.nodes[child];
        points.push(node.position);
        radii.push(node.radius);
        branch.push(child);
    }
    let mut current = start;
    loop {
        let node = &arena.nodes[current];
        points.push(node.position);
        radii.push(node.radius);
        branch.push(current);

        match node.parent {
            Some(parent) if !arena.nodes[parent].is_processed => current = parent,
            Some(_) | None => break,
        }
    }
    if points.len() < 3 {
        return;
    }
    let mut next_points = points.clone();
    let mut next_radii = radii.clone();
    for _ in 0..99 {
        average(&points, &radii, &mut next_points, &mut next_radii);
        std::mem::swap(&mut points, &mut next_points);
        std::mem::swap(&mut radii, &mut next_radii);
    }
    average(&points, &radii, &mut next_points, &mut next_radii);
    for index in 1..points.len() - 1 {
        let node = &mut arena.nodes[branch[index]];
        node.position = next_points[index];
        node.radius = next_radii[index];
        node.movement = Point::new(
            (points[index + 1].x() - points[index - 1].x()) / 2,
            (points[index + 1].y() - points[index - 1].y()) / 2,
        );
        node.is_processed = true;
    }
}

fn average(points: &[Point], radii: &[f64], next_points: &mut [Point], next_radii: &mut [f64]) {
    for index in 1..points.len() - 1 {
        next_points[index] = Point::new(
            (points[index - 1].x() + points[index].x() + points[index + 1].x()) / 3,
            (points[index - 1].y() + points[index].y() + points[index + 1].y()) / 3,
        );
        next_radii[index] = (radii[index - 1] + radii[index] + radii[index + 1]) / 3.0;
    }
}
