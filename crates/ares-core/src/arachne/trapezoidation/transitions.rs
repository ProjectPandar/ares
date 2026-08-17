use std::{cell::RefCell, rc::Rc};

use crate::geometry::Point;

use super::SkeletalTrapezoidation;
use crate::arachne::skeletal::{EdgeId, TransitionMiddle};

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
                let transition_radius =
                    (self.beading_strategy.transition_thickness(lower_bead_count) / 2)
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

    fn dissolve_bead_count_region(
        &mut self,
        edge_to_start: EdgeId,
        source_bead_count: i64,
        replacement_bead_count: i64,
    ) {
        let to = self.graph.edge(edge_to_start).to.unwrap();
        if self.graph.node(to).data.bead_count != source_bead_count {
            return;
        }
        self.graph.node_mut(to).data.bead_count = replacement_bead_count;

        let stop = self.graph.edge(edge_to_start).twin.unwrap();
        let mut next = self.graph.edge(edge_to_start).next;
        while let Some(edge) = next {
            if edge == stop {
                break;
            }
            let twin = self.graph.edge(edge).twin.unwrap();
            next = self.graph.edge(twin).next;
            if self.graph.edge(edge).data.is_central() {
                self.dissolve_bead_count_region(edge, source_bead_count, replacement_bead_count);
            }
        }
    }
    fn filter_end_of_central_transition(
        &mut self,
        edge_to_start: EdgeId,
        traveled_distance: i64,
        maximum_distance: i64,
        replacement_bead_count: i64,
    ) -> bool {
        if traveled_distance > maximum_distance {
            return false;
        }

        let stop = self.graph.edge(edge_to_start).twin.unwrap();
        let mut next = self.graph.edge(edge_to_start).next;
        let mut is_end_of_central = true;
        let mut should_replace = false;
        while let Some(edge) = next {
            if edge == stop {
                break;
            }
            let twin = self.graph.edge(edge).twin.unwrap();
            next = self.graph.edge(twin).next;
            if !self.graph.edge(edge).data.is_central() {
                continue;
            }
            let from = self.graph.edge(edge).from.unwrap();
            let to = self.graph.edge(edge).to.unwrap();
            let length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
            should_replace |= self.filter_end_of_central_transition(
                edge,
                traveled_distance + length,
                maximum_distance,
                replacement_bead_count,
            );
            is_end_of_central = false;
        }
        if is_end_of_central && traveled_distance < maximum_distance {
            should_replace = true;
        }
        if should_replace {
            let to = self.graph.edge(edge_to_start).to.unwrap();
            self.graph.node_mut(to).data.bead_count = replacement_bead_count;
        }
        should_replace
    }
}

fn point_distance(left: Point, right: Point) -> i64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    (dx * dx + dy * dy).sqrt() as i64
}

#[cfg(test)]
mod tests;
