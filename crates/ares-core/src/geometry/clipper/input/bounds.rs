use super::super::Clipper;
use super::super::types::{EdgeId, EdgeSide, LocalMinimum, OutputIndex};
use crate::geometry::Point;

impl Clipper {
    pub(super) fn build_flat_open_minimum(&mut self, start: EdgeId) {
        let terminal = self.edges.edge(start).previous;
        self.edges.edge_mut(terminal).output = OutputIndex::Skipped;
        self.edges.edge_mut(start).side = EdgeSide::Right;
        self.edges.edge_mut(start).wind_delta = 0;
        let y = self.edges.edge(start).bottom.y();
        let mut edge = start;
        loop {
            let previous = self.edges.edge(edge).previous;
            if self.edges.edge(edge).bottom.x() != self.edges.edge(previous).top.x() {
                self.reverse_horizontal(edge);
            }
            let next = self.edges.edge(edge).next;
            if self.edges.edge(next).output == OutputIndex::Skipped {
                break;
            }
            self.edges.edge_mut(edge).next_in_lml = Some(next);
            edge = next;
        }
        self.minima.push(LocalMinimum {
            y,
            left: None,
            right: Some(start),
        });
    }

    pub(super) fn build_local_minima(&mut self, start: EdgeId, closed: bool) {
        let mut edge = start;
        let previous = self.edges.edge(edge).previous;
        if self.edges.edge(previous).bottom == self.edges.edge(previous).top {
            edge = self.edges.edge(edge).next;
        }

        let mut first_minimum = None;
        loop {
            edge = self.find_next_local_minimum(edge);
            if first_minimum == Some(edge) {
                break;
            }
            first_minimum.get_or_insert(edge);

            let previous = self.edges.edge(edge).previous;
            let (left, right, left_is_forward) =
                if self.edges.edge(edge).dx < self.edges.edge(previous).dx {
                    (previous, edge, false)
                } else {
                    (edge, previous, true)
                };
            let y = self.edges.edge(edge).bottom.y();

            let left_wind = if !closed {
                0
            } else if self.edges.edge(left).next == right {
                -1
            } else {
                1
            };
            self.edges.edge_mut(left).wind_delta = left_wind;
            self.edges.edge_mut(right).wind_delta = -left_wind;

            edge = self.process_bound(left, left_is_forward);
            if self.edges.edge(edge).output == OutputIndex::Skipped {
                edge = self.process_bound(edge, left_is_forward);
            }
            let mut right_result = self.process_bound(right, !left_is_forward);
            if self.edges.edge(right_result).output == OutputIndex::Skipped {
                right_result = self.process_bound(right_result, !left_is_forward);
            }

            let mut left_bound = Some(left);
            let mut right_bound = Some(right);
            if self.edges.edge(left).output == OutputIndex::Skipped {
                left_bound = None;
            } else if self.edges.edge(right).output == OutputIndex::Skipped {
                right_bound = None;
            }
            self.minima.push(LocalMinimum {
                y,
                left: left_bound,
                right: right_bound,
            });

            if !left_is_forward {
                edge = right_result;
            }
        }
    }

    fn find_next_local_minimum(&self, mut edge: EdgeId) -> EdgeId {
        loop {
            edge = self.next_minimum_candidate(edge);
            let previous = self.edges.edge(edge).previous;
            if !self.edges.edge(edge).is_horizontal() && !self.edges.edge(previous).is_horizontal()
            {
                return edge;
            }
            while self
                .edges
                .edge(self.edges.edge(edge).previous)
                .is_horizontal()
            {
                edge = self.edges.edge(edge).previous;
            }
            let horizontal_start = edge;
            while self.edges.edge(edge).is_horizontal() {
                edge = self.edges.edge(edge).next;
            }
            let previous = self.edges.edge(edge).previous;
            if self.edges.edge(edge).top.y() == self.edges.edge(previous).bottom.y() {
                continue;
            }
            let before_horizontal = self.edges.edge(horizontal_start).previous;
            return if self.edges.edge(before_horizontal).bottom.x()
                < self.edges.edge(edge).bottom.x()
            {
                horizontal_start
            } else {
                edge
            };
        }
    }

    fn next_minimum_candidate(&self, mut edge: EdgeId) -> EdgeId {
        loop {
            let current = self.edges.edge(edge);
            let previous = self.edges.edge(current.previous);
            if current.bottom == previous.bottom && current.current != current.top {
                return edge;
            }
            edge = current.next;
        }
    }

    fn process_bound(&mut self, edge: EdgeId, next_is_forward: bool) -> EdgeId {
        if self.edges.edge(edge).output == OutputIndex::Skipped {
            return self.process_skipped_bound(edge, next_is_forward);
        }
        self.normalize_bound_start(edge, next_is_forward);
        if next_is_forward {
            self.process_forward_bound(edge)
        } else {
            self.process_reverse_bound(edge)
        }
    }

