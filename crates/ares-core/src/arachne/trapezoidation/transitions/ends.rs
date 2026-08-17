use std::{cell::RefCell, rc::Rc};

use crate::arachne::skeletal::{EdgeId, TransitionEnd, TransitionMiddle};

use super::{SkeletalTrapezoidation, point_distance};

impl SkeletalTrapezoidation<'_> {
    pub(super) fn generate_all_transition_ends(&mut self) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            let Some(storage) = self.graph.edge(edge).data.transitions() else {
                continue;
            };
            let transitions = storage.borrow().clone();
            if transitions.is_empty() {
                continue;
            }
            let from = self.graph.edge(edge).from.unwrap();
            let to = self.graph.edge(edge).to.unwrap();
            assert!(
                self.graph.node(from).data.distance_to_boundary
                    <= self.graph.node(to).data.distance_to_boundary
            );
            for transition in transitions {
                self.generate_transition_ends(edge, transition);
            }
        }
    }

    fn generate_transition_ends(&mut self, edge: EdgeId, middle: TransitionMiddle) {
        let from = self.graph.edge(edge).from.unwrap();
        let to = self.graph.edge(edge).to.unwrap();
        let edge_length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
        let lower_bead_count = i64::from(middle.lower_bead_count);
        let transition_length = self.beading_strategy.transitioning_length(lower_bead_count);
        let anchor = f64::from(
            self.beading_strategy
                .transition_anchor_pos(lower_bead_count),
        );

        let lower_half_length = (anchor * transition_length as f64) as i64;
        let twin = self.graph.edge(edge).twin.unwrap();
        self.generate_local_transition_end(
            twin,
            edge_length - middle.pos + lower_half_length,
            middle.lower_bead_count,
            true,
        );

        let upper_half_length = ((1.0 - anchor) * transition_length as f64) as i64;
        self.generate_local_transition_end(
            edge,
            middle.pos + upper_half_length,
            middle.lower_bead_count,
            false,
        );
    }

    fn generate_local_transition_end(
        &mut self,
        edge: EdgeId,
        end_position: i64,
        lower_bead_count: i32,
        is_lower_end: bool,
    ) -> bool {
        let from = self.graph.edge(edge).from.unwrap();
        let to = self.graph.edge(edge).to.unwrap();
        let edge_length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
        assert!(self.graph.edge(edge).data.is_central());
        if end_position > edge_length {
            return false;
        }

        let (upward_edge, position) = if self.graph.edge_is_upward(edge) {
            (edge, end_position)
        } else {
            (
                self.graph.edge(edge).twin.unwrap(),
                edge_length - end_position,
            )
        };
        let storage = if let Some(storage) = self.graph.edge(upward_edge).data.transition_ends() {
            storage
        } else {
            let storage = Rc::new(RefCell::new(Vec::new()));
            self.graph
                .edge_mut(upward_edge)
                .data
                .set_transition_ends(&storage);
            self.transition_end_storage.push(storage.clone());
            storage
        };
        let transition_end = TransitionEnd::new(position, lower_bead_count, is_lower_end);
        let mut ends = storage.borrow_mut();
        if ends.first().is_some_and(|first| position < first.pos) {
            ends.insert(0, transition_end);
        } else {
            ends.push(transition_end);
        }
        true
    }
}
