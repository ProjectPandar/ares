mod open;
mod top;

#[cfg(test)]
pub(in crate::geometry) use top::top_updates_for_test;

use super::ordering::fixed_gcc_sort_by;
use super::predicates::{intersect_point, top_x};
use super::types::{Edge, EdgeId, ExecutionConfig, IntersectionNode, OutputIndex};
use super::z::KernelPoint;
use super::{ClipOperation, Clipper, FillRule, PathRole};

impl Clipper {
    pub(super) fn intersect_edges(
        &mut self,
        first: EdgeId,
        second: EdgeId,
        mut point: KernelPoint,
        config: ExecutionConfig,
    ) {
        if self.z_intersections.is_some() {
            let first_before_z = *self.edges.edge(first);
            let second_before_z = *self.edges.edge(second);
            self.set_z(&mut point, first_before_z, second_before_z);
        }
        let first_was_output = matches!(self.edges.edge(first).output, OutputIndex::Assigned(_));
        let second_was_output = matches!(self.edges.edge(second).output, OutputIndex::Assigned(_));
        let first_before = *self.edges.edge(first);
        let second_before = *self.edges.edge(second);
        if self.intersect_open_edges(
            [first, second],
            point,
            config,
            [first_was_output, second_was_output],
        ) {
            return;
        }
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

    fn build_intersection_list(&mut self, top_y: i64) {
        self.intersections.clear();
        self.copy_ael_to_sel();
        let mut edge = self.sorted_edges;
        while let Some(id) = edge {
            let snapshot = *self.edges.edge(id);
            self.edges.edge_mut(id).current = snapshot.current.with_x(top_x(snapshot, top_y));
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
                point = KernelPoint::new(top_x(*self.edges.edge(edge), top_y), top_y, 0);
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
        fixed_gcc_sort_by(&mut self.intersections, |first, second| {
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
