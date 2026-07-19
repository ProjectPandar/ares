use super::ClosedClipper;
use super::ordering::fixed_msvc_sort_by;
use super::predicates::slopes_equal_four;
use super::types::{ExecutionConfig, Join, LocalMinimum, OutputIndex};

impl ClosedClipper {
    pub(super) fn reset_for_execute(&mut self) {
        self.maxima.clear();
        #[cfg(test)]
        self.collected_maxima_for_test.clear();
        fixed_msvc_sort_by(&mut self.minima, |first, second| first.y < second.y);
        for minimum in self.minima.iter().copied() {
            let left_bottom = self.edges.edge(minimum.left).bottom;
            let left = self.edges.edge_mut(minimum.left);
            left.current = left_bottom;
            left.side = super::types::EdgeSide::Left;
            left.output = OutputIndex::Unassigned;

            let right_bottom = self.edges.edge(minimum.right).bottom;
            let right = self.edges.edge_mut(minimum.right);
            right.current = right_bottom;
            right.side = super::types::EdgeSide::Right;
            right.output = OutputIndex::Unassigned;
        }
        self.scanbeam.clear();
        self.active_edges = None;
        self.sorted_edges = None;
        for minimum in self.minima.iter().rev() {
            self.scanbeam.push(minimum.y);
        }
    }

    pub(super) fn insert_local_minima_into_ael(&mut self, bottom_y: i64, config: ExecutionConfig) {
        while self
            .minima
            .last()
            .is_some_and(|minimum| minimum.y == bottom_y)
        {
            let minimum = self.minima.pop().expect("matching local minimum exists");
            self.insert_local_minimum(minimum, config);
        }
    }

    fn insert_local_minimum(&mut self, minimum: LocalMinimum, config: ExecutionConfig) {
        let left = minimum.left;
        let right = minimum.right;
        self.insert_edge_into_ael(left, None);
        self.insert_edge_into_ael(right, Some(left));
        self.set_winding_count(left, config);
        let left_edge = *self.edges.edge(left);
        self.edges.edge_mut(right).wind_count = left_edge.wind_count;
        self.edges.edge_mut(right).alternate_wind_count = left_edge.alternate_wind_count;
        let output = self
            .is_contributing(left, config)
            .then(|| self.add_local_min_polygon(left, right, left_edge.bottom));
        self.scanbeam.push(left_edge.top.y());
        let right_edge = *self.edges.edge(right);
        if right_edge.is_horizontal() {
            self.add_edge_to_sel(right);
            if let Some(next) = right_edge.next_in_lml {
                self.scanbeam.push(self.edges.edge(next).top.y());
            }
        } else {
            self.scanbeam.push(right_edge.top.y());
        }
        let Some(output) = output else {
            self.intersect_minimum_edges(left, right, config);
            return;
        };
        self.join_minimum_ghosts(right_edge, output);
        self.join_minimum_left(left, output);
        self.join_minimum_right(left, right, output);
        self.intersect_minimum_edges(left, right, config);
    }

    fn join_minimum_ghosts(
        &mut self,
        right_edge: super::types::Edge,
        output: super::types::OutPointId,
    ) {
        if !right_edge.is_horizontal() || right_edge.wind_delta == 0 {
            return;
        }
        for index in 0..self.ghost_joins.len() {
            let ghost = self.ghost_joins[index];
            if horizontal_segments_overlap(
                self.out_points.point(ghost.point).point.x(),
                ghost.offset.x(),
                right_edge.bottom.x(),
                right_edge.top.x(),
            ) {
                self.joins.push(Join {
                    first: ghost.point,
                    second: output,
                    offset: ghost.offset,
                });
            }
        }
    }

    fn join_minimum_left(&mut self, left: super::types::EdgeId, output: super::types::OutPointId) {
        let left_edge = *self.edges.edge(left);
        let Some(previous) = left_edge.previous_in_ael else {
            return;
        };
        let previous_edge = *self.edges.edge(previous);
        if matches!(left_edge.output, OutputIndex::Assigned(_))
            && previous_edge.current.x() == left_edge.bottom.x()
            && matches!(previous_edge.output, OutputIndex::Assigned(_))
            && slopes_equal_four(
                previous_edge.bottom,
                previous_edge.top,
                left_edge.current,
                left_edge.top,
                self.use_full_range,
            )
            && left_edge.wind_delta != 0
            && previous_edge.wind_delta != 0
        {
            let second = self.add_out_point(previous, left_edge.bottom);
            self.joins.push(Join {
                first: output,
                second,
                offset: left_edge.top,
            });
        }
    }

    fn join_minimum_right(
        &mut self,
        left: super::types::EdgeId,
        right: super::types::EdgeId,
        output: super::types::OutPointId,
    ) {
        if self.edges.edge(left).next_in_ael == Some(right) {
            return;
        }
        let right_edge = *self.edges.edge(right);
        let Some(previous) = right_edge.previous_in_ael else {
            return;
        };
        let previous_edge = *self.edges.edge(previous);
        if matches!(right_edge.output, OutputIndex::Assigned(_))
            && matches!(previous_edge.output, OutputIndex::Assigned(_))
            && slopes_equal_four(
                previous_edge.current,
                previous_edge.top,
                right_edge.current,
                right_edge.top,
                self.use_full_range,
            )
            && right_edge.wind_delta != 0
            && previous_edge.wind_delta != 0
        {
            let second = self.add_out_point(previous, right_edge.bottom);
            self.joins.push(Join {
                first: output,
                second,
                offset: right_edge.top,
            });
        }
    }

    fn intersect_minimum_edges(
        &mut self,
        left: super::types::EdgeId,
        right: super::types::EdgeId,
        config: ExecutionConfig,
    ) {
        let mut edge = self.edges.edge(left).next_in_ael;
        while let Some(current) = edge {
            if current == right {
                break;
            }
            self.intersect_edges(right, current, self.edges.edge(left).current, config);
            edge = self.edges.edge(current).next_in_ael;
        }
    }
}

fn horizontal_segments_overlap(first: i64, second: i64, third: i64, fourth: i64) -> bool {
    let (first_left, first_right) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    let (second_left, second_right) = if third < fourth {
        (third, fourth)
    } else {
        (fourth, third)
    };
    first_left < second_right && second_left < first_right
}
