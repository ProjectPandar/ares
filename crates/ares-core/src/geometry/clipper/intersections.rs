use super::ordering::fixed_msvc_sort_by;
use super::predicates::{intersect_point, slopes_equal_four, top_x};
use super::types::{Edge, EdgeId, ExecutionConfig, IntersectionNode, Join, OutputIndex};
use super::{ClipOperation, ClosedClipper, FillRule, PathRole};
use crate::geometry::Point;

impl ClosedClipper {
    pub(super) fn intersect_edges(
        &mut self,
        first: EdgeId,
        second: EdgeId,
        point: Point,
        config: ExecutionConfig,
    ) {
        let first_was_output = matches!(self.edges.edge(first).output, OutputIndex::Assigned(_));
        let second_was_output = matches!(self.edges.edge(second).output, OutputIndex::Assigned(_));
        let first_before = *self.edges.edge(first);
        let second_before = *self.edges.edge(second);
        if first_before.role == second_before.role {
            if own_fill(first_before, config) == FillRule::EvenOdd {
                self.edges.edge_mut(first).wind_count = second_before.wind_count;
                self.edges.edge_mut(second).wind_count = first_before.wind_count;
            } else {
                self.edges.edge_mut(first).wind_count =
                    adjusted_wind(first_before.wind_count, second_before.wind_delta, false);
                self.edges.edge_mut(second).wind_count =
                    adjusted_wind(second_before.wind_count, first_before.wind_delta, true);
            }
        } else {
            self.edges.edge_mut(first).alternate_wind_count =
                if own_fill(second_before, config) == FillRule::EvenOdd {
                    toggle(first_before.alternate_wind_count)
                } else {
                    first_before.alternate_wind_count + second_before.wind_delta
                };
            self.edges.edge_mut(second).alternate_wind_count =
                if own_fill(first_before, config) == FillRule::EvenOdd {
                    toggle(second_before.alternate_wind_count)
                } else {
                    second_before.alternate_wind_count - first_before.wind_delta
                };
        }

        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        let first_wind = normalized_wind(own_fill(first_edge, config), first_edge.wind_count);
        let second_wind = normalized_wind(own_fill(second_edge, config), second_edge.wind_count);
        if first_was_output && second_was_output {
            if !unit_or_zero(first_wind)
                || !unit_or_zero(second_wind)
                || (first_edge.role != second_edge.role && config.operation != ClipOperation::Xor)
            {
                self.add_local_max_polygon(first, second, point);
            } else {
                self.add_out_point(first, point);
                self.add_out_point(second, point);
                self.swap_edge_output(first, second);
            }
        } else if first_was_output {
            if unit_or_zero(second_wind) {
                self.add_out_point(first, point);
                self.swap_edge_output(first, second);
            }
        } else if second_was_output {
            if unit_or_zero(first_wind) {
                self.add_out_point(second, point);
                self.swap_edge_output(first, second);
            }
        } else if unit_or_zero(first_wind) && unit_or_zero(second_wind) {
            let first_alt = normalized_wind(
                alternate_fill(first_edge, config),
                first_edge.alternate_wind_count,
            );
            let second_alt = normalized_wind(
                alternate_fill(second_edge, config),
                second_edge.alternate_wind_count,
            );
            let create_minimum = if first_edge.role != second_edge.role {
                true
            } else if first_wind == 1 && second_wind == 1 {
                operation_creates_minimum(config.operation, first_edge.role, first_alt, second_alt)
            } else {
                self.swap_edge_sides(first, second);
                false
            };
            if create_minimum {
                self.add_local_min_polygon(first, second, point);
            }
        }
    }

    pub(super) fn process_intersections(&mut self, top_y: i64, config: ExecutionConfig) -> bool {
        if self.active_edges.is_none() {
            return true;
        }
        self.build_intersection_list(top_y);
        if self.intersections.len() > 1 && !self.fix_intersection_order() {
            return false;
        }
        for index in 0..self.intersections.len() {
            let node = self.intersections[index];
            self.intersect_edges(node.first, node.second, node.point, config);
            self.swap_positions_in_ael(node.first, node.second);
        }
        self.intersections.clear();
        self.sorted_edges = None;
        true
    }

    pub(super) fn process_edges_at_top(&mut self, top_y: i64, config: ExecutionConfig) {
        let mut edge = self.active_edges;
        while let Some(current) = edge {
            let snapshot = *self.edges.edge(current);
            edge = self.process_top_edge(current, snapshot, top_y, config);
        }

        self.process_horizontals(config);
        let mut edge = self.active_edges;
        while let Some(current) = edge {
            let snapshot = *self.edges.edge(current);
            if snapshot.top.y() == top_y && snapshot.next_in_lml.is_some() {
                let output = matches!(snapshot.output, OutputIndex::Assigned(_))
                    .then(|| self.add_out_point(current, snapshot.top));
                let promoted = self.update_edge_into_ael(current);
                self.join_promoted_edge(promoted, output);
                edge = self.edges.edge(promoted).next_in_ael;
            } else {
                edge = snapshot.next_in_ael;
            }
        }
    }

