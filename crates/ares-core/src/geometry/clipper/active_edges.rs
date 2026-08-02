use super::Clipper;
use super::predicates::top_x;
use super::types::EdgeId;

impl Clipper {
    pub(super) fn insert_edge_into_ael(&mut self, edge: EdgeId, start: Option<EdgeId>) {
        let Some(active) = self.active_edges else {
            let edge_state = self.edges.edge_mut(edge);
            edge_state.previous_in_ael = None;
            edge_state.next_in_ael = None;
            self.active_edges = Some(edge);
            return;
        };

        if start.is_none() && self.second_inserts_before_first(active, edge) {
            self.edges.edge_mut(edge).previous_in_ael = None;
            self.edges.edge_mut(edge).next_in_ael = Some(active);
            self.edges.edge_mut(active).previous_in_ael = Some(edge);
            self.active_edges = Some(edge);
            return;
        }

        let mut insertion = start.unwrap_or(active);
        while let Some(next) = self.edges.edge(insertion).next_in_ael {
            if self.second_inserts_before_first(next, edge) {
                break;
            }
            insertion = next;
        }
        let next = self.edges.edge(insertion).next_in_ael;
        self.edges.edge_mut(edge).next_in_ael = next;
        if let Some(next) = next {
            self.edges.edge_mut(next).previous_in_ael = Some(edge);
        }
        self.edges.edge_mut(edge).previous_in_ael = Some(insertion);
        self.edges.edge_mut(insertion).next_in_ael = Some(edge);
    }

    pub(super) fn delete_from_ael(&mut self, edge: EdgeId) {
        let snapshot = *self.edges.edge(edge);
        if snapshot.previous_in_ael.is_none()
            && snapshot.next_in_ael.is_none()
            && self.active_edges != Some(edge)
        {
            return;
        }
        if let Some(previous) = snapshot.previous_in_ael {
            self.edges.edge_mut(previous).next_in_ael = snapshot.next_in_ael;
        } else {
            self.active_edges = snapshot.next_in_ael;
        }
        if let Some(next) = snapshot.next_in_ael {
            self.edges.edge_mut(next).previous_in_ael = snapshot.previous_in_ael;
        }
        let edge = self.edges.edge_mut(edge);
        edge.next_in_ael = None;
        edge.previous_in_ael = None;
    }

    pub(super) fn delete_from_sel(&mut self, edge: EdgeId) {
        let snapshot = *self.edges.edge(edge);
        if snapshot.previous_in_sel.is_none()
            && snapshot.next_in_sel.is_none()
            && self.sorted_edges != Some(edge)
        {
            return;
        }
        if let Some(previous) = snapshot.previous_in_sel {
            self.edges.edge_mut(previous).next_in_sel = snapshot.next_in_sel;
        } else {
            self.sorted_edges = snapshot.next_in_sel;
        }
        if let Some(next) = snapshot.next_in_sel {
            self.edges.edge_mut(next).previous_in_sel = snapshot.previous_in_sel;
        }
        let edge = self.edges.edge_mut(edge);
        edge.next_in_sel = None;
        edge.previous_in_sel = None;
    }

    pub(super) fn add_edge_to_sel(&mut self, edge: EdgeId) {
        let head = self.sorted_edges;
        self.edges.edge_mut(edge).previous_in_sel = None;
        self.edges.edge_mut(edge).next_in_sel = head;
        if let Some(head) = head {
            self.edges.edge_mut(head).previous_in_sel = Some(edge);
        }
        self.sorted_edges = Some(edge);
    }

    pub(super) fn pop_edge_from_sel(&mut self) -> Option<EdgeId> {
        let edge = self.sorted_edges?;
        self.delete_from_sel(edge);
        Some(edge)
    }

    pub(super) fn copy_ael_to_sel(&mut self) {
        self.sorted_edges = self.active_edges;
        let mut edge = self.active_edges;
        while let Some(id) = edge {
            let snapshot = *self.edges.edge(id);
            let current = self.edges.edge_mut(id);
            current.previous_in_sel = snapshot.previous_in_ael;
            current.next_in_sel = snapshot.next_in_ael;
            edge = snapshot.next_in_ael;
        }
    }

    pub(super) fn swap_positions_in_ael(&mut self, first: EdgeId, second: EdgeId) {
        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        if first_edge.next_in_ael == first_edge.previous_in_ael
            || second_edge.next_in_ael == second_edge.previous_in_ael
        {
            return;
        }

        if first_edge.next_in_ael == Some(second) {
            self.swap_adjacent_ael(
                first,
                second,
                first_edge.previous_in_ael,
                second_edge.next_in_ael,
            );
        } else if second_edge.next_in_ael == Some(first) {
            self.swap_adjacent_ael(
                second,
                first,
                second_edge.previous_in_ael,
                first_edge.next_in_ael,
            );
        } else {
            self.edges.edge_mut(first).next_in_ael = second_edge.next_in_ael;
            if let Some(next) = second_edge.next_in_ael {
                self.edges.edge_mut(next).previous_in_ael = Some(first);
            }
            self.edges.edge_mut(first).previous_in_ael = second_edge.previous_in_ael;
            if let Some(previous) = second_edge.previous_in_ael {
                self.edges.edge_mut(previous).next_in_ael = Some(first);
            }
            self.edges.edge_mut(second).next_in_ael = first_edge.next_in_ael;
            if let Some(next) = first_edge.next_in_ael {
                self.edges.edge_mut(next).previous_in_ael = Some(second);
            }
            self.edges.edge_mut(second).previous_in_ael = first_edge.previous_in_ael;
            if let Some(previous) = first_edge.previous_in_ael {
                self.edges.edge_mut(previous).next_in_ael = Some(second);
            }
        }
        if self.edges.edge(first).previous_in_ael.is_none() {
            self.active_edges = Some(first);
        } else if self.edges.edge(second).previous_in_ael.is_none() {
            self.active_edges = Some(second);
        }
    }

