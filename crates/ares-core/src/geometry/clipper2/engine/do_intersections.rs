// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Do intersections
    /// Port from clipper.engine.cpp DoIntersections (line 2347); unlike C++,
    /// returns whether every active edge's curr_x is still top_x(e, top_y),
    /// i.e. curr_x was updated for top_y and no intersection moved it.
    pub(crate) fn do_intersections(&mut self, top_y: i64) -> bool {
        if self.build_intersect_list(top_y) {
            self.process_intersect_list();
            self.intersect_nodes.clear();
            false
        } else {
            // build_intersect_list updates curr_x when there are 2+ edges
            self.actives
                .is_some_and(|e| self.active_arena[e].next_in_ael.is_some())
        }
    }
}

impl ClipperBase {
    /// Process the intersection list
    /// Direct port from clipper.engine.cpp ProcessIntersectList (line 2448)
    pub(crate) fn process_intersect_list(&mut self) {
        // Sort by y descending (bottom up), then x ascending
        self.intersect_nodes.sort_by(intersect_list_sort);

        let len = self.intersect_nodes.len();
        for i in 0..len {
            // Ensure edges are adjacent
            if !edges_adjacent_in_ael(&self.intersect_nodes[i], &self.active_arena) {
                let mut j = i + 1;
                while j < len
                    && !edges_adjacent_in_ael(&self.intersect_nodes[j], &self.active_arena)
                {
                    j += 1;
                }
                if j < len {
                    self.intersect_nodes.swap(i, j);
                }
            }

            let e1 = self.intersect_nodes[i].edge1;
            let e2 = self.intersect_nodes[i].edge2;
            let pt = self.intersect_nodes[i].pt;

            self.intersect_edges(e1, e2, pt);
            self.swap_positions_in_ael(e1, e2);

            self.active_arena[e1].curr_x = pt.x;
            self.active_arena[e2].curr_x = pt.x;
            self.check_join_left(e2, pt, true);
            self.check_join_right(e1, pt, true);
        }
    }
}

impl ClipperBase {
    /// Process the top of a scanbeam
    /// Port from clipper.engine.cpp DoTopOfScanbeam (line 2708); unlike C++,
    /// doesn't recompute curr_x for edges that don't end at y when
    /// `curr_x_is_top_x` (do_intersections' result for the same y) says it
    /// already holds top_x(e, y).
    pub(crate) fn do_top_of_scanbeam(&mut self, y: i64, curr_x_is_top_x: bool) {
        self.sel = None;
        let mut e_opt = self.actives;
        while let Some(e_idx) = e_opt {
            if self.active_arena[e_idx].top.y == y {
                self.active_arena[e_idx].curr_x = self.active_arena[e_idx].top.x;
                if self.is_maxima_idx(e_idx) {
                    e_opt = self.do_maxima(e_idx);
                    continue;
                } else {
                    if is_hot_edge(&self.active_arena[e_idx]) {
                        let top = self.active_arena[e_idx].top;
                        self.add_out_pt(e_idx, top);
                    }
                    self.update_edge_into_ael(e_idx);
                    if self.is_horizontal_idx(e_idx) {
                        self.push_horz(e_idx);
                    }
                }
            } else if !curr_x_is_top_x {
                self.active_arena[e_idx].curr_x = top_x(&self.active_arena[e_idx], y);
            }
            e_opt = self.active_arena[e_idx].next_in_ael;
        }
    }
}

impl ClipperBase {
    /// Process a maxima edge
    /// Direct port from clipper.engine.cpp DoMaxima (line 2740)
    pub(crate) fn do_maxima(&mut self, e_idx: usize) -> Option<usize> {
        let prev_e = self.active_arena[e_idx].prev_in_ael;
        let mut next_e = self.active_arena[e_idx].next_in_ael;

        if self.is_open_end_idx(e_idx) {
            if is_hot_edge(&self.active_arena[e_idx]) {
                let top = self.active_arena[e_idx].top;
                self.add_out_pt(e_idx, top);
            }
            if !self.is_horizontal_idx(e_idx) {
                if is_hot_edge(&self.active_arena[e_idx]) {
                    if self.is_front(e_idx) {
                        let or = self.active_arena[e_idx].outrec.unwrap();
                        self.outrec_list[or].front_edge = None;
                    } else {
                        let or = self.active_arena[e_idx].outrec.unwrap();
                        self.outrec_list[or].back_edge = None;
                    }
                    self.active_arena[e_idx].outrec = None;
                }
                self.delete_from_ael(e_idx);
            }
            return next_e;
        }

        let max_pair = get_maxima_pair(&self.active_arena[e_idx], &self.active_arena);
        if max_pair.is_none() {
            return next_e;
        }
        let max_pair_idx = max_pair.unwrap();

        if is_joined(&self.active_arena[e_idx]) {
            let top = self.active_arena[e_idx].top;
            self.split(e_idx, top);
        }
        if is_joined(&self.active_arena[max_pair_idx]) {
            let top = self.active_arena[max_pair_idx].top;
            self.split(max_pair_idx, top);
        }

        // Process edges between maxima pair
        while next_e != Some(max_pair_idx) {
            if let Some(ne_idx) = next_e {
                let top = self.active_arena[e_idx].top;
                self.intersect_edges(e_idx, ne_idx, top);
                self.swap_positions_in_ael(e_idx, ne_idx);
                next_e = self.active_arena[e_idx].next_in_ael;
            } else {
                break;
            }
        }

        if self.is_open_idx(e_idx) {
            if is_hot_edge(&self.active_arena[e_idx]) {
                let top = self.active_arena[e_idx].top;
                self.add_local_max_poly(e_idx, max_pair_idx, top);
            }
            self.delete_from_ael(max_pair_idx);
            self.delete_from_ael(e_idx);
            return if let Some(pe) = prev_e {
                self.active_arena[pe].next_in_ael
            } else {
                self.actives
            };
        }

        // non-open maxima
        if is_hot_edge(&self.active_arena[e_idx]) {
            let top = self.active_arena[e_idx].top;
            self.add_local_max_poly(e_idx, max_pair_idx, top);
        }

        self.delete_from_ael(e_idx);
        self.delete_from_ael(max_pair_idx);
        if let Some(pe) = prev_e {
            self.active_arena[pe].next_in_ael
        } else {
            self.actives
        }
    }
}

impl ClipperBase {
    /// Set horizontal segment heading forward
    #[allow(dead_code)]
    pub(crate) fn set_horz_seg_heading_forward(
        &mut self,
        hs_idx: usize,
        op_p: usize,
        op_n: usize,
    ) -> bool {
        if self.outpt_arena[op_p].pt.x == self.outpt_arena[op_n].pt.x {
            return false;
        }
        if self.outpt_arena[op_p].pt.x < self.outpt_arena[op_n].pt.x {
            self.horz_seg_list[hs_idx].left_op = Some(op_p);
            self.horz_seg_list[hs_idx].right_op = Some(op_n);
            self.horz_seg_list[hs_idx].left_to_right = true;
        } else {
            self.horz_seg_list[hs_idx].left_op = Some(op_n);
            self.horz_seg_list[hs_idx].right_op = Some(op_p);
            self.horz_seg_list[hs_idx].left_to_right = false;
        }
        true
    }
}
