use crate::arachne::skeletal::EdgeId;

use super::{SkeletalTrapezoidation, point_at_distance, point_distance};

impl SkeletalTrapezoidation<'_> {
    pub(super) fn generate_extra_ribs(&mut self) {
        let snap_distance = self.config.coordinate_scale.checked_scale(0.02).unwrap();
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            self.generate_extra_ribs_on_edge(edge, snap_distance);
        }
    }

    fn generate_extra_ribs_on_edge(&mut self, edge: EdgeId, snap_distance: i64) {
        if !self.graph.edge(edge).data.is_central() {
            return;
        }
        let from = self.graph.edge(edge).from.unwrap();
        let to = self.graph.edge(edge).to.unwrap();
        let start = self.graph.node(from).point;
        let finish = self.graph.node(to).point;
        let edge_length = point_distance(start, finish);
        let start_radius = self.graph.node(from).data.distance_to_boundary;
        let end_radius = self.graph.node(to).data.distance_to_boundary;
        if edge_length < self.config.discretization_step_size || start_radius >= end_radius {
            return;
        }

        let thicknesses = self
            .beading_strategy
            .nonlinear_thicknesses(self.graph.node(from).data.bead_count);
        let bead_count = self
            .graph
            .node(from)
            .data
            .bead_count
            .min(self.graph.node(to).data.bead_count);
        let mut last_edge = edge;
        for thickness in thicknesses {
            let radius = thickness / 2;
            if radius <= start_radius {
                continue;
            }
            if radius >= end_radius {
                break;
            }
            let position = (i128::from(edge_length) * i128::from(radius - start_radius)
                / i128::from(end_radius - start_radius)) as i64;
            assert!(position > 0 && position < edge_length);
            let close_node = if position < edge_length / 2 { from } else { to };
            if (position < snap_distance || position > edge_length - snap_distance)
                && self.graph.node(close_node).data.bead_count == bead_count
            {
                self.graph.node_mut(close_node).data.transition_ratio = 0.0;
                continue;
            }
            let middle = point_at_distance(start, finish, position);
            last_edge = self.graph.insert_node(last_edge, middle, bead_count);
        }
    }
}

#[cfg(test)]
mod tests;