    fn process_top_edge(
        &mut self,
        current: EdgeId,
        snapshot: Edge,
        top_y: i64,
        config: ExecutionConfig,
    ) -> Option<EdgeId> {
        let maxima = snapshot.top.y() == top_y
            && snapshot.next_in_lml.is_none()
            && self
                .maxima_pair_ex(current)
                .is_none_or(|pair| !self.edges.edge(pair).is_horizontal());
        if maxima {
            let previous = snapshot.previous_in_ael;
            self.do_maxima(current, config);
            return previous
                .and_then(|id| self.edges.edge(id).next_in_ael)
                .or(self.active_edges.filter(|_| previous.is_none()));
        }
        let horizontal_promotion = snapshot
            .next_in_lml
            .filter(|&next| snapshot.top.y() == top_y && self.edges.edge(next).is_horizontal());
        if horizontal_promotion.is_some() {
            let promoted = self.update_edge_into_ael(current);
            if matches!(self.edges.edge(promoted).output, OutputIndex::Assigned(_)) {
                self.add_out_point(promoted, self.edges.edge(promoted).bottom);
            }
            self.add_edge_to_sel(promoted);
            return self.edges.edge(promoted).next_in_ael;
        }
        let x = top_x(snapshot, top_y);
        self.edges.edge_mut(current).current = Point::new(x, top_y);
        snapshot.next_in_ael
    }

    fn build_intersection_list(&mut self, top_y: i64) {
        self.intersections.clear();
        self.copy_ael_to_sel();
        let mut edge = self.sorted_edges;
        while let Some(id) = edge {
            let snapshot = *self.edges.edge(id);
            self.edges.edge_mut(id).current =
                Point::new(top_x(snapshot, top_y), snapshot.current.y());
            edge = snapshot.next_in_ael;
        }
        while let Some((modified, edge)) = self.sort_intersection_pass(top_y) {
            if let Some(previous) = self.edges.edge(edge).previous_in_sel {
                self.edges.edge_mut(previous).next_in_sel = None;
            } else {
                break;
            }
            if !modified {
                break;
            }
        }
        self.sorted_edges = None;
    }

    fn sort_intersection_pass(&mut self, top_y: i64) -> Option<(bool, EdgeId)> {
        let mut edge = self.sorted_edges?;
        let mut modified = false;
        while let Some(next) = self.edges.edge(edge).next_in_sel {
            if self.edges.edge(edge).current.x() <= self.edges.edge(next).current.x() {
                edge = next;
                continue;
            }
            let mut point = intersect_point(*self.edges.edge(edge), *self.edges.edge(next));
            if point.y() < top_y {
                point = Point::new(top_x(*self.edges.edge(edge), top_y), top_y);
            }
            self.intersections.push(IntersectionNode {
                first: edge,
                second: next,
                point,
            });
            self.swap_positions_in_sel(edge, next);
            modified = true;
        }
        Some((modified, edge))
    }

    fn fix_intersection_order(&mut self) -> bool {
        self.copy_ael_to_sel();
        fixed_msvc_sort_by(&mut self.intersections, |first, second| {
            second.point.y() < first.point.y()
        });
        for index in 0..self.intersections.len() {
            if !self.order_intersection(index) {
                return false;
            }
        }
        true
    }

    fn order_intersection(&mut self, index: usize) -> bool {
        if !self.intersection_edges_adjacent(self.intersections[index]) {
            let next = (index + 1..self.intersections.len())
                .find(|&candidate| self.intersection_edges_adjacent(self.intersections[candidate]));
            let Some(next) = next else { return false };
            self.intersections.swap(index, next);
        }
        let node = self.intersections[index];
        self.swap_positions_in_sel(node.first, node.second);
        true
    }

    fn intersection_edges_adjacent(&self, node: IntersectionNode) -> bool {
        self.edges.edge(node.first).next_in_sel == Some(node.second)
            || self.edges.edge(node.first).previous_in_sel == Some(node.second)
    }

    fn maxima_pair_ex(&self, edge: EdgeId) -> Option<EdgeId> {
        let edge_state = *self.edges.edge(edge);
        let next = self.edges.edge(edge_state.next);
        let pair = if next.top == edge_state.top && next.next_in_lml.is_none() {
            Some(edge_state.next)
        } else {
            let previous = self.edges.edge(edge_state.previous);
            (previous.top == edge_state.top && previous.next_in_lml.is_none())
                .then_some(edge_state.previous)
        }?;
        let pair_edge = self.edges.edge(pair);
        if pair_edge.output == OutputIndex::Skipped
            || pair_edge.next_in_ael == pair_edge.previous_in_ael && !pair_edge.is_horizontal()
        {
            None
        } else {
            Some(pair)
        }
    }

