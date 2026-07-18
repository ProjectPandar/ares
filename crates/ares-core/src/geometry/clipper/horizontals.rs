use super::ClosedClipper;
use super::predicates::slopes_equal_four;
use super::types::{EdgeId, ExecutionConfig, GhostJoin, Join, OutPointId, OutputIndex};
use crate::geometry::Point;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Direction {
    LeftToRight,
    RightToLeft,
}

#[derive(Clone, Copy)]
struct HorizontalScan {
    direction: Direction,
    left: i64,
    right: i64,
    is_last: bool,
    maxima_pair: Option<EdgeId>,
    config: ExecutionConfig,
}

impl ClosedClipper {
    pub(super) fn process_horizontals(&mut self, config: ExecutionConfig) {
        while let Some(horizontal) = self.pop_edge_from_sel() {
            self.process_horizontal(horizontal, config);
        }
    }

    fn process_horizontal(&mut self, mut horizontal: EdgeId, config: ExecutionConfig) {
        let (mut direction, mut left, mut right) = self.horizontal_direction(horizontal);
        let mut last_horizontal = horizontal;
        while let Some(next) = self.edges.edge(last_horizontal).next_in_lml {
            if !self.edges.edge(next).is_horizontal() {
                break;
            }
            last_horizontal = next;
        }
        let maxima_pair = self
            .edges
            .edge(last_horizontal)
            .next_in_lml
            .is_none()
            .then(|| self.horizontal_maxima_pair(last_horizontal))
            .flatten();
        let mut output_point = None;

        loop {
            let is_last = horizontal == last_horizontal;
            if self.scan_horizontal_crossings(
                horizontal,
                HorizontalScan {
                    direction,
                    left,
                    right,
                    is_last,
                    maxima_pair,
                    config,
                },
                &mut output_point,
            ) {
                return;
            }

            let Some(next) = self.edges.edge(horizontal).next_in_lml else {
                break;
            };
            if !self.edges.edge(next).is_horizontal() {
                break;
            }
            horizontal = self.update_edge_into_ael(horizontal);
            if matches!(self.edges.edge(horizontal).output, OutputIndex::Assigned(_)) {
                self.add_out_point(horizontal, self.edges.edge(horizontal).bottom);
            }
            (direction, left, right) = self.horizontal_direction(horizontal);
        }

        if matches!(self.edges.edge(horizontal).output, OutputIndex::Assigned(_))
            && output_point.is_none()
        {
            let point = self.last_out_point(horizontal);
            self.join_pending_horizontals(horizontal, point);
            self.ghost_joins.push(GhostJoin {
                point,
                offset: self.edges.edge(horizontal).top,
            });
        }

        if self.edges.edge(horizontal).next_in_lml.is_some() {
            if matches!(self.edges.edge(horizontal).output, OutputIndex::Assigned(_)) {
                let point = self.add_out_point(horizontal, self.edges.edge(horizontal).top);
                let promoted = self.update_edge_into_ael(horizontal);
                self.join_horizontal_promotion(promoted, point);
            } else {
                self.update_edge_into_ael(horizontal);
            }
        } else {
            if matches!(self.edges.edge(horizontal).output, OutputIndex::Assigned(_)) {
                self.add_out_point(horizontal, self.edges.edge(horizontal).top);
            }
            self.delete_from_ael(horizontal);
        }
    }

