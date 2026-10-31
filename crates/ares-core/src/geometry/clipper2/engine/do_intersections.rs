// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    /// `clipper.engine.cpp:2373–2380`.
    pub(crate) fn do_intersections(&mut self, top_y: i64) {
        if self.build_intersect_list(top_y) {
            self.process_intersect_list();
            self.intersect_nodes.clear();
        }
    }

    // clipper.engine.cpp:2474–2499, same sort and event sequence.
    pub(crate) fn process_intersect_list(&mut self) {
        crate::geometry::fixed_gcc_sort_by(&mut self.intersect_nodes, |a, b| {
            intersect_list_sort(a, b).is_lt()
        });
        let len = self.intersect_nodes.len();
        for i in 0..len {
            self.make_intersection_adjacent(i);
            let e1 = self.intersect_nodes[i].edge1;
            let e2 = self.intersect_nodes[i].edge2;
            let point = self.intersect_nodes[i].pt;
            self.intersect_edges(e1, e2, point);
            self.swap_positions_in_ael(e1, e2);
            self.active_arena[e1].curr_x = point.x;
            self.active_arena[e2].curr_x = point.x;
            self.check_join_left(e2, point, true);
            self.check_join_right(e1, point, true);
        }
    }
    fn make_intersection_adjacent(&mut self, index: usize) {
        if edges_adjacent_in_ael(&self.intersect_nodes[index], &self.active_arena) {
            return;
        }
        let mut next = index + 1;
        while !edges_adjacent_in_ael(&self.intersect_nodes[next], &self.active_arena) {
            next += 1;
        }
        self.intersect_nodes.swap(index, next);
    }

    /// `clipper.engine.cpp:2734–2768`: keep the live next edge after each update.
    pub(crate) fn do_top_of_scanbeam(&mut self, y: i64) {
        self.sel = None;
        let mut current = self.actives;
        while let Some(edge) = current {
            current = self.advance_scanbeam_edge(edge, y);
        }
    }
    fn advance_scanbeam_edge(&mut self, edge: usize, y: i64) -> Option<usize> {
        if self.active_arena[edge].top.y != y {
            self.active_arena[edge].curr_x = top_x(&self.active_arena[edge], y);
            return self.active_arena[edge].next_in_ael;
        }
        self.active_arena[edge].curr_x = self.active_arena[edge].top.x;
        if self.is_maxima_idx(edge) {
            return self.do_maxima(edge);
        }
        if is_hot_edge(&self.active_arena[edge]) {
            self.add_out_pt(edge, self.active_arena[edge].top);
        }
        self.update_edge_into_ael(edge);
        if self.is_horizontal_idx(edge) {
            self.push_horz(edge);
        }
        self.active_arena[edge].next_in_ael
    }

    // clipper.engine.cpp:DoMaxima. Preserve the different open/closed pair
    // deletion order; do not silently terminate the intervening-edge walk.
    pub(crate) fn do_maxima(&mut self, edge: usize) -> Option<usize> {
        let previous = self.active_arena[edge].prev_in_ael;
        let mut next = self.active_arena[edge].next_in_ael;
        if self.is_open_end_idx(edge) {
            if is_hot_edge(&self.active_arena[edge]) {
                self.add_out_pt(edge, self.active_arena[edge].top);
            }
            if !self.is_horizontal_idx(edge) {
                self.finish_open_maximum(edge);
            }
            return next;
        }
        let Some(pair) = get_maxima_pair(&self.active_arena[edge], &self.active_arena) else {
            return next;
        };
        if is_joined(&self.active_arena[edge]) {
            self.split(edge, self.active_arena[edge].top);
        }
        if is_joined(&self.active_arena[pair]) {
            self.split(pair, self.active_arena[pair].top);
        }
        while next != Some(pair) {
            let next_edge = next.unwrap();
            self.intersect_edges(edge, next_edge, self.active_arena[edge].top);
            self.swap_positions_in_ael(edge, next_edge);
            next = self.active_arena[edge].next_in_ael;
        }
        if self.is_open_idx(edge) {
            if is_hot_edge(&self.active_arena[edge]) {
                self.add_local_max_poly(edge, pair, self.active_arena[edge].top);
            }
            self.delete_from_ael(pair);
            self.delete_from_ael(edge);
            return if let Some(previous) = previous {
                self.active_arena[previous].next_in_ael
            } else {
                self.actives
            };
        }
        if is_hot_edge(&self.active_arena[edge]) {
            self.add_local_max_poly(edge, pair, self.active_arena[edge].top);
        }
        self.delete_from_ael(edge);
        self.delete_from_ael(pair);
        if let Some(previous) = previous {
            self.active_arena[previous].next_in_ael
        } else {
            self.actives
        }
    }
    fn finish_open_maximum(&mut self, edge: usize) {
        if is_hot_edge(&self.active_arena[edge]) {
            let record = self.active_arena[edge].outrec.unwrap();
            if self.is_front(edge) {
                self.outrec_list[record].front_edge = None;
            } else {
                self.outrec_list[record].back_edge = None;
            }
            self.active_arena[edge].outrec = None;
        }
        self.delete_from_ael(edge);
    }
}
