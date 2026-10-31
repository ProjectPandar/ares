// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:1224–1340. The loop body is extracted without moving
    // allocation, contribution, intersection or scanline work across minima.
    pub(crate) fn insert_local_minima_into_ael(&mut self, bot_y: i64) {
        while let Some(minimum) = self.pop_local_minima(bot_y) {
            self.insert_local_minimum(minimum);
        }
    }

    fn insert_local_minimum(&mut self, minimum: usize) {
        let vertex = self.minima_list[minimum].vertex;
        let flags = self.vertex_arena[vertex].flags;
        let point = self.vertex_arena[vertex].pt;
        let mut left = if (flags & VertexFlags::OPEN_START) != VertexFlags::EMPTY {
            None
        } else {
            let index = self.active_arena.len();
            let mut edge = Active::new();
            edge.bot = point;
            edge.curr_x = point.x;
            edge.wind_dx = -1;
            edge.vertex_top = self.vertex_arena[vertex].prev;
            edge.top = self.vertex_arena[edge.vertex_top].pt;
            edge.local_min = minimum;
            set_dx(&mut edge);
            self.active_arena.push(edge);
            Some(index)
        };
        let mut right = if (flags & VertexFlags::OPEN_END) != VertexFlags::EMPTY {
            None
        } else {
            let index = self.active_arena.len();
            let mut edge = Active::new();
            edge.bot = point;
            edge.curr_x = point.x;
            edge.wind_dx = 1;
            edge.vertex_top = self.vertex_arena[vertex].next;
            edge.top = self.vertex_arena[edge.vertex_top].pt;
            edge.local_min = minimum;
            set_dx(&mut edge);
            self.active_arena.push(edge);
            Some(index)
        };
        if let (Some(left_index), Some(right_index)) = (left, right) {
            if self.minimum_bounds_need_swap(left_index, right_index) {
                std::mem::swap(&mut left, &mut right);
            }
        } else if left.is_none() {
            left = right;
            right = None;
        }

        let left = left.unwrap();
        self.active_arena[left].is_left_bound = true;
        self.insert_left_edge(left);
        let contributing = if self.is_open_idx(left) {
            self.set_wind_count_for_open_path_edge(left);
            self.is_contributing_open(left)
        } else {
            self.set_wind_count_for_closed_path_edge(left);
            self.is_contributing_closed(left)
        };
        if let Some(right) = right {
            self.insert_minimum_right_bound(left, right, contributing);
        } else if contributing {
            self.start_open_path(left, self.active_arena[left].bot);
        }
        if self.is_horizontal_idx(left) {
            self.push_horz(left);
        } else {
            self.insert_scanline(self.active_arena[left].top.y);
        }
    }

    // InsertLocalMinimaIntoAEL:1268–1283, short-circuit in the source order.
    fn minimum_bounds_need_swap(&self, left: usize, right: usize) -> bool {
        if self.is_horizontal_idx(left) {
            return is_heading_right_horz(&self.active_arena[left]);
        }
        if self.is_horizontal_idx(right) {
            return is_heading_left_horz(&self.active_arena[right]);
        }
        self.active_arena[left].dx < self.active_arena[right].dx
    }

    // InsertLocalMinimaIntoAEL:1302–1333, right bound remains immediately
    // after left contribution and before left horizontal/scanline admission.
    fn insert_minimum_right_bound(&mut self, left: usize, right: usize, contributing: bool) {
        self.active_arena[right].is_left_bound = false;
        self.active_arena[right].wind_cnt = self.active_arena[left].wind_cnt;
        self.active_arena[right].wind_cnt2 = self.active_arena[left].wind_cnt2;
        self.insert_right_edge(left, right);
        if contributing {
            self.add_local_min_poly(left, right, self.active_arena[left].bot, true);
            if !self.is_horizontal_idx(left) {
                self.check_join_left(left, self.active_arena[left].bot, false);
            }
        }
        while let Some(next) = self.active_arena[right].next_in_ael {
            if !is_valid_ael_order(
                &self.active_arena[next],
                &self.active_arena[right],
                &self.vertex_arena,
                &self.minima_list,
            ) {
                break;
            }
            self.intersect_edges(right, next, self.active_arena[right].bot);
            self.swap_positions_in_ael(right, next);
        }
        if self.is_horizontal_idx(right) {
            self.push_horz(right);
        } else {
            self.check_join_right(right, self.active_arena[right].bot, false);
            self.insert_scanline(self.active_arena[right].top.y);
        }
    }

    // clipper.engine.cpp:ResetHorzDirection. Nonvertical cases finish before
    // the vertical maxima search; direction and visit order are unchanged.
    pub(crate) fn reset_horz_direction(
        &self,
        index: usize,
        maximum: Option<usize>,
    ) -> (i64, i64, bool) {
        let horizontal = &self.active_arena[index];
        if horizontal.bot.x != horizontal.top.x {
            return if horizontal.curr_x < horizontal.top.x {
                (horizontal.curr_x, horizontal.top.x, true)
            } else {
                (horizontal.top.x, horizontal.curr_x, false)
            };
        }
        let x = horizontal.curr_x;
        let mut current = horizontal.next_in_ael;
        while let Some(edge) = current {
            if Some(self.active_arena[edge].vertex_top) == maximum {
                return (x, x, true);
            }
            current = self.active_arena[edge].next_in_ael;
        }
        (x, x, false)
    }
}