    fn do_maxima(&mut self, edge: EdgeId, config: ExecutionConfig) {
        let point = self.edges.edge(edge).top;
        let Some(pair) = self.maxima_pair_ex(edge) else {
            if matches!(self.edges.edge(edge).output, OutputIndex::Assigned(_)) {
                self.add_out_point(edge, point);
            }
            self.delete_from_ael(edge);
            return;
        };
        while let Some(next) = self.edges.edge(edge).next_in_ael {
            if next == pair {
                break;
            }
            self.intersect_edges(edge, next, point, config);
            self.swap_positions_in_ael(edge, next);
        }
        let first_output = self.edges.edge(edge).output;
        let second_output = self.edges.edge(pair).output;
        if first_output == OutputIndex::Unassigned && second_output == OutputIndex::Unassigned {
            self.delete_from_ael(edge);
            self.delete_from_ael(pair);
        } else if matches!(first_output, OutputIndex::Assigned(_))
            && matches!(second_output, OutputIndex::Assigned(_))
        {
            self.add_local_max_polygon(edge, pair, point);
            self.delete_from_ael(edge);
            self.delete_from_ael(pair);
        } else {
            unreachable!("closed maxima output state is paired");
        }
    }

    fn join_promoted_edge(&mut self, edge: EdgeId, output: Option<super::types::OutPointId>) {
        let Some(output) = output else { return };
        let snapshot = *self.edges.edge(edge);
        for neighbour in [snapshot.previous_in_ael, snapshot.next_in_ael]
            .into_iter()
            .flatten()
        {
            let other = *self.edges.edge(neighbour);
            if other.current == snapshot.bottom
                && matches!(other.output, OutputIndex::Assigned(_))
                && other.current.y() > other.top.y()
                && slopes_equal_four(
                    snapshot.current,
                    snapshot.top,
                    other.current,
                    other.top,
                    self.use_full_range,
                )
                && snapshot.wind_delta != 0
                && other.wind_delta != 0
            {
                let second = self.add_out_point(neighbour, snapshot.bottom);
                self.joins.push(Join {
                    first: output,
                    second,
                    offset: snapshot.top,
                });
                break;
            }
        }
    }

    fn swap_edge_output(&mut self, first: EdgeId, second: EdgeId) {
        let first_edge = *self.edges.edge(first);
        let second_edge = *self.edges.edge(second);
        self.edges.edge_mut(first).side = second_edge.side;
        self.edges.edge_mut(second).side = first_edge.side;
        self.edges.edge_mut(first).output = second_edge.output;
        self.edges.edge_mut(second).output = first_edge.output;
    }

    fn swap_edge_sides(&mut self, first: EdgeId, second: EdgeId) {
        let first_side = self.edges.edge(first).side;
        let second_side = self.edges.edge(second).side;
        self.edges.edge_mut(first).side = second_side;
        self.edges.edge_mut(second).side = first_side;
    }
}

fn adjusted_wind(count: i32, delta: i32, subtract: bool) -> i32 {
    let delta = if subtract { -delta } else { delta };
    if count + delta == 0 {
        -count
    } else {
        count + delta
    }
}

fn toggle(count: i32) -> i32 {
    if count == 0 { 1 } else { 0 }
}
fn unit_or_zero(count: i32) -> bool {
    count == 0 || count == 1
}
fn normalized_wind(fill: FillRule, count: i32) -> i32 {
    match fill {
        FillRule::Positive => count,
        FillRule::Negative => -count,
        _ => count.abs(),
    }
}
fn own_fill(edge: Edge, config: ExecutionConfig) -> FillRule {
    match edge.role {
        PathRole::Subject => config.subject_fill,
        PathRole::Clip => config.clip_fill,
    }
}
fn alternate_fill(edge: Edge, config: ExecutionConfig) -> FillRule {
    match edge.role {
        PathRole::Subject => config.clip_fill,
        PathRole::Clip => config.subject_fill,
    }
}

fn difference_creates_minimum(role: PathRole, first: i32, second: i32) -> bool {
    match role {
        PathRole::Clip => first > 0 && second > 0,
        PathRole::Subject => first <= 0 && second <= 0,
    }
}

fn operation_creates_minimum(
    operation: ClipOperation,
    role: PathRole,
    first: i32,
    second: i32,
) -> bool {
    match operation {
        ClipOperation::Intersection => first > 0 && second > 0,
        ClipOperation::Union => first <= 0 && second <= 0,
        ClipOperation::Difference => difference_creates_minimum(role, first, second),
        ClipOperation::Xor => true,
    }
}
