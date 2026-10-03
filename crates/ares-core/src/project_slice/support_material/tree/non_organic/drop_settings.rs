//! `TreeSupport` constructor, active radius calculation and drop move cache.

use super::nodes::{NodeArena, NodeId};
use crate::{ObjectOptions, geometry::CoordinateScale};

pub(super) struct DropSettings {
    pub(super) diameter_angle_scale: f64,
    pub(super) tan_angle: f64,
    pub(super) max_move_distance: f64,
    pub(super) radius_resolution: f64,
    pub(super) scale: CoordinateScale,
    base_radius: f64,
    has_top_interface: bool,
    support_width: f64,
}

impl DropSettings {
    pub(super) fn new(
        object: &ObjectOptions,
        support_width: f64,
        radius_resolution: f64,
        scale: CoordinateScale,
    ) -> Self {
        let angle = object.tree_support_branch_angle.0 * std::f64::consts::PI / 180.0;
        let tan_angle = angle.tan();
        Self {
            diameter_angle_scale: (object.tree_support_branch_diameter_angle.0
                * std::f64::consts::PI
                / 180.0)
                .clamp(0.0, std::f64::consts::FRAC_PI_2 - 0.0001),
            tan_angle,
            max_move_distance: if angle < std::f64::consts::FRAC_PI_2 {
                tan_angle
                    * object.layer_height.0
                    * f64::from(object.tree_support_wall_count.0.max(1))
            } else {
                f64::MAX
            },
            radius_resolution,
            scale,
            base_radius: (object.tree_support_branch_diameter.0 / 2.0).max(0.4),
            has_top_interface: object.support_interface_top_layers.0 > 0,
            support_width,
        }
    }

    pub(super) fn radius(&self, distance_mm_to_top: f64) -> f64 {
        let radius = if distance_mm_to_top > self.base_radius {
            self.base_radius + (distance_mm_to_top - self.base_radius) * self.diameter_angle_scale
        } else {
            distance_mm_to_top
        }
        .clamp(0.4, 10.0);
        if self.has_top_interface {
            radius.max(self.base_radius)
        } else {
            radius
        }
    }

    pub(super) fn node_radius(&self, arena: &mut NodeArena, id: NodeId) -> f64 {
        let node = &mut arena.nodes[id];
        if node.radius == 0.0 {
            node.radius = self.radius(node.distance_mm_to_top);
        }
        node.radius
    }

    pub(super) fn move_distance(&self, arena: &mut NodeArena, id: NodeId) -> f64 {
        if arena.nodes[id].max_move_distance == 0.0 {
            self.node_radius(arena, id);
            let node = &mut arena.nodes[id];
            node.max_move_distance = (self.tan_angle * node.height).min(self.support_width);
        }
        arena.nodes[id].max_move_distance
    }
}
