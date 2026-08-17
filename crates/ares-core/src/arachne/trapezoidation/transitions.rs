use std::{cell::RefCell, rc::Rc};

use crate::geometry::Point;

use super::SkeletalTrapezoidation;
use crate::arachne::skeletal::{EdgeId, TransitionMiddle};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TransitionMidRef {
    edge: EdgeId,
    index: usize,
}

#[derive(Clone, Copy)]
struct NearbyTransitionSearch {
    origin: TransitionMiddle,
    maximum_distance: i64,
    going_up: bool,
}

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

    #[expect(
        clippy::excessive_nesting,
        reason = "the source requires every central branch to reach a compatible transition"
    )]
    fn dissolve_nearby_transitions(
        &self,
        edge_to_start: EdgeId,
        traveled_distance: i64,
        search: NearbyTransitionSearch,
    ) -> Vec<TransitionMidRef> {
        if traveled_distance > search.maximum_distance {
            return Vec::new();
        }

        let stop = self.graph.edge(edge_to_start).twin.unwrap();
        let mut next = self.graph.edge(edge_to_start).next;
        let mut discovered = Vec::new();
        let mut should_dissolve = true;
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
            let edge_length =
                point_distance(self.graph.node(from).point, self.graph.node(to).point);
            let is_aligned = self.graph.edge_is_upward(edge);
            let aligned = if is_aligned { edge } else { twin };
            let width_deviation = (search.origin.feature_radius
                - self.graph.node(from).data.distance_to_boundary)
                .abs()
                * 2;
            let dissolve_result_is_odd =
                (search.origin.lower_bead_count % 2 != 0) == search.going_up;
            let line_width_deviation = if dissolve_result_is_odd {
                width_deviation
            } else {
                width_deviation / 2
            };
            if line_width_deviation > self.config.allowed_filter_deviation {
                should_dissolve = false;
            }

            let mut seen_on_edge = false;
            if should_dissolve && let Some(storage) = self.graph.edge(aligned).data.transitions() {
                let transitions = storage.borrow();
                for (index, transition) in transitions.iter().enumerate() {
                    let position = if is_aligned {
                        transition.pos
                    } else {
                        edge_length - transition.pos
                    };
                    if traveled_distance + position < search.maximum_distance
                        && transition.lower_bead_count == search.origin.lower_bead_count
                    {
                        if traveled_distance + position
                            < self
                                .beading_strategy
                                .transitioning_length(i64::from(transition.lower_bead_count))
                        {
                            assert!(
                                search.going_up != is_aligned || transition.lower_bead_count == 0
                            );
                        }
                        discovered.push(TransitionMidRef {
                            edge: aligned,
                            index,
                        });
                        seen_on_edge = true;
                    }
                }
            }
            if should_dissolve && !seen_on_edge {
                let nested =
                    self.dissolve_nearby_transitions(edge, traveled_distance + edge_length, search);
                if nested.is_empty() {
                    return Vec::new();
                }
                discovered.extend(nested);
            }
        }

        if should_dissolve {
            discovered
        } else {
            Vec::new()
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
