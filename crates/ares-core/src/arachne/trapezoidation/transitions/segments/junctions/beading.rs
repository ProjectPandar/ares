use std::{cell::RefCell, cmp::Ordering, collections::BinaryHeap, rc::Rc};

use crate::arachne::{
    skeletal::{BeadingPropagation, EdgeId, NodeId},
    trapezoidation::SkeletalTrapezoidation,
};

impl SkeletalTrapezoidation<'_> {
    pub(in crate::arachne::trapezoidation::transitions) fn get_or_create_beading(
        &mut self,
        node: NodeId,
    ) -> Rc<RefCell<BeadingPropagation>> {
        if let Some(storage) = self.graph.node(node).data.beading() {
            return storage;
        }
        let mut bead_count = self.graph.node(node).data.bead_count;
        if bead_count == -1 {
            let nearby_distance = self.config.coordinate_scale.checked_scale(0.1).unwrap();
            if let Some(storage) = self.nearest_beading(node, nearby_distance) {
                return storage;
            }
            let mut distance = i64::MAX;
            for edge in self.outgoing_edges(node) {
                let half_edge = self.graph.edge(edge);
                let to = half_edge.to.unwrap();
                let candidate = self.graph.node(to).data.distance_to_boundary
                    + super::super::super::point_distance(
                        self.graph.node(node).point,
                        self.graph.node(to).point,
                    );
                distance = distance.min(candidate);
            }
            assert_ne!(distance, i64::MAX);
            bead_count = self.beading_strategy.optimal_bead_count(distance * 2);
            self.graph.node_mut(node).data.bead_count = bead_count;
        }
        let radius = self.graph.node(node).data.distance_to_boundary;
        let storage = Rc::new(RefCell::new(BeadingPropagation::new(
            self.beading_strategy.compute(radius * 2, bead_count),
        )));
        self.graph.node_mut(node).data.set_beading(&storage);
        self.beading_storage.push(Rc::clone(&storage));
        storage
    }

    fn nearest_beading(
        &self,
        node: NodeId,
        maximum_distance: i64,
    ) -> Option<Rc<RefCell<BeadingPropagation>>> {
        let mut edges = BinaryHeap::new();
        for edge in self.outgoing_edges(node) {
            edges.push(DistanceEdge {
                edge,
                distance: self.edge_length(edge),
            });
        }
        for _ in 0..1_000 {
            let here = edges.pop()?;
            if here.distance > maximum_distance {
                return None;
            }
            let to = self.graph.edge(here.edge).to.unwrap();
            if let Some(storage) = self.graph.node(to).data.beading() {
                return Some(storage);
            }
            let incoming = self.graph.edge(here.edge).twin.unwrap();
            for further in self
                .outgoing_edges(to)
                .into_iter()
                .filter(|&edge| edge != incoming)
            {
                edges.push(DistanceEdge {
                    edge: further,
                    distance: here.distance + self.edge_length(further),
                });
            }
        }
        None
    }

    fn outgoing_edges(&self, node: NodeId) -> Vec<EdgeId> {
        let Some(start) = self.graph.node(node).incident_edge else {
            return Vec::new();
        };
        let mut output = Vec::new();
        let mut edge = start;
        loop {
            output.push(edge);
            let twin = self.graph.edge(edge).twin.unwrap();
            let Some(next) = self.graph.edge(twin).next else {
                break;
            };
            edge = next;
            if edge == start {
                break;
            }
        }
        output
    }

    fn edge_length(&self, edge: EdgeId) -> i64 {
        let half_edge = self.graph.edge(edge);
        super::super::super::point_distance(
            self.graph.node(half_edge.from.unwrap()).point,
            self.graph.node(half_edge.to.unwrap()).point,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DistanceEdge {
    edge: EdgeId,
    distance: i64,
}

impl Ord for DistanceEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance
            .cmp(&self.distance)
            .then_with(|| other.edge.0.cmp(&self.edge.0))
    }
}

impl PartialOrd for DistanceEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
