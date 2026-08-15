use crate::arachne::skeletal::EdgeId;

use super::SkeletalTrapezoidation;

impl SkeletalTrapezoidation<'_> {
    #[expect(
        clippy::excessive_nesting,
        reason = "preserves the pinned central-edge classification branches"
    )]
    pub(super) fn update_is_central(&mut self) {
        let outer_filter = self.beading_strategy.transition_thickness(0) / 2;
        let cap = (self.beading_strategy.transitioning_angle() * 0.5).sin() as f32;
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            let twin = self.graph.edge(edge).twin.unwrap();
            let central = if self.graph.edge(twin).data.central_is_set() {
                self.graph.edge(twin).data.is_central()
            } else if self.graph.edge(edge).data.edge_type
                == crate::arachne::skeletal::EdgeType::ExtraVoronoi
            {
                false
            } else {
                let from = self.graph.edge(edge).from.unwrap();
                let to = self.graph.edge(edge).to.unwrap();
                let from_distance = self.graph.node(from).data.distance_to_boundary;
                let to_distance = self.graph.node(to).data.distance_to_boundary;
                if from_distance.max(to_distance) < outer_filter {
                    false
                } else {
                    let delta_radius = (to_distance - from_distance).abs();
                    let edge_length =
                        point_distance(self.graph.node(from).point, self.graph.node(to).point);
                    (delta_radius as f32) < edge_length as f32 * cap
                }
            };
            self.graph.edge_mut(edge).data.set_is_central(central);
        }
    }

    pub(super) fn filter_central(&mut self, max_length: i64) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            let to = self.graph.edge(edge).to.unwrap();
            let local_maximum = self.graph.node_is_local_maximum(to, false);
            let repeated_local_maximum = self.graph.node_is_local_maximum(to, false);
            if self.is_end_of_central(edge) && local_maximum && !repeated_local_maximum {
                let twin = self.graph.edge(edge).twin.unwrap();
                self.filter_central_from(twin, 0, max_length);
            }
        }
    }

    pub(super) fn filter_central_from(
        &mut self,
        starting_edge: EdgeId,
        traveled_distance: i64,
        max_length: i64,
    ) -> bool {
        let from = self.graph.edge(starting_edge).from.unwrap();
        let to = self.graph.edge(starting_edge).to.unwrap();
        let length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
        if traveled_distance + length > max_length {
            return false;
        }
        let twin = self.graph.edge(starting_edge).twin.unwrap();
        let mut outgoing = self.graph.edge(starting_edge).next;
        let mut dissolve = true;
        while let Some(edge) = outgoing {
            if edge == twin {
                break;
            }
            if self.graph.edge(edge).data.is_central() {
                dissolve &= self.filter_central_from(edge, traveled_distance + length, max_length);
            }
            let edge_twin = self.graph.edge(edge).twin.unwrap();
            outgoing = self.graph.edge(edge_twin).next;
        }
        dissolve &= !self.graph.node_is_local_maximum(to, false);
        if dissolve {
            self.graph
                .edge_mut(starting_edge)
                .data
                .set_is_central(false);
            self.graph.edge_mut(twin).data.set_is_central(false);
        }
        dissolve
    }

    pub(super) fn filter_outer_central(&mut self) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            if self.graph.edge(edge).prev.is_none() {
                let twin = self.graph.edge(edge).twin.unwrap();
                self.graph.edge_mut(edge).data.set_is_central(false);
                self.graph.edge_mut(twin).data.set_is_central(false);
            }
        }
    }

    #[expect(
        clippy::excessive_nesting,
        reason = "preserves the pinned local-maximum bead-count traversal"
    )]
    pub(super) fn update_bead_count(&mut self) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            if self.graph.edge(edge).data.is_central() {
                let to = self.graph.edge(edge).to.unwrap();
                let thickness = self.graph.node(to).data.distance_to_boundary * 2;
                self.graph.node_mut(to).data.bead_count =
                    self.beading_strategy.optimal_bead_count(thickness);
            }
        }
        let nodes = self.graph.active_nodes().collect::<Vec<_>>();
        for node in nodes {
            if !self.graph.node_is_local_maximum(node, false) {
                continue;
            }
            if self.graph.node(node).data.distance_to_boundary < 0 {
                let incident = self.graph.node(node).incident_edge.unwrap();
                let mut edge = incident;
                let mut distance = i64::MAX;
                loop {
                    let to = self.graph.edge(edge).to.unwrap();
                    distance = distance.min(
                        self.graph.node(to).data.distance_to_boundary
                            + point_distance(
                                self.graph.node(node).point,
                                self.graph.node(to).point,
                            ),
                    );
                    let twin = self.graph.edge(edge).twin.unwrap();
                    edge = self.graph.edge(twin).next.unwrap();
                    if edge == incident {
                        break;
                    }
                }
                self.graph.node_mut(node).data.distance_to_boundary = distance;
            }
            let thickness = self.graph.node(node).data.distance_to_boundary * 2;
            self.graph.node_mut(node).data.bead_count =
                self.beading_strategy.optimal_bead_count(thickness);
        }
    }

    pub(super) fn filter_noncentral_regions(&mut self) {
        let max_distance = self.config.coordinate_scale.checked_scale(0.4).unwrap();
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            if !self.is_end_of_central(edge) {
                continue;
            }
            let to = self.graph.edge(edge).to.unwrap();
            let bead_count = self.graph.node(to).data.bead_count;
            self.filter_noncentral_from(edge, bead_count, 0, max_distance);
        }
    }

    pub(super) fn filter_noncentral_from(
        &mut self,
        to_edge: EdgeId,
        bead_count: i64,
        traveled_distance: i64,
        max_distance: i64,
    ) -> bool {
        let to = self.graph.edge(to_edge).to.unwrap();
        let radius = self.graph.node(to).data.distance_to_boundary;
        let stop = self.graph.edge(to_edge).twin.unwrap();
        let mut next_edge = self.graph.edge(to_edge).next;
        while let Some(edge) = next_edge {
            if edge == stop {
                return false;
            }
            let from = self.graph.edge(edge).from.unwrap();
            let to = self.graph.edge(edge).to.unwrap();
            if self.graph.node(to).data.distance_to_boundary >= radius
                || nodes_are_close(
                    self.graph.node(from).point,
                    self.graph.node(to).point,
                    self.config.coordinate_scale.checked_scale(0.01).unwrap(),
                )
            {
                next_edge = Some(edge);
                break;
            }
            let twin = self.graph.edge(edge).twin.unwrap();
            next_edge = self.graph.edge(twin).next;
        }
        let Some(next_edge) = next_edge else {
            return false;
        };
        let from = self.graph.edge(next_edge).from.unwrap();
        let to = self.graph.edge(next_edge).to.unwrap();
        let length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
        let next_bead_count = self.graph.node(to).data.bead_count;
        let dissolve = if next_bead_count == bead_count {
            true
        } else if next_bead_count < 0 {
            self.filter_noncentral_from(
                next_edge,
                bead_count,
                traveled_distance + length,
                max_distance,
            )
        } else {
            traveled_distance + length < max_distance && (next_bead_count - bead_count).abs() == 1
        };
        if dissolve {
            let twin = self.graph.edge(next_edge).twin.unwrap();
            self.graph.edge_mut(next_edge).data.set_is_central(true);
            self.graph.edge_mut(twin).data.set_is_central(true);
            let thickness = self.graph.node(to).data.distance_to_boundary * 2;
            self.graph.node_mut(to).data.bead_count =
                self.beading_strategy.optimal_bead_count(thickness);
            self.graph.node_mut(to).data.transition_ratio = 0.0;
        }
        dissolve
    }

    fn is_end_of_central(&self, edge: EdgeId) -> bool {
        if !self.graph.edge(edge).data.is_central() {
            return false;
        }
        let twin = self.graph.edge(edge).twin.unwrap();
        let mut next = self.graph.edge(edge).next;
        while let Some(candidate) = next {
            if candidate == twin {
                break;
            }
            if self.graph.edge(candidate).data.is_central() {
                return false;
            }
            let candidate_twin = self.graph.edge(candidate).twin.unwrap();
            next = self.graph.edge(candidate_twin).next;
        }
        true
    }
}

fn point_distance(left: crate::geometry::Point, right: crate::geometry::Point) -> i64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    (dx * dx + dy * dy).sqrt() as i64
}

fn nodes_are_close(
    left: crate::geometry::Point,
    right: crate::geometry::Point,
    limit: i64,
) -> bool {
    let dx = (left.x() - right.x()) as i128;
    let dy = (left.y() - right.y()) as i128;
    dx * dx + dy * dy <= limit as i128 * limit as i128
}
