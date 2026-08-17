mod apply;
mod ends;
mod filtering;
mod ribs;
mod segments;

#[cfg(test)]
mod test_support;

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
}

fn point_distance(left: Point, right: Point) -> i64 {
    let dx = (left.x() - right.x()) as f64;
    let dy = (left.y() - right.y()) as f64;
    (dx * dx + dy * dy).sqrt() as i64
}

fn point_at_distance(start: Point, finish: Point, distance: i64) -> Point {
    let dx = finish.x() - start.x();
    let dy = finish.y() - start.y();
    let length = point_distance(start, finish);
    if length < 1 {
        return Point::new(start.x() + distance, start.y());
    }
    Point::new(
        start.x() + (i128::from(dx) * i128::from(distance) / i128::from(length)) as i64,
        start.y() + (i128::from(dy) * i128::from(distance) / i128::from(length)) as i64,
    )
}

#[cfg(test)]
mod tests;
