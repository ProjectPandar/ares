use std::{cell::RefCell, rc::Rc};

use crate::arachne::skeletal::{EdgeId, TransitionEnd, TransitionMiddle};

use super::{SkeletalTrapezoidation, point_distance};

#[derive(Clone, Copy)]
struct TransitionEndSearch {
    start_position: i64,
    end_position: i64,
    start_rest: f64,
    end_rest: f64,
    lower_bead_count: i32,
}

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
        self.generate_transition_end(
            twin,
            TransitionEndSearch {
                start_position: edge_length - middle.pos,
                end_position: edge_length - middle.pos + lower_half_length,
                start_rest: anchor,
                end_rest: 0.0,
                lower_bead_count: middle.lower_bead_count,
            },
        );

        let upper_half_length = ((1.0 - anchor) * transition_length as f64) as i64;
        self.generate_transition_end(
            edge,
            TransitionEndSearch {
                start_position: middle.pos,
                end_position: middle.pos + upper_half_length,
                start_rest: anchor,
                end_rest: 1.0,
                lower_bead_count: middle.lower_bead_count,
            },
        );
    }

    fn generate_transition_end(&mut self, edge: EdgeId, search: TransitionEndSearch) -> bool {
        let from = self.graph.edge(edge).from.unwrap();
        let to = self.graph.edge(edge).to.unwrap();
        let edge_length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
        assert!(search.start_position <= edge_length);
        assert!(self.graph.edge(edge).data.is_central());

        if search.end_position > edge_length {
            return self.continue_transition_end(edge, edge_length, search);
        }

        let (upward_edge, position) = if self.graph.edge_is_upward(edge) {
            (edge, search.end_position)
        } else {
            (
                self.graph.edge(edge).twin.unwrap(),
                edge_length - search.end_position,
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
        let transition_end =
            TransitionEnd::new(position, search.lower_bead_count, search.end_rest == 0.0);
        let mut ends = storage.borrow_mut();
        if ends.first().is_some_and(|first| position < first.pos) {
            ends.insert(0, transition_end);
        } else {
            ends.push(transition_end);
        }
        false
    }

    fn continue_transition_end(
        &mut self,
        edge: EdgeId,
        edge_length: i64,
        search: TransitionEndSearch,
    ) -> bool {
        let rest = search.end_rest
            - (search.start_rest - search.end_rest) * (search.end_position - edge_length) as f64
                / (search.start_position - search.end_position) as f64;
        assert!(rest >= search.start_rest.min(search.end_rest));
        assert!(rest <= search.start_rest.max(search.end_rest));

        let stop = self.graph.edge(edge).twin.unwrap();
        let mut next = self.graph.edge(edge).next;
        let mut is_only_going_down = true;
        let mut has_recursed = false;
        while let Some(outgoing) = next {
            if outgoing == stop {
                break;
            }
            let twin = self.graph.edge(outgoing).twin.unwrap();
            next = self.graph.edge(twin).next;
            if !self.graph.edge(outgoing).data.is_central() {
                continue;
            }
            is_only_going_down &= self.generate_transition_end(
                outgoing,
                TransitionEndSearch {
                    start_position: 0,
                    end_position: search.end_position - edge_length,
                    start_rest: rest,
                    ..search
                },
            );
            has_recursed = true;
        }
        if search.end_rest <= search.start_rest || (has_recursed && !is_only_going_down) {
            let to = self.graph.edge(edge).to.unwrap();
            self.graph.node_mut(to).data.transition_ratio = rest as f32;
            self.graph.node_mut(to).data.bead_count = i64::from(search.lower_bead_count);
        }
        is_only_going_down
    }
}

#[cfg(test)]
mod tests;
