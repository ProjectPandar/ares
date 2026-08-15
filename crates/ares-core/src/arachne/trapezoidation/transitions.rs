use std::{cell::RefCell, rc::Rc};

use crate::geometry::Point;

use super::SkeletalTrapezoidation;
use crate::arachne::skeletal::TransitionMiddle;

impl SkeletalTrapezoidation<'_> {
    pub(super) fn generate_transition_mids(&mut self) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            if !self.graph.edge(edge).data.is_central() {
                continue;
            }
            let from = self.graph.edge(edge).from.unwrap();
            let to = self.graph.edge(edge).to.unwrap();
            let start_radius = self.graph.node(from).data.distance_to_boundary;
            let end_radius = self.graph.node(to).data.distance_to_boundary;
            if start_radius >= end_radius {
                continue;
            }
            let start_beads = self.graph.node(from).data.bead_count;
            let end_beads = self.graph.node(to).data.bead_count;
            if start_beads >= end_beads {
                continue;
            }
            let edge_length =
                point_distance(self.graph.node(from).point, self.graph.node(to).point);
            let transitions = Rc::new(RefCell::new(Vec::new()));
            for lower_bead_count in start_beads..end_beads {
                let transition_radius = self
                    .beading_strategy
                    .transition_thickness(lower_bead_count)
                    .clamp(start_radius, end_radius);
                let position = i128::from(edge_length)
                    * i128::from(transition_radius - start_radius)
                    / i128::from(end_radius - start_radius);
                transitions.borrow_mut().push(TransitionMiddle::new(
                    position as i64,
                    lower_bead_count as i32,
                    transition_radius,
                ));
            }
            if !transitions.borrow().is_empty() {
                self.graph.edge_mut(edge).data.set_transitions(&transitions);
                self.transition_storage.push(transitions);
            }
        }
    }
}

fn point_distance(left: Point, right: Point) -> i64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    (dx * dx + dy * dy).sqrt() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arachne::{
        beading::factory::{BeadingStrategyFactoryConfig, make_strategy},
        skeletal::{SkeletalEdge, SkeletalGraph, SkeletalJoint},
    };
    use crate::geometry::CoordinateScale;

    #[test]
    fn task22o102_generates_ordered_mids_on_upward_central_edge() {
        let scale = CoordinateScale::Normal;
        let scaled = |value| scale.checked_scale(value).unwrap();
        let strategy = make_strategy(BeadingStrategyFactoryConfig {
            preferred_bead_width_outer: scaled(0.42),
            preferred_bead_width_inner: scaled(0.45),
            preferred_transition_length: scaled(0.4),
            transitioning_angle: 1.0,
            print_thin_walls: true,
            min_bead_width: scaled(0.2),
            min_feature_size: scaled(0.25),
            wall_split_middle_threshold: 0.5,
            wall_add_middle_threshold: 0.5,
            max_bead_count: 4,
            outer_wall_offset: 0,
            inward_distributed_center_wall_count: 1,
            minimum_variable_line_ratio: 0.8,
            coordinate_scale: scale,
        });
        let mut graph = SkeletalGraph::default();
        let from = graph.add_node(SkeletalJoint::default(), Point::new(0, 0));
        let to = graph.add_node(SkeletalJoint::default(), Point::new(scaled(2.0), 0));
        graph.node_mut(from).data.distance_to_boundary = scaled(0.3);
        graph.node_mut(to).data.distance_to_boundary = scaled(2.0);
        graph.node_mut(from).data.bead_count = 1;
        graph.node_mut(to).data.bead_count = 3;
        let edge = graph.add_edge(SkeletalEdge::default());
        let twin = graph.add_edge(SkeletalEdge::default());
        graph.edge_mut(edge).from = Some(from);
        graph.edge_mut(edge).to = Some(to);
        graph.edge_mut(twin).from = Some(to);
        graph.edge_mut(twin).to = Some(from);
        graph.connect_twins(edge, twin);
        graph.edge_mut(edge).data.set_is_central(true);
        graph.edge_mut(twin).data.set_is_central(true);
        let mut trapezoidation = SkeletalTrapezoidation {
            graph,
            beading_strategy: strategy.as_ref(),
            config: super::super::TrapezoidationConfig {
                transitioning_angle: 1.0,
                discretization_step_size: scaled(0.1),
                transition_filter_dist: scaled(0.4),
                allowed_filter_deviation: scaled(0.02),
                beading_propagation_transition_dist: scaled(0.4),
                coordinate_scale: scale,
            },
            vd_edge_to_he_edge: Default::default(),
            vd_node_to_he_node: Default::default(),
            transition_storage: Vec::new(),
        };

        trapezoidation.generate_transition_mids();

        let transitions = trapezoidation.graph.edge(edge).data.transitions().unwrap();
        let transitions = transitions.borrow();
        assert_eq!(transitions.len(), 2);
        assert!(transitions[0].pos < transitions[1].pos);
        assert_eq!(transitions[0].lower_bead_count, 1);
        assert_eq!(transitions[1].lower_bead_count, 2);
        assert!(transitions.iter().all(|transition| {
            transition.feature_radius >= scaled(0.3) && transition.feature_radius <= scaled(2.0)
        }));
    }
}