    fn scan_horizontal_crossings(
        &mut self,
        horizontal: EdgeId,
        scan: HorizontalScan,
        output_point: &mut Option<OutPointId>,
    ) -> bool {
        let mut edge = match scan.direction {
            Direction::LeftToRight => self.edges.edge(horizontal).next_in_ael,
            Direction::RightToLeft => self.edges.edge(horizontal).previous_in_ael,
        };
        while let Some(crossing) = edge {
            let crossing_edge = *self.edges.edge(crossing);
            if scan.direction == Direction::LeftToRight && crossing_edge.current.x() > scan.right
                || scan.direction == Direction::RightToLeft && crossing_edge.current.x() < scan.left
            {
                break;
            }
            let horizontal_edge = *self.edges.edge(horizontal);
            if crossing_edge.current.x() == horizontal_edge.top.x()
                && let Some(next) = horizontal_edge.next_in_lml
                && crossing_edge.dx < self.edges.edge(next).dx
            {
                break;
            }
            if matches!(horizontal_edge.output, OutputIndex::Assigned(_)) {
                let point = self.add_out_point(horizontal, crossing_edge.current);
                *output_point = Some(point);
                self.join_pending_horizontals(horizontal, point);
                self.ghost_joins.push(GhostJoin {
                    point,
                    offset: horizontal_edge.bottom,
                });
            }
            if Some(crossing) == scan.maxima_pair && scan.is_last {
                self.finish_horizontal_maximum(horizontal, crossing);
                return true;
            }
            let intersection = Point::new(
                crossing_edge.current.x(),
                self.edges.edge(horizontal).current.y(),
            );
            match scan.direction {
                Direction::LeftToRight => {
                    self.intersect_edges(horizontal, crossing, intersection, scan.config)
                }
                Direction::RightToLeft => {
                    self.intersect_edges(crossing, horizontal, intersection, scan.config)
                }
            }
            edge = match scan.direction {
                Direction::LeftToRight => crossing_edge.next_in_ael,
                Direction::RightToLeft => crossing_edge.previous_in_ael,
            };
            self.swap_positions_in_ael(horizontal, crossing);
        }
        false
    }

    fn finish_horizontal_maximum(&mut self, horizontal: EdgeId, crossing: EdgeId) {
        if matches!(self.edges.edge(horizontal).output, OutputIndex::Assigned(_)) {
            self.add_local_max_polygon(horizontal, crossing, self.edges.edge(horizontal).top);
        }
        self.delete_from_ael(horizontal);
        self.delete_from_ael(crossing);
    }

    fn join_pending_horizontals(&mut self, horizontal: EdgeId, point: OutPointId) {
        let horizontal_edge = *self.edges.edge(horizontal);
        let mut pending = self.sorted_edges;
        while let Some(edge) = pending {
            let edge_state = *self.edges.edge(edge);
            if matches!(edge_state.output, OutputIndex::Assigned(_))
                && horizontal_segments_overlap(
                    horizontal_edge.bottom.x(),
                    horizontal_edge.top.x(),
                    edge_state.bottom.x(),
                    edge_state.top.x(),
                )
            {
                self.joins.push(Join {
                    first: self.last_out_point(edge),
                    second: point,
                    offset: edge_state.top,
                });
            }
            pending = edge_state.next_in_sel;
        }
    }

    fn join_horizontal_promotion(&mut self, edge: EdgeId, point: OutPointId) {
        let edge_state = *self.edges.edge(edge);
        for neighbour in [edge_state.previous_in_ael, edge_state.next_in_ael]
            .into_iter()
            .flatten()
        {
            let other = *self.edges.edge(neighbour);
            if other.current == edge_state.bottom
                && other.wind_delta != 0
                && matches!(other.output, OutputIndex::Assigned(_))
                && other.current.y() > other.top.y()
                && slopes_equal_four(
                    edge_state.bottom,
                    edge_state.top,
                    other.bottom,
                    other.top,
                    self.use_full_range,
                )
            {
                let second = self.add_out_point(neighbour, edge_state.bottom);
                self.joins.push(Join {
                    first: point,
                    second,
                    offset: edge_state.top,
                });
                break;
            }
        }
    }

    fn horizontal_direction(&self, edge: EdgeId) -> (Direction, i64, i64) {
        let edge = self.edges.edge(edge);
        if edge.bottom.x() < edge.top.x() {
            (Direction::LeftToRight, edge.bottom.x(), edge.top.x())
        } else {
            (Direction::RightToLeft, edge.top.x(), edge.bottom.x())
        }
    }

    fn horizontal_maxima_pair(&self, edge: EdgeId) -> Option<EdgeId> {
        let edge_state = self.edges.edge(edge);
        let next = self.edges.edge(edge_state.next);
        if next.top == edge_state.top && next.next_in_lml.is_none() {
            Some(edge_state.next)
        } else {
            let previous = self.edges.edge(edge_state.previous);
            (previous.top == edge_state.top && previous.next_in_lml.is_none())
                .then_some(edge_state.previous)
        }
    }
}

fn horizontal_segments_overlap(first: i64, second: i64, third: i64, fourth: i64) -> bool {
    let (first_left, first_right) = ordered(first, second);
    let (second_left, second_right) = ordered(third, fourth);
    first_left < second_right && second_left < first_right
}

fn ordered(first: i64, second: i64) -> (i64, i64) {
    if first < second {
        (first, second)
    } else {
        (second, first)
    }
}
