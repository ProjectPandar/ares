// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Add a local maximum polygon (close a polygon)
    /// Direct port from clipper.engine.cpp AddLocalMaxPoly (line 1380)
    pub(crate) fn add_local_max_poly(
        &mut self,
        e1_idx: usize,
        e2_idx: usize,
        pt: Point64,
    ) -> Option<usize> {
        if is_joined(&self.active_arena[e1_idx]) {
            self.split(e1_idx, pt);
        }
        if is_joined(&self.active_arena[e2_idx]) {
            self.split(e2_idx, pt);
        }

        if self.is_front(e1_idx) == self.is_front(e2_idx) {
            if self.is_open_end_idx(e1_idx) {
                let or_idx = self.active_arena[e1_idx].outrec.unwrap();
                swap_front_back_sides(or_idx, &mut self.outrec_list, &self.outpt_arena);
            } else if self.is_open_end_idx(e2_idx) {
                let or_idx = self.active_arena[e2_idx].outrec.unwrap();
                swap_front_back_sides(or_idx, &mut self.outrec_list, &self.outpt_arena);
            } else {
                self.succeeded = false;
                return None;
            }
        }

        let result = self.add_out_pt(e1_idx, pt);
        let or1 = self.active_arena[e1_idx].outrec;
        let or2 = self.active_arena[e2_idx].outrec;

        if or1 == or2 {
            let or_idx = or1.unwrap();
            self.outrec_list[or_idx].pts = Some(result);

            if self.using_polytree {
                let prev_hot = self.get_prev_hot_edge(e1_idx);
                if prev_hot.is_none() {
                    self.outrec_list[or_idx].owner = None;
                } else if let Some(prev_hot_idx) = prev_hot {
                    if let Some(prev_or) = self.active_arena[prev_hot_idx].outrec {
                        set_owner(&mut self.outrec_list, or_idx, prev_or);
                    }
                }
            }

            uncouple_outrec(e1_idx, &mut self.active_arena, &mut self.outrec_list);
            let final_result = self.outrec_list[or_idx].pts;
            if let Some(owner) = self.outrec_list[or_idx].owner {
                if self.outrec_list[owner].front_edge.is_none() {
                    self.outrec_list[or_idx].owner = get_real_outrec(&self.outrec_list, owner);
                }
            }
            return final_result;
        } else if self.is_open_idx(e1_idx) {
            if self.active_arena[e1_idx].wind_dx < 0 {
                self.join_outrec_paths(e1_idx, e2_idx);
            } else {
                self.join_outrec_paths(e2_idx, e1_idx);
            }
        } else {
            let or1_idx = or1.unwrap();
            let or2_idx = or2.unwrap();
            if self.outrec_list[or1_idx].idx < self.outrec_list[or2_idx].idx {
                self.join_outrec_paths(e1_idx, e2_idx);
            } else {
                self.join_outrec_paths(e2_idx, e1_idx);
            }
        }
        Some(result)
    }
}

impl ClipperBase {
    /// Join two outrec paths
    /// Direct port from clipper.engine.cpp JoinOutrecPaths (line 1435)
    pub(crate) fn join_outrec_paths(&mut self, e1_idx: usize, e2_idx: usize) {
        let or1 = self.active_arena[e1_idx].outrec.unwrap();
        let or2 = self.active_arena[e2_idx].outrec.unwrap();

        let p1_st = self.outrec_list[or1].pts.unwrap();
        let p2_st = self.outrec_list[or2].pts.unwrap();
        let p1_end = self.outpt_arena[p1_st].next;
        let p2_end = self.outpt_arena[p2_st].next;

        if self.is_front(e1_idx) {
            self.outpt_arena[p2_end].prev = p1_st;
            self.outpt_arena[p1_st].next = p2_end;
            self.outpt_arena[p2_st].next = p1_end;
            self.outpt_arena[p1_end].prev = p2_st;
            self.outrec_list[or1].pts = Some(p2_st);
            self.outrec_list[or1].front_edge = self.outrec_list[or2].front_edge;
            if let Some(fe) = self.outrec_list[or1].front_edge {
                self.active_arena[fe].outrec = Some(or1);
            }
        } else {
            self.outpt_arena[p1_end].prev = p2_st;
            self.outpt_arena[p2_st].next = p1_end;
            self.outpt_arena[p1_st].next = p2_end;
            self.outpt_arena[p2_end].prev = p1_st;
            self.outrec_list[or1].back_edge = self.outrec_list[or2].back_edge;
            if let Some(be) = self.outrec_list[or1].back_edge {
                self.active_arena[be].outrec = Some(or1);
            }
        }

        self.outrec_list[or2].front_edge = None;
        self.outrec_list[or2].back_edge = None;
        self.outrec_list[or2].pts = None;

        if self.is_open_end_idx(e1_idx) {
            self.outrec_list[or2].pts = self.outrec_list[or1].pts;
            self.outrec_list[or1].pts = None;
        } else {
            set_owner(&mut self.outrec_list, or2, or1);
        }

        self.active_arena[e1_idx].outrec = None;
        self.active_arena[e2_idx].outrec = None;
    }
}