    fn process_skipped_bound(&mut self, start: EdgeId, next_is_forward: bool) -> EdgeId {
        let mut edge = start;
        if next_is_forward {
            while self.edges.edge(edge).top.y()
                == self.edges.edge(self.edges.edge(edge).next).bottom.y()
            {
                edge = self.edges.edge(edge).next;
            }
            while edge != start && self.edges.edge(edge).is_horizontal() {
                edge = self.edges.edge(edge).previous;
            }
        } else {
            while self.edges.edge(edge).top.y()
                == self.edges.edge(self.edges.edge(edge).previous).bottom.y()
            {
                edge = self.edges.edge(edge).previous;
            }
            while edge != start && self.edges.edge(edge).is_horizontal() {
                edge = self.edges.edge(edge).next;
            }
        }

        if edge == start {
            return if next_is_forward {
                self.edges.edge(edge).next
            } else {
                self.edges.edge(edge).previous
            };
        }

        edge = if next_is_forward {
            self.edges.edge(start).next
        } else {
            self.edges.edge(start).previous
        };
        let y = self.edges.edge(edge).bottom.y();
        self.edges.edge_mut(edge).wind_delta = 0;
        let result = self.process_bound(edge, next_is_forward);
        self.minima.push(LocalMinimum {
            y,
            left: None,
            right: Some(edge),
        });
        result
    }

    fn normalize_bound_start(&mut self, edge: EdgeId, next_is_forward: bool) {
        if !self.edges.edge(edge).is_horizontal() {
            return;
        }
        let adjoining = if next_is_forward {
            self.edges.edge(edge).previous
        } else {
            self.edges.edge(edge).next
        };
        let should_reverse = if self.edges.edge(adjoining).is_horizontal() {
            self.edges.edge(adjoining).bottom.x() != self.edges.edge(edge).bottom.x()
                && self.edges.edge(adjoining).top.x() != self.edges.edge(edge).bottom.x()
        } else {
            self.edges.edge(adjoining).bottom.x() != self.edges.edge(edge).bottom.x()
        };
        if should_reverse {
            self.reverse_horizontal(edge);
        }
    }

    fn process_forward_bound(&mut self, start: EdgeId) -> EdgeId {
        let mut result = start;
        while self.edges.edge(result).top.y()
            == self.edges.edge(self.edges.edge(result).next).bottom.y()
            && self.edges.edge(self.edges.edge(result).next).output != OutputIndex::Skipped
        {
            result = self.edges.edge(result).next;
        }
        if self.edges.edge(result).is_horizontal()
            && self.edges.edge(self.edges.edge(result).next).output != OutputIndex::Skipped
        {
            let mut horizontal = result;
            while self
                .edges
                .edge(self.edges.edge(horizontal).previous)
                .is_horizontal()
            {
                horizontal = self.edges.edge(horizontal).previous;
            }
            let previous = self.edges.edge(horizontal).previous;
            let next = self.edges.edge(result).next;
            if self.edges.edge(previous).top.x() > self.edges.edge(next).top.x() {
                result = previous;
            }
        }
        let mut edge = start;
        while edge != result {
            let next = self.edges.edge(edge).next;
            self.edges.edge_mut(edge).next_in_lml = Some(next);
            self.align_forward_horizontal(edge, start);
            edge = next;
        }
        self.align_forward_horizontal(edge, start);
        self.edges.edge(result).next
    }

    fn align_forward_horizontal(&mut self, edge: EdgeId, start: EdgeId) {
        let previous = self.edges.edge(edge).previous;
        if edge != start
            && self.edges.edge(edge).is_horizontal()
            && self.edges.edge(edge).bottom.x() != self.edges.edge(previous).top.x()
        {
            self.reverse_horizontal(edge);
        }
    }

    fn process_reverse_bound(&mut self, start: EdgeId) -> EdgeId {
        let mut result = start;
        while self.edges.edge(result).top.y()
            == self.edges.edge(self.edges.edge(result).previous).bottom.y()
            && self.edges.edge(self.edges.edge(result).previous).output != OutputIndex::Skipped
        {
            result = self.edges.edge(result).previous;
        }
        if self.edges.edge(result).is_horizontal()
            && self.edges.edge(self.edges.edge(result).previous).output != OutputIndex::Skipped
        {
            let mut horizontal = result;
            while self
                .edges
                .edge(self.edges.edge(horizontal).next)
                .is_horizontal()
            {
                horizontal = self.edges.edge(horizontal).next;
            }
            let next = self.edges.edge(horizontal).next;
            let previous = self.edges.edge(result).previous;
            if self.edges.edge(next).top.x() == self.edges.edge(previous).top.x()
                || self.edges.edge(next).top.x() > self.edges.edge(previous).top.x()
            {
                result = next;
            }
        }
        let mut edge = start;
        while edge != result {
            let previous = self.edges.edge(edge).previous;
            self.edges.edge_mut(edge).next_in_lml = Some(previous);
            self.align_reverse_horizontal(edge, start);
            edge = previous;
        }
        self.align_reverse_horizontal(edge, start);
        self.edges.edge(result).previous
    }

    fn align_reverse_horizontal(&mut self, edge: EdgeId, start: EdgeId) {
        let next = self.edges.edge(edge).next;
        if edge != start
            && self.edges.edge(edge).is_horizontal()
            && self.edges.edge(edge).bottom.x() != self.edges.edge(next).top.x()
        {
            self.reverse_horizontal(edge);
        }
    }

    fn reverse_horizontal(&mut self, edge: EdgeId) {
        let bottom = self.edges.edge(edge).bottom;
        let top = self.edges.edge(edge).top;
        let edge = self.edges.edge_mut(edge);
        edge.bottom = Point::new(top.x(), bottom.y());
        edge.top = Point::new(bottom.x(), top.y());
    }
}
