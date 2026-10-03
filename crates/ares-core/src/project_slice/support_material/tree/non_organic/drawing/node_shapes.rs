//! `TreeSupport.cpp:1978-2008,2110-2141` branch shape and first-layer brim.

use super::super::{contacts::ContactNodes, nodes::SupportNode};
use crate::{
    ObjectOptions,
    geometry::{ClipperError, CoordinateScale, JoinType, Point, Polygon, offset_paths},
};

pub(super) struct NodeShapes<'a> {
    object: &'a ObjectOptions,
    circle: Polygon,
    branch_radius: f64,
    scaled_radius: f64,
    pub(super) square: bool,
    scale: CoordinateScale,
}

impl<'a> NodeShapes<'a> {
    pub(super) fn new(
        contacts: &ContactNodes,
        object: &'a ObjectOptions,
        scale: CoordinateScale,
    ) -> Self {
        let branch_radius = object.tree_support_branch_diameter.0 / 2.0;
        let scaled_radius = scale.scaled_delta(branch_radius);
        let square = contacts.average_per_layer > 200;
        let resolution = if square { 4 } else { 100 };
        let mut points = Vec::with_capacity(resolution);
        for index in 0..resolution {
            let angle = index as f64 / resolution as f64 * std::f64::consts::TAU;
            let angle = if square {
                angle + std::f64::consts::FRAC_PI_4 + f64::from(contacts.angle)
            } else {
                angle
            };
            points.push(Point::new(
                (angle.cos() * scaled_radius).round() as i64,
                (angle.sin() * scaled_radius).round() as i64,
            ));
        }
        Self {
            object,
            circle: Polygon::new(points),
            branch_radius,
            scaled_radius,
            square,
            scale,
        }
    }

    pub(super) fn for_node(
        &self,
        node: &SupportNode,
        object_layer: usize,
        raft_layers: usize,
    ) -> Result<Polygon, ClipperError> {
        let ratio = node.radius / self.branch_radius;
        let move_x = node.movement.x() as f64 / (ratio * self.scaled_radius);
        let move_y = node.movement.y() as f64 / (ratio * self.scaled_radius);
        let mut points = self.circle.points().to_vec();
        if !self.square && move_x.abs() > 0.001 && move_y.abs() > 0.001 {
            let inverse = 0.5 / (0.01 + (move_x * move_x + move_y * move_y).sqrt());
            let matrix = [
                ratio * (1.0 + move_x * move_x * inverse),
                ratio * (0.0 + move_x * move_y * inverse),
                ratio * (0.0 + move_x * move_y * inverse),
                ratio * (1.0 + move_y * move_y * inverse),
            ];
            for point in &mut points {
                let x = point.x() as f64;
                let y = point.y() as f64;
                *point = Point::new(
                    node.position.x() + (matrix[0] * x + matrix[1] * y).round() as i64,
                    node.position.y() + (matrix[2] * x + matrix[3] * y).round() as i64,
                );
            }
        } else {
            for point in &mut points {
                *point = Point::new(
                    (point.x() as f64 * ratio).round() as i64 + node.position.x(),
                    (point.y() as f64 * ratio).round() as i64 + node.position.y(),
                );
            }
        }
        let mut circle = Polygon::new(points);
        if object_layer == 0 && raft_layers == 0 {
            let brim = if self.object.tree_support_auto_brim.0 {
                ((node.radius + node.distance_mm_to_top / (ratio * self.branch_radius) * 0.5)
                    .min(12.0)
                    - node.radius)
                    .max(2.0)
            } else {
                f64::from(self.object.tree_support_brim_width.0 as f32)
            };
            let expanded = offset_paths(
                std::slice::from_ref(&circle),
                self.scale.scaled_delta(brim) as f32,
                JoinType::Miter,
                3.0,
            )?;
            if let Some(first) = expanded.into_iter().next() {
                circle = first;
            }
        }
        Ok(circle)
    }
}
