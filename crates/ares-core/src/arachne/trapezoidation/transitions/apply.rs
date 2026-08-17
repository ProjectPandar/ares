use std::{cell::RefCell, rc::Rc};

use crate::arachne::skeletal::{EdgeId, TransitionEnd};

use super::{SkeletalTrapezoidation, point_at_distance, point_distance};

impl SkeletalTrapezoidation<'_> {
    pub(super) fn apply_transitions(&mut self) {
        self.normalize_transition_end_directions();
        let snap_distance = self.config.coordinate_scale.checked_scale(0.02).unwrap();
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            self.apply_transitions_on_edge(edge, snap_distance);
        }
    }

    fn apply_transitions_on_edge(&mut self, edge: EdgeId, snap_distance: i64) {
        let Some(storage) = self.graph.edge(edge).data.transition_ends() else {
            return;
        };
        let mut transitions = storage.borrow().clone();
        if transitions.is_empty() {
            return;
        }
        assert!(self.graph.edge(edge).data.is_central());
        transitions.sort_unstable_by_key(|end| end.pos);
        let from = self.graph.edge(edge).from.unwrap();
        let to = self.graph.edge(edge).to.unwrap();
        let start = self.graph.node(from).point;
        let finish = self.graph.node(to).point;
        let edge_length = point_distance(start, finish);
        let mut last_edge = edge;
        for transition_end in transitions {
            assert!((0..=edge_length).contains(&transition_end.pos));
            let bead_count = i64::from(transition_end.lower_bead_count)
                + i64::from(!transition_end.is_lower_end);
            let close_node = if transition_end.pos < edge_length / 2 {
                from
            } else {
                to
            };
            if (transition_end.pos < snap_distance
                || transition_end.pos > edge_length - snap_distance)
                && self.graph.node(close_node).data.bead_count == bead_count
            {
                self.graph.node_mut(close_node).data.transition_ratio = 0.0;
                continue;
            }
            let middle = point_at_distance(start, finish, transition_end.pos);
            last_edge = self.graph.insert_node(last_edge, middle, bead_count);
        }
    }

    fn normalize_transition_end_directions(&mut self) {
        let edges = self.graph.active_edges().collect::<Vec<_>>();
        for edge in edges {
            let twin = self.graph.edge(edge).twin.unwrap();
            let Some(twin_storage) = self.graph.edge(twin).data.transition_ends() else {
                continue;
            };
            let twin_ends = std::mem::take(&mut *twin_storage.borrow_mut());
            if twin_ends.is_empty() {
                continue;
            }
            let from = self.graph.edge(edge).from.unwrap();
            let to = self.graph.edge(edge).to.unwrap();
            let edge_length =
                point_distance(self.graph.node(from).point, self.graph.node(to).point);
            let storage = self.transition_end_storage(edge);
            storage
                .borrow_mut()
                .extend(twin_ends.into_iter().map(|end| {
                    TransitionEnd::new(
                        edge_length - end.pos,
                        end.lower_bead_count,
                        end.is_lower_end,
                    )
                }));
        }
    }

    fn transition_end_storage(&mut self, edge: EdgeId) -> Rc<RefCell<Vec<TransitionEnd>>> {
        if let Some(storage) = self.graph.edge(edge).data.transition_ends() {
            return storage;
        }
        let storage = Rc::new(RefCell::new(Vec::new()));
        self.graph.edge_mut(edge).data.set_transition_ends(&storage);
        self.transition_end_storage.push(storage.clone());
        storage
    }
}

#[cfg(test)]
mod tests;
