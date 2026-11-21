//! `TreeSupport.hpp::SupportNode` and `TreeSupportData::create_node`.

use std::collections::VecDeque;

use crate::geometry::{ExPolygon, Point, Polygon};

pub(super) type NodeId = usize;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum NodeType {
    Circle,
    Polygon,
}

pub(super) struct NodeSeed {
    pub(super) position: Point,
    pub(super) distance_to_top: i32,
    pub(super) roof_layers_below: i32,
    pub(super) to_buildplate: bool,
    pub(super) parent: Option<NodeId>,
    pub(super) print_z: f64,
    pub(super) height: f64,
    pub(super) distance_mm_to_top: f64,
    pub(super) radius: f64,
}

pub(super) struct SupportNode {
    pub(super) position: Point,
    pub(super) movement: Point,
    pub(super) distance_to_top: i32,
    pub(super) distance_mm_to_top: f64,
    pub(super) roof_layers_below: i32,
    pub(super) to_buildplate: bool,
    pub(super) parent: Option<NodeId>,
    pub(super) parents: Vec<NodeId>,
    pub(super) child: Option<NodeId>,
    pub(super) merged_neighbors: VecDeque<NodeId>,
    pub(super) print_z: f64,
    pub(super) height: f64,
    pub(super) radius: f64,
    pub(super) max_move_distance: f64,
    pub(super) node_type: NodeType,
    pub(super) is_corner: bool,
    pub(super) is_processed: bool,
    pub(super) is_sharp_tail: bool,
    pub(super) valid: bool,
    pub(super) overhang: ExPolygon,
    pub(super) skin_direction: Point,
}

pub(super) struct NodeArena {
    pub(super) nodes: Vec<SupportNode>,
    diameter_angle_scale: f64,
}

impl NodeArena {
    pub(super) fn new(diameter_angle_scale: f64) -> Self {
        Self {
            nodes: Vec::new(),
            diameter_angle_scale,
        }
    }

    pub(super) fn create_node(&mut self, seed: NodeSeed) -> NodeId {
        let id = self.nodes.len();
        let mut node = SupportNode {
            position: seed.position,
            movement: Point::new(0, 0),
            distance_to_top: seed.distance_to_top,
            distance_mm_to_top: seed.distance_mm_to_top,
            roof_layers_below: seed.roof_layers_below,
            to_buildplate: seed.to_buildplate,
            parent: seed.parent,
            parents: Vec::new(),
            child: None,
            merged_neighbors: VecDeque::new(),
            print_z: seed.print_z,
            height: seed.height,
            radius: seed.radius,
            max_move_distance: 0.0,
            node_type: NodeType::Circle,
            is_corner: false,
            is_processed: false,
            is_sharp_tail: false,
            valid: true,
            overhang: ExPolygon::new(Polygon::new(Vec::new()), Vec::new()),
            skin_direction: Point::new(0, 0),
        };
        if let Some(parent_id) = seed.parent {
            let parent = &self.nodes[parent_id];
            node.parents.push(parent_id);
            node.node_type = parent.node_type;
            node.overhang = parent.overhang.clone();
            if node.distance_mm_to_top == 0.0 {
                node.distance_mm_to_top = parent.distance_mm_to_top + parent.height;
            }
            if node.radius == 0.0 && parent.radius > 0.0 {
                node.radius = parent.radius
                    + (node.distance_mm_to_top - parent.distance_mm_to_top)
                        * self.diameter_angle_scale;
            }
            node.parents.extend(parent.merged_neighbors.iter().copied());
            node.is_sharp_tail = parent.is_sharp_tail;
            node.skin_direction = parent.skin_direction;
            node.movement = Point::new(
                seed.position.x() - parent.position.x(),
                seed.position.y() - parent.position.y(),
            );
            for &parent_id in &node.parents {
                self.nodes[parent_id].child = Some(id);
            }
        }
        self.nodes.push(node);
        id
    }
}
