use std::collections::VecDeque;

use crate::geometry::Point;

use super::payload::{SkeletalEdge, SkeletalJoint};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NodeId(pub(crate) usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct EdgeId(pub(crate) usize);

#[derive(Debug)]
pub(crate) struct HalfEdge {
    pub(crate) data: SkeletalEdge,
    pub(crate) twin: Option<EdgeId>,
    pub(crate) next: Option<EdgeId>,
    pub(crate) prev: Option<EdgeId>,
    pub(crate) from: Option<NodeId>,
    pub(crate) to: Option<NodeId>,
}

impl HalfEdge {
    pub(crate) fn new(data: SkeletalEdge) -> Self {
        Self {
            data,
            twin: None,
            next: None,
            prev: None,
            from: None,
            to: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct HalfEdgeNode {
    pub(crate) data: SkeletalJoint,
    pub(crate) point: Point,
    pub(crate) incident_edge: Option<EdgeId>,
}

impl HalfEdgeNode {
    pub(crate) const fn new(data: SkeletalJoint, point: Point) -> Self {
        Self {
            data,
            point,
            incident_edge: None,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SkeletalGraph {
    pub(super) edges: Vec<Option<HalfEdge>>,
    pub(super) nodes: Vec<Option<HalfEdgeNode>>,
    edge_order: VecDeque<EdgeId>,
    node_order: VecDeque<NodeId>,
}

impl SkeletalGraph {
    pub(crate) fn add_node(&mut self, data: SkeletalJoint, point: Point) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Some(HalfEdgeNode::new(data, point)));
        self.node_order.push_back(id);
        id
    }

    pub(super) fn add_node_front(&mut self, data: SkeletalJoint, point: Point) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Some(HalfEdgeNode::new(data, point)));
        self.node_order.push_front(id);
        id
    }

    pub(crate) fn add_edge(&mut self, data: SkeletalEdge) -> EdgeId {
        let id = EdgeId(self.edges.len());
        self.edges.push(Some(HalfEdge::new(data)));
        self.edge_order.push_back(id);
        id
    }

    pub(super) fn add_edge_front(&mut self, data: SkeletalEdge) -> EdgeId {
        let id = EdgeId(self.edges.len());
        self.edges.push(Some(HalfEdge::new(data)));
        self.edge_order.push_front(id);
        id
    }

    pub(crate) fn node(&self, id: NodeId) -> &HalfEdgeNode {
        self.nodes[id.0].as_ref().expect("node id must be active")
    }

    pub(crate) fn node_mut(&mut self, id: NodeId) -> &mut HalfEdgeNode {
        self.nodes[id.0].as_mut().expect("node id must be active")
    }

    pub(crate) fn edge(&self, id: EdgeId) -> &HalfEdge {
        self.edges[id.0].as_ref().expect("edge id must be active")
    }

    pub(crate) fn edge_mut(&mut self, id: EdgeId) -> &mut HalfEdge {
        self.edges[id.0].as_mut().expect("edge id must be active")
    }

    pub(crate) fn contains_node(&self, id: NodeId) -> bool {
        self.nodes.get(id.0).is_some_and(Option::is_some)
    }

    pub(crate) fn contains_edge(&self, id: EdgeId) -> bool {
        self.edges.get(id.0).is_some_and(Option::is_some)
    }

    pub(crate) fn active_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_order
            .iter()
            .filter(|id| self.contains_node(**id))
            .copied()
    }

    pub(crate) fn active_edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.edge_order
            .iter()
            .filter(|id| self.contains_edge(**id))
            .copied()
    }

    pub(crate) fn connect_twins(&mut self, left: EdgeId, right: EdgeId) {
        self.edge_mut(left).twin = Some(right);
        self.edge_mut(right).twin = Some(left);
    }

    pub(crate) fn edge_can_go_up(&self, edge: EdgeId, strict: bool) -> bool {
        let half_edge = self.edge(edge);
        let from_distance = self
            .node(half_edge.from.expect("edge must have from node"))
            .data
            .distance_to_boundary;
        let to_distance = self
            .node(half_edge.to.expect("edge must have to node"))
            .data
            .distance_to_boundary;
        if to_distance > from_distance {
            return true;
        }
        if to_distance < from_distance || strict {
            return false;
        }

        let Some(mut outgoing) = half_edge.next else {
            return false;
        };
        while Some(outgoing) != half_edge.twin {
            if self.edge_can_go_up(outgoing, false) {
                return true;
            }
            let Some(twin) = self.edge(outgoing).twin else {
                return false;
            };
            let Some(next) = self.edge(twin).next else {
                return true;
            };
            outgoing = next;
        }
        false
    }

    pub(crate) fn edge_is_upward(&self, edge: EdgeId) -> bool {
        let half_edge = self.edge(edge);
        let from = self.node(half_edge.from.expect("edge must have from node"));
        let to = self.node(half_edge.to.expect("edge must have to node"));
        if to.data.distance_to_boundary != from.data.distance_to_boundary {
            return to.data.distance_to_boundary > from.data.distance_to_boundary;
        }
        match (
            self.edge_dist_to_go_up(edge),
            self.edge_dist_to_go_up(half_edge.twin.expect("edge must have twin")),
        ) {
            (Some(forward), Some(backward)) => forward < backward,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => to.point < from.point,
        }
    }

    pub(crate) fn edge_dist_to_go_up(&self, edge: EdgeId) -> Option<i64> {
        let half_edge = self.edge(edge);
        let from = self.node(half_edge.from.expect("edge must have from node"));
        let to = self.node(half_edge.to.expect("edge must have to node"));
        if to.data.distance_to_boundary > from.data.distance_to_boundary {
            return Some(0);
        }
        if to.data.distance_to_boundary < from.data.distance_to_boundary {
            return None;
        }

        let mut nearest = None;
        let mut outgoing = half_edge.next?;
        while Some(outgoing) != half_edge.twin {
            if let Some(distance) = self.edge_dist_to_go_up(outgoing) {
                nearest = Some(nearest.map_or(distance, |current: i64| current.min(distance)));
            }
            let twin = self.edge(outgoing).twin?;
            let Some(next) = self.edge(twin).next else {
                return Some(0);
            };
            outgoing = next;
        }
        nearest.map(|distance| distance + point_distance(from.point, to.point))
    }

    pub(crate) fn next_unconnected(&self, edge: EdgeId) -> Option<EdgeId> {
        let mut result = edge;
        while let Some(next) = self.edge(result).next {
            result = next;
            if result == edge {
                return None;
            }
        }
        self.edge(result).twin
    }

    pub(crate) fn node_is_multi_intersection(&self, node: NodeId) -> bool {
        let Some(start) = self.node(node).incident_edge else {
            return false;
        };
        let mut outgoing = start;
        let mut central_paths = 0;
        loop {
            if self.edge(outgoing).data.is_central() {
                central_paths += 1;
            }
            let Some(twin) = self.edge(outgoing).twin else {
                return false;
            };
            let Some(next) = self.edge(twin).next else {
                return false;
            };
            outgoing = next;
            if outgoing == start {
                return central_paths > 2;
            }
        }
    }

    pub(crate) fn node_is_central(&self, node: NodeId) -> bool {
        let Some(start) = self.node(node).incident_edge else {
            return false;
        };
        let mut edge = start;
        loop {
            if self.edge(edge).data.is_central() {
                return true;
            }
            let Some(twin) = self.edge(edge).twin else {
                return false;
            };
            let Some(next) = self.edge(twin).next else {
                return false;
            };
            edge = next;
            if edge == start {
                return false;
            }
        }
    }

    pub(crate) fn node_is_local_maximum(&self, node: NodeId, strict: bool) -> bool {
        let joint = self.node(node);
        if joint.data.distance_to_boundary == 0 {
            return false;
        }
        let Some(start) = joint.incident_edge else {
            return false;
        };
        let mut edge = start;
        loop {
            if self.edge_can_go_up(edge, strict) {
                return false;
            }
            let Some(twin) = self.edge(edge).twin else {
                return false;
            };
            let Some(next) = self.edge(twin).next else {
                return false;
            };
            edge = next;
            if edge == start {
                return true;
            }
        }
    }
}

pub(super) fn point_distance(left: Point, right: Point) -> i64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    (dx * dx + dy * dy).sqrt() as i64
}
