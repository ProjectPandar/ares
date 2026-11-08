//! Active second-pass descent from `TreeSupport.cpp:2937-3113`.

use std::collections::VecDeque;

use super::{
    data::TreeSupportData,
    drop_settings::DropSettings,
    grouping::Group,
    heights::LayerHeightData,
    line_cut::LineCutCache,
    movement,
    neighbor_movement::NeighborMovement,
    nodes::{NodeArena, NodeId, NodeSeed, NodeType},
};
use crate::geometry::{
    BoundingBox, ClipperError, Point, Polygon, clip_clipper_expolygons_with_subject_bbox,
    difference_ex_polygons,
};

pub(super) struct DropPass<'a> {
    pub(super) group_index: usize,
    pub(super) arena: &'a mut NodeArena,
    pub(super) data: &'a mut TreeSupportData,
    pub(super) settings: &'a DropSettings,
    pub(super) current_height: &'a LayerHeightData,
    pub(super) next_height: &'a LayerHeightData,
    pub(super) next_nodes: &'a mut Vec<NodeId>,
    pub(super) unsupported: &'a mut VecDeque<NodeId>,
    pub(super) cuts: &'a mut LineCutCache,
    pub(super) buildplate_only: bool,
    pub(super) is_slim: bool,
    pub(super) is_strong: bool,
}

