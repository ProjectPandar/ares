//! `TreeSupport.cpp:3172-3250::smooth_nodes`.

use super::nodes::{NodeArena, NodeId, NodeType};
use crate::{
    FloatOrPercent, ObjectOptions,
    geometry::{CoordinateScale, Point},
};

pub(super) fn smooth_nodes(
    arena: &mut NodeArena,
    layers: &[Vec<NodeId>],
    object: &ObjectOptions,
    scale: CoordinateScale,
) {
    for &id in layers.iter().flatten() {
        arena.nodes[id].is_processed = false;
    }
    let width = match object.support_line_width {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => value.0,
    };
    let max_move = scale.scaled_delta(width / 2.0) as f32;
    for &id in layers.iter().flatten() {
        if arena.nodes[id].is_processed {
            continue;
        }
        smooth_branch(arena, id, max_move);
    }
}

fn smooth_branch(arena: &mut NodeArena, start: NodeId, max_move: f32) {
    let mut points = Vec::new();
    let mut radii = Vec::new();
    let mut branch = Vec::new();
    let mut total_height = 0.0_f32;
    if let Some(child) = arena.nodes[start].child
        && arena.nodes[child].node_type != NodeType::Polygon
    {
        let node = &arena.nodes[child];
        points.push(node.position);
        radii.push(node.radius);
        branch.push(child);
        total_height = (f64::from(total_height) + node.height) as f32;
    }
    let mut current = start;
    loop {
        let node = &arena.nodes[current];
        points.push(node.position);
        radii.push(node.radius);
        branch.push(current);
        total_height = (f64::from(total_height) + node.height) as f32;
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
        if node.parents.len() > 1
            || node.movement.x() as f32 > max_move
            || node.movement.y() as f32 > max_move
            || (total_height > 100.0 && node.distance_mm_to_top < 30.0)
        {
            node.need_extra_wall = true;
        }
    }
    for index in 1..branch.len() - 1 {
        if arena.nodes[branch[index - 1]].need_extra_wall
            && arena.nodes[branch[index + 1]].need_extra_wall
        {
            arena.nodes[branch[index]].need_extra_wall = true;
        }
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