    pub(super) fn swap_positions_in_sel(&mut self, first: EdgeId, second: EdgeId) {
        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        if first_edge.next_in_sel.is_none() && first_edge.previous_in_sel.is_none()
            || second_edge.next_in_sel.is_none() && second_edge.previous_in_sel.is_none()
        {
            return;
        }

        if first_edge.next_in_sel == Some(second) {
            self.swap_adjacent_sel(
                first,
                second,
                first_edge.previous_in_sel,
                second_edge.next_in_sel,
            );
        } else if second_edge.next_in_sel == Some(first) {
            self.swap_adjacent_sel(
                second,
                first,
                second_edge.previous_in_sel,
                first_edge.next_in_sel,
            );
        } else {
            self.edges.edge_mut(first).next_in_sel = second_edge.next_in_sel;
            if let Some(next) = second_edge.next_in_sel {
                self.edges.edge_mut(next).previous_in_sel = Some(first);
            }
            self.edges.edge_mut(first).previous_in_sel = second_edge.previous_in_sel;
            if let Some(previous) = second_edge.previous_in_sel {
                self.edges.edge_mut(previous).next_in_sel = Some(first);
            }
            self.edges.edge_mut(second).next_in_sel = first_edge.next_in_sel;
            if let Some(next) = first_edge.next_in_sel {
                self.edges.edge_mut(next).previous_in_sel = Some(second);
            }
            self.edges.edge_mut(second).previous_in_sel = first_edge.previous_in_sel;
            if let Some(previous) = first_edge.previous_in_sel {
                self.edges.edge_mut(previous).next_in_sel = Some(second);
            }
        }
        if self.edges.edge(first).previous_in_sel.is_none() {
            self.sorted_edges = Some(first);
        } else if self.edges.edge(second).previous_in_sel.is_none() {
            self.sorted_edges = Some(second);
        }
    }

    pub(super) fn update_edge_into_ael(&mut self, edge: EdgeId) -> EdgeId {
        let old = *self.edges.edge(edge);
        let next = old.next_in_lml.expect("edge update requires next LML edge");
        {
            let replacement = self.edges.edge_mut(next);
            replacement.output = old.output;
            replacement.side = old.side;
            replacement.wind_delta = old.wind_delta;
            replacement.wind_count = old.wind_count;
            replacement.alternate_wind_count = old.alternate_wind_count;
            replacement.current = replacement.bottom;
            replacement.previous_in_ael = old.previous_in_ael;
            replacement.next_in_ael = old.next_in_ael;
        }
        if let Some(previous) = old.previous_in_ael {
            self.edges.edge_mut(previous).next_in_ael = Some(next);
        } else {
            self.active_edges = Some(next);
        }
        if let Some(after) = old.next_in_ael {
            self.edges.edge_mut(after).previous_in_ael = Some(next);
        }
        if !self.edges.edge(next).is_horizontal() {
            self.scanbeam.push(self.edges.edge(next).top.y());
        }
        next
    }

    fn second_inserts_before_first(&self, first: EdgeId, second: EdgeId) -> bool {
        let first = *self.edges.edge(first);
        let second = *self.edges.edge(second);
        if second.current.x() == first.current.x() {
            if second.top.y() > first.top.y() {
                second.top.x() < top_x(first, second.top.y())
            } else {
                first.top.x() > top_x(second, first.top.y())
            }
        } else {
            second.current.x() < first.current.x()
        }
    }

    fn swap_adjacent_ael(
        &mut self,
        left: EdgeId,
        right: EdgeId,
        previous: Option<EdgeId>,
        next: Option<EdgeId>,
    ) {
        if let Some(next) = next {
            self.edges.edge_mut(next).previous_in_ael = Some(left);
        }
        if let Some(previous) = previous {
            self.edges.edge_mut(previous).next_in_ael = Some(right);
        }
        self.edges.edge_mut(right).previous_in_ael = previous;
        self.edges.edge_mut(right).next_in_ael = Some(left);
        self.edges.edge_mut(left).previous_in_ael = Some(right);
        self.edges.edge_mut(left).next_in_ael = next;
    }

    fn swap_adjacent_sel(
        &mut self,
        left: EdgeId,
        right: EdgeId,
        previous: Option<EdgeId>,
        next: Option<EdgeId>,
    ) {
        if let Some(next) = next {
            self.edges.edge_mut(next).previous_in_sel = Some(left);
        }
        if let Some(previous) = previous {
            self.edges.edge_mut(previous).next_in_sel = Some(right);
        }
        self.edges.edge_mut(right).previous_in_sel = previous;
        self.edges.edge_mut(right).next_in_sel = Some(left);
        self.edges.edge_mut(left).previous_in_sel = Some(right);
        self.edges.edge_mut(left).next_in_sel = next;
    }
}
