//! First-pass node merging in `TreeSupport.cpp:2844-2934`.

use super::{
    data::TreeSupportData,
    drop_settings::DropSettings,
    grouping::Group,
    heights::LayerHeightData,
    movement,
    nodes::{NodeArena, NodeId, NodeSeed, NodeType},
};
use crate::geometry::{ClipperError, Point};

pub(super) struct MergePass<'a> {
    pub(super) group_index: usize,
    pub(super) arena: &'a mut NodeArena,
    pub(super) data: &'a mut TreeSupportData,
    pub(super) settings: &'a DropSettings,
    pub(super) next_height: &'a LayerHeightData,
    pub(super) next_nodes: &'a mut Vec<NodeId>,
}

impl MergePass<'_> {
    pub(super) fn run(&mut self, group: &Group) -> Result<(), ClipperError> {
        for (index, &id) in group.nodes.iter().enumerate() {
            if !self.arena.nodes[id].valid {
                continue;
            }
            let neighbors = group.tree.adjacent_nodes(index);
            if self.arena.nodes[id].node_type == NodeType::Polygon {
                self.merge_polygon(id, group, neighbors);
            } else if neighbors.len() == 1
                && self.pair_is_close(id, group.nodes[neighbors[0]])
                && group.tree.adjacent_nodes(neighbors[0]).len() == 1
                && self.arena.nodes[group.nodes[neighbors[0]]].node_type != NodeType::Polygon
            {
                self.collapse_pair(id, group.nodes[neighbors[0]])?;
            } else if neighbors.len() > 1 {
                self.merge_neighbors(id, group, neighbors);
            }
        }
        Ok(())
    }

    fn pair_is_close(&mut self, id: NodeId, neighbor: NodeId) -> bool {
        let node = self.arena.nodes[id].position;
        let other = self.arena.nodes[neighbor].position;
        let distance = movement::distance_squared(
            Point::new(other.x() - node.x(), other.y() - node.y()),
            self.settings.scale,
        );
        let maximum = self.settings.move_distance(self.arena, id);
        distance < maximum * maximum
    }

    fn merge_polygon(&mut self, id: NodeId, group: &Group, neighbors: &[usize]) {
        for &neighbor in neighbors {
            let neighbor = group.nodes[neighbor];
            let other = &self.arena.nodes[neighbor];
            if !other.valid || other.node_type == NodeType::Polygon {
                continue;
            }
            let radius = self.settings.scale.scaled_delta(other.radius) as i64;
            let position = other.position;
            let points = [
                position,
                Point::new(position.x(), position.y() + radius),
                Point::new(position.x(), position.y() - radius),
                Point::new(position.x() - radius, position.y()),
                Point::new(position.x() + radius, position.y()),
            ];
            if !points
                .iter()
                .all(|&point| movement::is_inside(&self.arena.nodes[id].overhang, point))
            {
                continue;
            }
            self.arena.nodes[id].distance_to_top = self.arena.nodes[id]
                .distance_to_top
                .max(self.arena.nodes[neighbor].distance_to_top);
            self.arena.nodes[id].roof_layers_below = self.arena.nodes[id]
                .roof_layers_below
                .max(self.arena.nodes[neighbor].roof_layers_below);
            self.arena.nodes[id].distance_mm_to_top = self.arena.nodes[id]
                .distance_mm_to_top
                .max(self.arena.nodes[neighbor].distance_mm_to_top);
            self.merge_neighbor(id, neighbor);
        }
    }

    fn collapse_pair(&mut self, id: NodeId, neighbor: NodeId) -> Result<(), ClipperError> {
        let node = &self.arena.nodes[id];
        let other = self.arena.nodes[neighbor].position;
        let mut position = Point::new(
            (node.position.x() + other.x()) / 2,
            (node.position.y() + other.y()) / 2,
        );
        let radius = self
            .settings
            .radius(node.distance_mm_to_top + self.next_height.height);
        let avoidance = self.data.avoidance(radius, self.next_height.object_layer)?;
        if self.group_index == 0 {
            movement::move_out(
                avoidance,
                &mut position,
                self.settings.radius_resolution + 0.0001,
                self.settings.max_move_distance + self.settings.radius_resolution + 0.0001,
                self.settings.scale,
            )?;
        }
        let parent = match (
            self.arena.nodes[id].parent,
            self.arena.nodes[neighbor].parent,
        ) {
            (Some(_), Some(_)) => {
                if self.arena.nodes[id].distance_mm_to_top
                    >= self.arena.nodes[neighbor].distance_mm_to_top
                {
                    id
                } else {
                    neighbor
                }
            }
            (Some(_), None) => id,
            (None, _) => neighbor,
        };
        self.arena.nodes[parent]
            .merged_neighbors
            .push_front(if parent == id { neighbor } else { id });
        let to_buildplate = !movement::is_inside_any(
            self.data.collision(0.0, self.next_height.object_layer)?,
            position,
        );
        let node = &self.arena.nodes[parent];
        let next = self.arena.create_node(NodeSeed {
            position,
            distance_to_top: node.distance_to_top + 1,
            roof_layers_below: node.roof_layers_below - i32::from(node.distance_to_top >= 0),
            to_buildplate,
            parent: Some(parent),
            print_z: self.next_height.print_z,
            height: self.next_height.height,
            distance_mm_to_top: 0.0,
            radius: 0.0,
        });
        self.settings.move_distance(self.arena, next);
        self.next_nodes.push(next);
        self.arena.nodes[neighbor].valid = false;
        self.arena.nodes[id].valid = false;
        Ok(())
    }

    fn merge_neighbors(&mut self, id: NodeId, group: &Group, neighbors: &[usize]) {
        for &neighbor in neighbors {
            let neighbor = group.nodes[neighbor];
            if !self.pair_is_close(id, neighbor)
                || self.arena.nodes[neighbor].node_type == NodeType::Polygon
                || self.arena.nodes[id].distance_mm_to_top
                    < self.arena.nodes[neighbor].distance_mm_to_top
            {
                continue;
            }
            if self.arena.nodes[id].valid {
                self.merge_neighbor(id, neighbor);
            }
        }
    }

    fn merge_neighbor(&mut self, id: NodeId, neighbor: NodeId) {
        let (node, other) = if id < neighbor {
            let (lower, upper) = self.arena.nodes.split_at_mut(neighbor);
            (&mut lower[id], &mut upper[0])
        } else {
            let (lower, upper) = self.arena.nodes.split_at_mut(id);
            (&mut upper[0], &mut lower[neighbor])
        };
        node.merged_neighbors.push_front(neighbor);
        node.merged_neighbors
            .extend(other.merged_neighbors.iter().copied());
        other.valid = false;
    }
}
