// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Check join left
    /// Direct port from clipper.engine.cpp CheckJoinLeft (line 2812)
    pub(crate) fn check_join_left(&mut self, e_idx: usize, pt: Point64, check_curr_x: bool) {
        let prev = self.active_arena[e_idx].prev_in_ael;
        let prev_idx = match prev {
            Some(p) => p,
            None => return,
        };

        if !is_hot_edge(&self.active_arena[e_idx])
            || !is_hot_edge(&self.active_arena[prev_idx])
            || self.is_horizontal_idx(e_idx)
            || self.is_horizontal_idx(prev_idx)
            || self.is_open_idx(e_idx)
            || self.is_open_idx(prev_idx)
        {
            return;
        }

        let e_top = self.active_arena[e_idx].top;
        let p_top = self.active_arena[prev_idx].top;
        if (pt.y < e_top.y + 2 || pt.y < p_top.y + 2)
            && (self.active_arena[e_idx].bot.y > pt.y || self.active_arena[prev_idx].bot.y > pt.y)
        {
            return;
        }

        if check_curr_x {
            if perpendic_dist_from_line_sqrd(
                pt,
                self.active_arena[prev_idx].bot,
                self.active_arena[prev_idx].top,
            ) > 0.25
            {
                return;
            }
        } else if self.active_arena[e_idx].curr_x != self.active_arena[prev_idx].curr_x {
            return;
        }

        if !is_collinear(e_top, pt, p_top) {
            return;
        }

        let e_or = self.active_arena[e_idx].outrec.unwrap();
        let p_or = self.active_arena[prev_idx].outrec.unwrap();

        if self.outrec_list[e_or].idx == self.outrec_list[p_or].idx {
            self.add_local_max_poly(prev_idx, e_idx, pt);
        } else if self.outrec_list[e_or].idx < self.outrec_list[p_or].idx {
            self.join_outrec_paths(e_idx, prev_idx);
        } else {
            self.join_outrec_paths(prev_idx, e_idx);
        }
        self.active_arena[prev_idx].join_with = JoinWith::Right;
        self.active_arena[e_idx].join_with = JoinWith::Left;
    }
}

impl ClipperBase {
    /// Check join right
    /// Direct port from clipper.engine.cpp CheckJoinRight (line 2840)
    pub(crate) fn check_join_right(&mut self, e_idx: usize, pt: Point64, check_curr_x: bool) {
        let next = self.active_arena[e_idx].next_in_ael;
        let next_idx = match next {
            Some(n) => n,
            None => return,
        };

        if !is_hot_edge(&self.active_arena[e_idx])
            || !is_hot_edge(&self.active_arena[next_idx])
            || self.is_horizontal_idx(e_idx)
            || self.is_horizontal_idx(next_idx)
            || self.is_open_idx(e_idx)
            || self.is_open_idx(next_idx)
        {
            return;
        }

        let e_top = self.active_arena[e_idx].top;
        let n_top = self.active_arena[next_idx].top;
        if (pt.y < e_top.y + 2 || pt.y < n_top.y + 2)
            && (self.active_arena[e_idx].bot.y > pt.y || self.active_arena[next_idx].bot.y > pt.y)
        {
            return;
        }

        if check_curr_x {
            if perpendic_dist_from_line_sqrd(
                pt,
                self.active_arena[next_idx].bot,
                self.active_arena[next_idx].top,
            ) > 0.35
            {
                return;
            }
        } else if self.active_arena[e_idx].curr_x != self.active_arena[next_idx].curr_x {
            return;
        }

        if !is_collinear(e_top, pt, n_top) {
            return;
        }

        let e_or = self.active_arena[e_idx].outrec.unwrap();
        let n_or = self.active_arena[next_idx].outrec.unwrap();

        if self.outrec_list[e_or].idx == self.outrec_list[n_or].idx {
            self.add_local_max_poly(e_idx, next_idx, pt);
        } else if self.outrec_list[e_or].idx < self.outrec_list[n_or].idx {
            self.join_outrec_paths(e_idx, next_idx);
        } else {
            self.join_outrec_paths(next_idx, e_idx);
        }

        self.active_arena[e_idx].join_with = JoinWith::Right;
        self.active_arena[next_idx].join_with = JoinWith::Left;
    }
}
