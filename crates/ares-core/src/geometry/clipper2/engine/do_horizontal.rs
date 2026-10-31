// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:2563–2732, consecutive horizontal bounds.
    pub(crate) fn do_horizontal(&mut self, horizontal: usize) {
        let open = self.is_open_idx(horizontal);
        let y = self.active_arena[horizontal].bot.y;
        let maximum = if open {
            get_curr_y_maxima_vertex_open(&self.active_arena[horizontal], &self.vertex_arena)
        } else {
            get_curr_y_maxima_vertex(&self.active_arena[horizontal], &self.vertex_arena)
        };
        let mut direction = self.reset_horz_direction(horizontal, maximum);
        if is_hot_edge(&self.active_arena[horizontal]) {
            let point = Point64::new_z(
                self.active_arena[horizontal].curr_x,
                y,
                self.active_arena[horizontal].bot.z,
            );
            let output = self.add_out_pt(horizontal, point);
            self.add_trial_horz_join(output);
        }
        loop {
            if self.sweep_horizontal(horizontal, maximum, direction) {
                return;
            }
            if open && self.is_open_end_idx(horizontal) {
                self.finish_open_horizontal(horizontal);
                return;
            }
            let next = self.next_vertex_idx(horizontal);
            if self.vertex_arena[next].pt.y != self.active_arena[horizontal].top.y {
                break;
            }
            if is_hot_edge(&self.active_arena[horizontal]) {
                self.add_out_pt(horizontal, self.active_arena[horizontal].top);
            }
            self.update_edge_into_ael(horizontal);
            direction = self.reset_horz_direction(horizontal, maximum);
        }
        if is_hot_edge(&self.active_arena[horizontal]) {
            let point = self.add_out_pt(horizontal, self.active_arena[horizontal].top);
            self.add_trial_horz_join(point);
        }
        self.update_edge_into_ael(horizontal);
    }

    // DoHorizontal:2606–2688, the neighbor walk. True means both maxima edges
    // were deleted; false means the bound reached its normal stopping point.
    fn sweep_horizontal(
        &mut self,
        horizontal: usize,
        maximum: Option<usize>,
        direction: (i64, i64, bool),
    ) -> bool {
        let mut current = if direction.2 {
            self.active_arena[horizontal].next_in_ael
        } else {
            self.active_arena[horizontal].prev_in_ael
        };
        while let Some(edge) = current {
            if Some(self.active_arena[edge].vertex_top) == maximum {
                self.finish_horizontal_maximum(horizontal, edge, (maximum, direction.2));
                return true;
            }
            if self.horizontal_reaches_end(horizontal, edge, maximum, direction) {
                break;
            }
            current = self.cross_horizontal_neighbor(horizontal, edge, direction.2);
        }
        false
    }
    fn horizontal_reaches_end(
        &self,
        horizontal: usize,
        edge: usize,
        maximum: Option<usize>,
        (left, right, forward): (i64, i64, bool),
    ) -> bool {
        if maximum == Some(self.active_arena[horizontal].vertex_top)
            && !self.is_open_end_idx(horizontal)
        {
            return false;
        }
        if (forward && self.active_arena[edge].curr_x > right)
            || (!forward && self.active_arena[edge].curr_x < left)
        {
            return true;
        }
        if self.active_arena[edge].curr_x != self.active_arena[horizontal].top.x
            || self.is_horizontal_idx(edge)
        {
            return false;
        }
        let point = self.vertex_arena[self.next_vertex_idx(horizontal)].pt;
        if forward {
            if self.is_open_idx(edge)
                && !self.is_same_poly_type_idx(edge, horizontal)
                && !is_hot_edge(&self.active_arena[edge])
            {
                return top_x(&self.active_arena[edge], point.y) > point.x;
            }
            top_x(&self.active_arena[edge], point.y) >= point.x
        } else {
            if self.is_open_idx(edge)
                && !self.is_same_poly_type_idx(edge, horizontal)
                && !is_hot_edge(&self.active_arena[edge])
            {
                return top_x(&self.active_arena[edge], point.y) < point.x;
            }
            top_x(&self.active_arena[edge], point.y) <= point.x
        }
    }
    fn cross_horizontal_neighbor(
        &mut self,
        horizontal: usize,
        edge: usize,
        forward: bool,
    ) -> Option<usize> {
        let point = Point64::new(
            self.active_arena[edge].curr_x,
            self.active_arena[horizontal].bot.y,
        );
        let next = if forward {
            self.intersect_edges(horizontal, edge, point);
            self.swap_positions_in_ael(horizontal, edge);
            self.check_join_left(edge, point, false);
            self.active_arena[horizontal].curr_x = self.active_arena[edge].curr_x;
            self.active_arena[horizontal].next_in_ael
        } else {
            self.intersect_edges(edge, horizontal, point);
            self.swap_positions_in_ael(edge, horizontal);
            self.check_join_right(edge, point, false);
            self.active_arena[horizontal].curr_x = self.active_arena[edge].curr_x;
            self.active_arena[horizontal].prev_in_ael
        };
        if self.active_arena[horizontal].outrec.is_some() {
            let point = get_last_op(
                horizontal,
                &self.active_arena,
                &self.outrec_list,
                &self.outpt_arena,
            )
            .unwrap();
            self.add_trial_horz_join(point);
        }
        next
    }
    fn finish_horizontal_maximum(
        &mut self,
        horizontal: usize,
        edge: usize,
        (maximum, forward): (Option<usize>, bool),
    ) {
        if is_hot_edge(&self.active_arena[horizontal]) && is_joined(&self.active_arena[edge]) {
            self.split(edge, self.active_arena[edge].top);
        }
        if is_hot_edge(&self.active_arena[horizontal]) {
            while Some(self.active_arena[horizontal].vertex_top) != maximum {
                self.add_out_pt(horizontal, self.active_arena[horizontal].top);
                self.update_edge_into_ael(horizontal);
            }
            let point = self.active_arena[horizontal].top;
            if forward {
                self.add_local_max_poly(horizontal, edge, point);
            } else {
                self.add_local_max_poly(edge, horizontal, point);
            }
        }
        self.delete_from_ael(edge);
        self.delete_from_ael(horizontal);
    }
    fn finish_open_horizontal(&mut self, horizontal: usize) {
        if is_hot_edge(&self.active_arena[horizontal]) {
            self.add_out_pt(horizontal, self.active_arena[horizontal].top);
            let record = self.active_arena[horizontal].outrec.unwrap();
            if self.is_front(horizontal) {
                self.outrec_list[record].front_edge = None;
            } else {
                self.outrec_list[record].back_edge = None;
            }
            self.active_arena[horizontal].outrec = None;
        }
        self.delete_from_ael(horizontal);
    }
}
