//! Neighbor convergence from `TreeSupport.cpp:2990-3035`.

use super::{
    drop_settings::DropSettings,
    grouping::Group,
    line_cut::LineCutCache,
    movement,
    nodes::{NodeArena, NodeId},
};
use crate::geometry::{ClipperError, Point};

pub(super) struct NeighborMovement<'a> {
    pub(super) arena: &'a mut NodeArena,
    pub(super) settings: &'a DropSettings,
    pub(super) cuts: &'a mut LineCutCache,
    pub(super) is_slim: bool,
    pub(super) is_strong: bool,
}

impl NeighborMovement<'_> {
    pub(super) fn for_node(
        &mut self,
        id: NodeId,
        group: &Group,
        index: usize,
    ) -> Result<Point, ClipperError> {
        let neighbors = group.tree.adjacent_nodes(index);
        let threshold = if self.is_slim { 0.0_f32 } else { 5.0_f32 };
        let first_distance = neighbors.first().map_or(0.0_f32, |&neighbor| {
            self.distance(id, group.nodes[neighbor]) as f32
        });
        if self.arena.nodes[id].print_z <= f64::from(threshold) {
            return Ok(Point::new(0, 0));
        }
        let can_move = if neighbors.len() > 1 {
            true
        } else if neighbors.len() == 1 {
            let maximum = self.settings.move_distance(self.arena, id);
            f64::from(first_distance) >= maximum * maximum
        } else {
            false
        };
        if !can_move {
            return Ok(Point::new(0, 0));
        }
        let mut sum = Point::new(0, 0);
        for &neighbor in neighbors {
            let other = &self.arena.nodes[group.nodes[neighbor]];
            if !other.valid {
                continue;
            }
            let node = &self.arena.nodes[id];
            let direction = Point::new(
                other.position.x() - node.position.x(),
                other.position.y() - node.position.y(),
            );
            let distance = movement::distance_squared(direction, self.settings.scale) as f32;
            let bottom_radius = self.settings.radius(node.distance_mm_to_top + node.print_z);
            let other_radius = self
                .settings
                .radius(other.distance_mm_to_top + other.print_z);
            let maximum = self.settings.tan_angle * (node.print_z - f64::from(threshold))
                + bottom_radius.max(other_radius);
            if f64::from(distance) > maximum * maximum
                || self.cuts.is_cut(node.position, other.position)?
            {
                continue;
            }
            let direction = if self.is_strong {
                direction
            } else {
                // Native divss precedes the rounded non-const Point product.
                let weight = f64::from(1.0_f32 / distance);
                Point::new(
                    (direction.x() as f64 * weight).round() as i64,
                    (direction.y() as f64 * weight).round() as i64,
                )
            };
            sum = Point::new(sum.x() + direction.x(), sum.y() + direction.y());
        }
        if self.is_strong {
            let maximum = self.settings.move_distance(self.arena, id);
            if movement::distance_squared(sum, self.settings.scale) > maximum * maximum {
                return Ok(movement::normal(
                    sum,
                    self.settings.scale.scaled_delta(maximum),
                    self.settings.scale,
                ));
            }
        }
        Ok(sum)
    }

    fn distance(&self, id: NodeId, neighbor: NodeId) -> f64 {
        let first = self.arena.nodes[id].position;
        let second = self.arena.nodes[neighbor].position;
        movement::distance_squared(
            Point::new(second.x() - first.x(), second.y() - first.y()),
            self.settings.scale,
        )
    }
}