impl ClipperBase {
    /// Split joined edges
    /// Direct port from clipper.engine.cpp Split (line 2796)
    pub(crate) fn split(&mut self, e_idx: usize, pt: Point64) {
        if self.active_arena[e_idx].join_with == JoinWith::Right {
            self.active_arena[e_idx].join_with = JoinWith::NoJoin;
            let next = self.active_arena[e_idx].next_in_ael.unwrap();
            self.active_arena[next].join_with = JoinWith::NoJoin;
            self.add_local_min_poly(e_idx, next, pt, true);
        } else {
            self.active_arena[e_idx].join_with = JoinWith::NoJoin;
            let prev = self.active_arena[e_idx].prev_in_ael.unwrap();
            self.active_arena[prev].join_with = JoinWith::NoJoin;
            self.add_local_min_poly(prev, e_idx, pt, true);
        }
    }
}

impl ClipperBase {
    /// Delete an active from the AEL
    /// Direct port from clipper.engine.cpp DeleteFromAEL (line 2099)
    pub(crate) fn delete_from_ael(&mut self, e_idx: usize) {
        let prev = self.active_arena[e_idx].prev_in_ael;
        let next = self.active_arena[e_idx].next_in_ael;
        if prev.is_none() && next.is_none() && self.actives != Some(e_idx) {
            return; // already deleted
        }
        if let Some(prev_idx) = prev {
            self.active_arena[prev_idx].next_in_ael = next;
        } else {
            self.actives = next;
        }
        if let Some(next_idx) = next {
            self.active_arena[next_idx].prev_in_ael = prev;
        }
        // Mark as unlinked
        self.active_arena[e_idx].prev_in_ael = None;
        self.active_arena[e_idx].next_in_ael = None;
    }
}

impl ClipperBase {
    /// Swap positions of two adjacent active edges in the AEL
    /// Direct port from clipper.engine.cpp SwapPositionsInAEL (line 2482)
    pub(crate) fn swap_positions_in_ael(&mut self, e1_idx: usize, e2_idx: usize) {
        // precondition: e1 must be immediately to the left of e2
        let next = self.active_arena[e2_idx].next_in_ael;
        if let Some(next_idx) = next {
            self.active_arena[next_idx].prev_in_ael = Some(e1_idx);
        }
        let prev = self.active_arena[e1_idx].prev_in_ael;
        if let Some(prev_idx) = prev {
            self.active_arena[prev_idx].next_in_ael = Some(e2_idx);
        }
        self.active_arena[e2_idx].prev_in_ael = prev;
        self.active_arena[e2_idx].next_in_ael = Some(e1_idx);
        self.active_arena[e1_idx].prev_in_ael = Some(e2_idx);
        self.active_arena[e1_idx].next_in_ael = next;
        if self.active_arena[e2_idx].prev_in_ael.is_none() {
            self.actives = Some(e2_idx);
        }
    }
}

