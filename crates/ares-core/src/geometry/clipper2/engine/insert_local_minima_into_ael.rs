// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Insert local minima into AEL at a given y
    /// Direct port from clipper.engine.cpp InsertLocalMinimaIntoAEL (line 1198)
    pub(crate) fn insert_local_minima_into_ael(&mut self, bot_y: i64) {
        while let Some(loc_min_idx) = self.pop_local_minima(bot_y) {
            let vert_idx = self.minima_list[loc_min_idx].vertex;
            let vert_flags = self.vertex_arena[vert_idx].flags;
            let vert_pt = self.vertex_arena[vert_idx].pt;

            // Create left bound if not open start
            let left_bound_opt = if (vert_flags & VertexFlags::OPEN_START) != VertexFlags::EMPTY {
                None
            } else {
                let lb_idx = self.active_arena.len();
                let mut lb = Active::new();
                lb.bot = vert_pt;
                lb.curr_x = vert_pt.x;
                lb.wind_dx = -1;
                lb.vertex_top = self.vertex_arena[vert_idx].prev; // descending
                lb.top = self.vertex_arena[lb.vertex_top].pt;
                lb.local_min = loc_min_idx;
                set_dx(&mut lb);
                self.active_arena.push(lb);
                Some(lb_idx)
            };

            // Create right bound if not open end
            let right_bound_opt = if (vert_flags & VertexFlags::OPEN_END) != VertexFlags::EMPTY {
                None
            } else {
                let rb_idx = self.active_arena.len();
                let mut rb = Active::new();
                rb.bot = vert_pt;
                rb.curr_x = vert_pt.x;
                rb.wind_dx = 1;
                rb.vertex_top = self.vertex_arena[vert_idx].next; // ascending
                rb.top = self.vertex_arena[rb.vertex_top].pt;
                rb.local_min = loc_min_idx;
                set_dx(&mut rb);
                self.active_arena.push(rb);
                Some(rb_idx)
            };

            // Determine which is actually left and which is right
            let (mut left_bound, mut right_bound) = (left_bound_opt, right_bound_opt);

            if let (Some(lb), Some(rb)) = (left_bound, right_bound) {
                if self.is_horizontal_idx(lb) {
                    if is_heading_right_horz(&self.active_arena[lb]) {
                        std::mem::swap(&mut left_bound, &mut right_bound);
                    }
                } else if self.is_horizontal_idx(rb) {
                    if is_heading_left_horz(&self.active_arena[rb]) {
                        std::mem::swap(&mut left_bound, &mut right_bound);
                    }
                } else if self.active_arena[lb].dx < self.active_arena[rb].dx {
                    std::mem::swap(&mut left_bound, &mut right_bound);
                }
            } else if left_bound.is_none() {
                left_bound = right_bound;
                right_bound = None;
            }

            let lb_idx = left_bound.unwrap();
            self.active_arena[lb_idx].is_left_bound = true;
            self.insert_left_edge(lb_idx);

            let contributing = if self.is_open_idx(lb_idx) {
                self.set_wind_count_for_open_path_edge(lb_idx);
                self.is_contributing_open(lb_idx)
            } else {
                self.set_wind_count_for_closed_path_edge(lb_idx);
                self.is_contributing_closed(lb_idx)
            };

            if let Some(rb_idx) = right_bound {
                self.active_arena[rb_idx].is_left_bound = false;
                self.active_arena[rb_idx].wind_cnt = self.active_arena[lb_idx].wind_cnt;
                self.active_arena[rb_idx].wind_cnt2 = self.active_arena[lb_idx].wind_cnt2;
                self.insert_right_edge(lb_idx, rb_idx);

                if contributing {
                    let bot = self.active_arena[lb_idx].bot;
                    self.add_local_min_poly(lb_idx, rb_idx, bot, true);
                    if !self.is_horizontal_idx(lb_idx) {
                        let bot = self.active_arena[lb_idx].bot;
                        self.check_join_left(lb_idx, bot, false);
                    }
                }

                // Process any right-bound edge intersections
                while let Some(next) = self.active_arena[rb_idx].next_in_ael {
                    if !is_valid_ael_order(
                        &self.active_arena[next],
                        &self.active_arena[rb_idx],
                        &self.vertex_arena,
                        &self.minima_list,
                    ) {
                        break;
                    }
                    let bot = self.active_arena[rb_idx].bot;
                    self.intersect_edges(rb_idx, next, bot);
                    self.swap_positions_in_ael(rb_idx, next);
                }

                if self.is_horizontal_idx(rb_idx) {
                    self.push_horz(rb_idx);
                } else {
                    let bot = self.active_arena[rb_idx].bot;
                    self.check_join_right(rb_idx, bot, false);
                    let top_y = self.active_arena[rb_idx].top.y;
                    self.insert_scanline(top_y);
                }
            } else if contributing {
                let bot = self.active_arena[lb_idx].bot;
                self.start_open_path(lb_idx, bot);
            }

            if self.is_horizontal_idx(lb_idx) {
                self.push_horz(lb_idx);
            } else {
                let top_y = self.active_arena[lb_idx].top.y;
                self.insert_scanline(top_y);
            }
        }
    }
}

impl ClipperBase {
    /// Reset horizontal direction for a horizontal edge
    /// Direct port from clipper.engine.cpp ResetHorzDirection (line 2511)
    pub(crate) fn reset_horz_direction(
        &self,
        horz_idx: usize,
        max_vertex: Option<usize>,
    ) -> (i64, i64, bool) {
        let horz = &self.active_arena[horz_idx];
        if horz.bot.x == horz.top.x {
            let horz_left = horz.curr_x;
            let horz_right = horz.curr_x;
            let mut e = horz.next_in_ael;
            while let Some(e_idx) = e {
                if Some(self.active_arena[e_idx].vertex_top) == max_vertex {
                    return (horz_left, horz_right, true);
                }
                e = self.active_arena[e_idx].next_in_ael;
            }
            (horz_left, horz_right, false)
        } else if horz.curr_x < horz.top.x {
            (horz.curr_x, horz.top.x, true)
        } else {
            (horz.top.x, horz.curr_x, false)
        }
    }
}