impl DropPass<'_> {
    pub(super) fn run(&mut self, group: &Group) -> Result<(), ClipperError> {
        for (index, &id) in group.nodes.iter().enumerate() {
            if !self.arena.nodes[id].valid {
                continue;
            }
            if self.arena.nodes[id].node_type == NodeType::Polygon {
                self.drop_polygon(id)?;
                continue;
            }
            if self.reject_buried(id)? {
                continue;
            }
            let center = NeighborMovement {
                arena: self.arena,
                settings: self.settings,
                cuts: self.cuts,
                is_slim: self.is_slim,
                is_strong: self.is_strong,
            }
            .for_node(id, group, index)?;
            let (outer, distance) = self.outward_direction(id)?;
            let node = &self.arena.nodes[id];
            let direction = if node.is_sharp_tail && node.distance_mm_to_top < 3.0 {
                node.skin_direction
            } else if distance > 0.0 {
                outer
            } else {
                center
            };
            let maximum = self.settings.move_distance(self.arena, id);
            let movement = movement::normal(
                direction,
                self.settings.scale.scaled_delta(maximum),
                self.settings.scale,
            );
            let position = self.arena.nodes[id].position;
            self.drop_circle(
                id,
                Point::new(position.x() + movement.x(), position.y() + movement.y()),
            )?;
        }
        Ok(())
    }

    fn drop_polygon(&mut self, id: NodeId) -> Result<(), ClipperError> {
        let polygon = &self.arena.nodes[id].overhang;
        let bounds = BoundingBox::from_expolygon(polygon)
            .expect("polygon support node has a contour")
            .inflated(self.settings.scale.scaled_delta(0.0001) as i64);
        let collision = self.data.collision(0.0, self.next_height.object_layer)?;
        let clip = clip_clipper_expolygons_with_subject_bbox(collision, bounds);
        let polygons = difference_ex_polygons(std::slice::from_ref(polygon), &clip)?;
        for polygon in polygons {
            let next =
                self.arena
                    .create_node(self.next_seed(id, centroid(polygon.contour()), true));
            self.arena.nodes[next].max_move_distance = 0.0;
            self.arena.nodes[next].overhang = polygon;
            self.next_nodes.push(next);
        }
        Ok(())
    }

    fn reject_buried(&mut self, id: NodeId) -> Result<bool, ClipperError> {
        if self.group_index == 0 {
            return Ok(false);
        }
        let position = self.arena.nodes[id].position;
        if !movement::is_inside_any(
            self.data.collision(0.0, self.current_height.object_layer)?,
            position,
        ) {
            return Ok(false);
        }
        let radius = self.settings.node_radius(self.arena, id);
        let projected = movement::projection_onto(
            self.data.collision(0.0, self.current_height.object_layer)?,
            position,
        );
        let direction = Point::new(position.x() - projected.x(), position.y() - projected.y());
        if movement::distance_squared(direction, self.settings.scale) >= radius * radius {
            if self.buildplate_only {
                self.unsupported.push_front(id);
            } else {
                self.arena.nodes[id].valid = false;
            }
            return Ok(true);
        }
        if let Some(parent) = self.arena.nodes[id].parent
            && self
                .cuts
                .intersects(position, self.arena.nodes[parent].position)?
        {
            self.arena.nodes[id].valid = false;
            return Ok(true);
        }
        Ok(false)
    }

    fn outward_direction(&mut self, id: NodeId) -> Result<(Point, f64), ClipperError> {
        let node = &self.arena.nodes[id];
        let position = node.position;
        let radius = self
            .settings
            .radius(node.distance_mm_to_top + self.next_height.height);
        let avoidance = self.data.avoidance(radius, self.next_height.object_layer)?;
        let projected = movement::projection_onto(avoidance, position);
        let direction = Point::new(projected.x() - position.x(), projected.y() - position.y());
        let distance = movement::distance_squared(direction, self.settings.scale);
        let maximum = self.settings.max_move_distance;
        // Preserve the reference's unsigned 64-bit size_t SQ before conversion.
        let object_layer = self.current_height.object_layer as u64;
        let object_layer_squared = object_layer.wrapping_mul(object_layer) as f64;
        if self.cuts.is_cut(position, projected)?
            || distance > maximum * maximum * object_layer_squared
            || !movement::is_inside_any(avoidance, position)
        {
            let mut candidate = position;
            let between_samples = maximum + self.settings.radius_resolution + 0.0001;
            if movement::move_out(
                self.data.collision(radius, self.next_height.object_layer)?,
                &mut candidate,
                between_samples,
                between_samples,
                self.settings.scale,
            )? {
                let direction =
                    Point::new(candidate.x() - position.x(), candidate.y() - position.y());
                return Ok((
                    direction,
                    movement::distance_squared(direction, self.settings.scale),
                ));
            }
            return Ok((Point::new(0, 0), 0.0));
        }
        Ok((direction, distance))
    }

    fn drop_circle(&mut self, id: NodeId, position: Point) -> Result<(), ClipperError> {
        let collision = self.data.collision(0.0, self.next_height.object_layer)?;
        let projected = movement::projection_onto(collision, position);
        let to_buildplate =
            !movement::is_inside_any(&self.data.outlines[self.next_height.object_layer], position);
        let old_position = self.arena.nodes[id].position;
        let direction = Point::new(
            projected.x() - old_position.x(),
            projected.y() - old_position.y(),
        );
        let x = direction.x() as f64;
        let y = direction.y() as f64;
        let distance = self.settings.scale.unscale(1) * (x * x + y * y).sqrt();
        let next = self
            .arena
            .create_node(self.next_seed(id, position, to_buildplate));
        self.arena.nodes[next].radius = self.arena.nodes[id]
            .radius
            .max(self.arena.nodes[next].radius.min(distance));
        self.settings.move_distance(self.arena, next);
        self.next_nodes.push(next);
        Ok(())
    }

    fn next_seed(&self, id: NodeId, position: Point, to_buildplate: bool) -> NodeSeed {
        let node = &self.arena.nodes[id];
        NodeSeed {
            position,
            distance_to_top: node.distance_to_top + 1,
            roof_layers_below: node.roof_layers_below - i32::from(node.distance_to_top >= 0),
            to_buildplate,
            parent: Some(id),
            print_z: self.next_height.print_z,
            height: self.next_height.height,
            distance_mm_to_top: 0.0,
            radius: 0.0,
        }
    }
}

// Polygon.cpp:141-156, with the rounded Point(Vec2d) constructor.
fn centroid(polygon: &Polygon) -> Point {
    let points = polygon.points();
    let mut previous = points[points.len() - 1];
    let mut area = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    for &point in points {
        let cross = previous.x() as f64 * point.y() as f64 - previous.y() as f64 * point.x() as f64;
        area += cross;
        x += (previous.x() as f64 + point.x() as f64) * cross;
        y += (previous.y() as f64 + point.y() as f64) * cross;
        previous = point;
    }
    Point::new(
        (x / (3.0 * area)).round() as i64,
        (y / (3.0 * area)).round() as i64,
    )
}