impl ClipperBase {
    /// Copy AEL to SEL and update curr_x for top_y
    /// Port from clipper.engine.cpp AdjustCurrXAndCopyToSEL (line 2113); unlike
    /// C++, also returns whether any edge's new curr_x is less than its left
    /// neighbour's, i.e. whether any edges cross in this scanbeam.
    pub(crate) fn adjust_curr_x_and_copy_to_sel(&mut self, top_y: i64) -> bool {
        let mut e_opt = self.actives;
        self.sel = e_opt;
        let mut prev_x = i64::MIN;
        let mut out_of_order = false;
        while let Some(e_idx) = e_opt {
            let e = &mut self.active_arena[e_idx];
            e.prev_in_sel = e.prev_in_ael;
            e.next_in_sel = e.next_in_ael;
            e.jump = e.next_in_sel;
            e.curr_x = top_x(e, top_y);
            out_of_order |= e.curr_x < prev_x;
            prev_x = e.curr_x;
            e_opt = e.next_in_ael;
        }
        out_of_order
    }
}

impl ClipperBase {
    /// Trim horizontal edge
    /// Direct port from clipper.engine.cpp TrimHorz (line 1719)
    pub(crate) fn trim_horz(&mut self, e_idx: usize) {
        let mut was_trimmed = false;
        let nv = self.next_vertex_idx(e_idx);
        let mut pt = self.vertex_arena[nv].pt;

        while pt.y == self.active_arena[e_idx].top.y {
            if self.preserve_collinear
                && (pt.x < self.active_arena[e_idx].top.x)
                    != (self.active_arena[e_idx].bot.x < self.active_arena[e_idx].top.x)
            {
                break;
            }

            self.active_arena[e_idx].vertex_top = self.next_vertex_idx(e_idx);
            self.active_arena[e_idx].top = pt;
            was_trimmed = true;
            if self.is_maxima_idx(e_idx) {
                break;
            }
            let nv2 = self.next_vertex_idx(e_idx);
            pt = self.vertex_arena[nv2].pt;
        }

        if was_trimmed {
            set_dx(&mut self.active_arena[e_idx]);
        }
    }
}

impl ClipperBase {
    /// Update edge into AEL (advance to next segment)
    /// Direct port from clipper.engine.cpp UpdateEdgeIntoAEL (line 1742)
    pub(crate) fn update_edge_into_ael(&mut self, e_idx: usize) {
        self.active_arena[e_idx].bot = self.active_arena[e_idx].top;
        self.active_arena[e_idx].vertex_top = self.next_vertex_idx(e_idx);
        self.active_arena[e_idx].top = self.vertex_arena[self.active_arena[e_idx].vertex_top].pt;
        self.active_arena[e_idx].curr_x = self.active_arena[e_idx].bot.x;
        set_dx(&mut self.active_arena[e_idx]);

        if is_joined(&self.active_arena[e_idx]) {
            let bot = self.active_arena[e_idx].bot;
            self.split(e_idx, bot);
        }

        if self.is_horizontal_idx(e_idx) {
            if !self.is_open_idx(e_idx) {
                self.trim_horz(e_idx);
            }
            return;
        }

        let top_y = self.active_arena[e_idx].top.y;
        self.insert_scanline(top_y);
        let bot = self.active_arena[e_idx].bot;
        self.check_join_left(e_idx, bot, false);
        self.check_join_right(e_idx, bot, true);
    }
}

impl ClipperBase {
    /// Find edge with matching local min
    /// Direct port from clipper.engine.cpp FindEdgeWithMatchingLocMin (line 1763)
    pub(crate) fn find_edge_with_matching_loc_min(&self, e_idx: usize) -> Option<usize> {
        let local_min = self.active_arena[e_idx].local_min;

        // Search forward
        let mut result = self.active_arena[e_idx].next_in_ael;
        while let Some(r_idx) = result {
            if self.active_arena[r_idx].local_min == local_min {
                return Some(r_idx);
            }
            if !self.is_horizontal_idx(r_idx)
                && self.active_arena[e_idx].bot != self.active_arena[r_idx].bot
            {
                result = None;
                break;
            }
            result = self.active_arena[r_idx].next_in_ael;
        }

        // Search backward
        if result.is_none() {
            result = self.active_arena[e_idx].prev_in_ael;
            while let Some(r_idx) = result {
                if self.active_arena[r_idx].local_min == local_min {
                    return Some(r_idx);
                }
                if !self.is_horizontal_idx(r_idx)
                    && self.active_arena[e_idx].bot != self.active_arena[r_idx].bot
                {
                    return None;
                }
                result = self.active_arena[r_idx].prev_in_ael;
            }
        }
        result
    }
}
