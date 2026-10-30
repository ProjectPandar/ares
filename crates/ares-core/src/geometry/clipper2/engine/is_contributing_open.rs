// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Check if an open path edge is contributing to the solution
    /// Direct port from clipper.engine.cpp IsContributingOpen (line 984)
    pub(crate) fn is_contributing_open(&self, e_idx: usize) -> bool {
        let e = &self.active_arena[e_idx];
        let (is_in_clip, is_in_subj) = match self.fillrule {
            FillRule::Positive => (e.wind_cnt2 > 0, e.wind_cnt > 0),
            FillRule::Negative => (e.wind_cnt2 < 0, e.wind_cnt < 0),
            _ => (e.wind_cnt2 != 0, e.wind_cnt != 0),
        };

        match self.cliptype {
            ClipType::Intersection => is_in_clip,
            ClipType::Union => !is_in_subj && !is_in_clip,
            _ => !is_in_clip,
        }
    }
}

impl ClipperBase {
    /// Set wind count for a closed path edge
    /// Direct port from clipper.engine.cpp SetWindCountForClosedPathEdge (line 1011)
    pub(crate) fn set_wind_count_for_closed_path_edge(&mut self, e_idx: usize) {
        let pt = self.get_poly_type_idx(e_idx);
        let e_wind_dx = self.active_arena[e_idx].wind_dx;

        // Find nearest closed path edge of same PolyType in AEL (heading left)
        let mut e2_opt = self.active_arena[e_idx].prev_in_ael;
        while let Some(e2_idx) = e2_opt {
            if self.get_poly_type_idx(e2_idx) == pt && !self.is_open_idx(e2_idx) {
                break;
            }
            e2_opt = self.active_arena[e2_idx].prev_in_ael;
        }

        let e2_start; // where to start scanning for wind_cnt2
        if let Some(e2_idx) = e2_opt {
            if self.fillrule == FillRule::EvenOdd {
                self.active_arena[e_idx].wind_cnt = e_wind_dx;
                self.active_arena[e_idx].wind_cnt2 = self.active_arena[e2_idx].wind_cnt2;
                e2_start = self.active_arena[e2_idx].next_in_ael;
            } else {
                let e2_wind_cnt = self.active_arena[e2_idx].wind_cnt;
                let e2_wind_dx = self.active_arena[e2_idx].wind_dx;
                // NonZero, Positive, or Negative filling
                if e2_wind_cnt * e2_wind_dx < 0 {
                    // opposite directions: 'e' is outside 'e2'
                    if e2_wind_cnt.abs() > 1 {
                        if e2_wind_dx * e_wind_dx < 0 {
                            self.active_arena[e_idx].wind_cnt = e2_wind_cnt;
                        } else {
                            self.active_arena[e_idx].wind_cnt = e2_wind_cnt + e_wind_dx;
                        }
                    } else {
                        self.active_arena[e_idx].wind_cnt = if self.is_open_idx(e_idx) {
                            1
                        } else {
                            e_wind_dx
                        };
                    }
                } else {
                    // 'e' must be inside 'e2'
                    if e2_wind_dx * e_wind_dx < 0 {
                        self.active_arena[e_idx].wind_cnt = e2_wind_cnt;
                    } else {
                        self.active_arena[e_idx].wind_cnt = e2_wind_cnt + e_wind_dx;
                    }
                }
                self.active_arena[e_idx].wind_cnt2 = self.active_arena[e2_idx].wind_cnt2;
                e2_start = self.active_arena[e2_idx].next_in_ael;
            }
        } else {
            self.active_arena[e_idx].wind_cnt = e_wind_dx;
            e2_start = self.actives;
        }

        // Update wind_cnt2
        let mut e2_cur = e2_start;
        if self.fillrule == FillRule::EvenOdd {
            while e2_cur != Some(e_idx) {
                if let Some(e2i) = e2_cur {
                    if self.get_poly_type_idx(e2i) != pt && !self.is_open_idx(e2i) {
                        let wc2 = self.active_arena[e_idx].wind_cnt2;
                        self.active_arena[e_idx].wind_cnt2 = if wc2 == 0 { 1 } else { 0 };
                    }
                    e2_cur = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
        } else {
            while e2_cur != Some(e_idx) {
                if let Some(e2i) = e2_cur {
                    if self.get_poly_type_idx(e2i) != pt && !self.is_open_idx(e2i) {
                        let e2_wd = self.active_arena[e2i].wind_dx;
                        self.active_arena[e_idx].wind_cnt2 += e2_wd;
                    }
                    e2_cur = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
        }
    }
}

impl ClipperBase {
    /// Set wind count for an open path edge
    /// Direct port from clipper.engine.cpp SetWindCountForOpenPathEdge (line 1089)
    pub(crate) fn set_wind_count_for_open_path_edge(&mut self, e_idx: usize) {
        let mut e2_opt = self.actives;
        if self.fillrule == FillRule::EvenOdd {
            let mut cnt1 = 0i32;
            let mut cnt2 = 0i32;
            while e2_opt != Some(e_idx) {
                if let Some(e2i) = e2_opt {
                    if self.get_poly_type_idx(e2i) == PathType::Clip {
                        cnt2 += 1;
                    } else if !self.is_open_idx(e2i) {
                        cnt1 += 1;
                    }
                    e2_opt = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
            self.active_arena[e_idx].wind_cnt = if is_odd(cnt1) { 1 } else { 0 };
            self.active_arena[e_idx].wind_cnt2 = if is_odd(cnt2) { 1 } else { 0 };
        } else {
            while e2_opt != Some(e_idx) {
                if let Some(e2i) = e2_opt {
                    if self.get_poly_type_idx(e2i) == PathType::Clip {
                        self.active_arena[e_idx].wind_cnt2 += self.active_arena[e2i].wind_dx;
                    } else if !self.is_open_idx(e2i) {
                        self.active_arena[e_idx].wind_cnt += self.active_arena[e2i].wind_dx;
                    }
                    e2_opt = self.active_arena[e2i].next_in_ael;
                } else {
                    break;
                }
            }
        }
    }
}

impl ClipperBase {
    /// Insert a left-bound edge into the AEL
    /// Direct port from clipper.engine.cpp InsertLeftEdge (line 1157)
    pub(crate) fn insert_left_edge(&mut self, e_idx: usize) {
        if let Some(actives_idx) = self.actives {
            if !is_valid_ael_order(
                &self.active_arena[actives_idx],
                &self.active_arena[e_idx],
                &self.vertex_arena,
                &self.minima_list,
            ) {
                self.active_arena[e_idx].prev_in_ael = None;
                self.active_arena[e_idx].next_in_ael = self.actives;
                self.active_arena[actives_idx].prev_in_ael = Some(e_idx);
                self.actives = Some(e_idx);
            } else {
                let mut e2 = actives_idx;
                while let Some(next) = self.active_arena[e2].next_in_ael {
                    if !is_valid_ael_order(
                        &self.active_arena[next],
                        &self.active_arena[e_idx],
                        &self.vertex_arena,
                        &self.minima_list,
                    ) {
                        break;
                    }
                    e2 = next;
                }
                if self.active_arena[e2].join_with == JoinWith::Right {
                    if let Some(next) = self.active_arena[e2].next_in_ael {
                        e2 = next;
                    } else {
                        return;
                    }
                }
                let next = self.active_arena[e2].next_in_ael;
                self.active_arena[e_idx].next_in_ael = next;
                if let Some(next_idx) = next {
                    self.active_arena[next_idx].prev_in_ael = Some(e_idx);
                }
                self.active_arena[e_idx].prev_in_ael = Some(e2);
                self.active_arena[e2].next_in_ael = Some(e_idx);
            }
        } else {
            self.active_arena[e_idx].prev_in_ael = None;
            self.active_arena[e_idx].next_in_ael = None;
            self.actives = Some(e_idx);
        }
    }
}

impl ClipperBase {
    /// Insert a right-bound edge into AEL after its left-bound partner
    /// Direct port from clipper.engine.cpp InsertRightEdge (line 1189)
    pub(crate) fn insert_right_edge(&mut self, e_idx: usize, e2_idx: usize) {
        let next = self.active_arena[e_idx].next_in_ael;
        self.active_arena[e2_idx].next_in_ael = next;
        if let Some(next_idx) = next {
            self.active_arena[next_idx].prev_in_ael = Some(e2_idx);
        }
        self.active_arena[e2_idx].prev_in_ael = Some(e_idx);
        self.active_arena[e_idx].next_in_ael = Some(e2_idx);
    }
}

impl ClipperBase {
    /// Add an output point for an active edge
    /// Direct port from clipper.engine.cpp AddOutPt (line 1497)
    pub(crate) fn add_out_pt(&mut self, e_idx: usize, pt: Point64) -> usize {
        let or_idx = self.active_arena[e_idx].outrec.unwrap();
        let to_front = self.is_front(e_idx);
        let op_front = self.outrec_list[or_idx].pts.unwrap();
        let op_back = self.outpt_arena[op_front].next;

        if to_front {
            if pt == self.outpt_arena[op_front].pt {
                return op_front;
            }
        } else if pt == self.outpt_arena[op_back].pt {
            return op_back;
        }

        let new_idx = self.outpt_arena.len();
        let mut new_op = OutPt::new(pt, or_idx);
        new_op.prev = op_front;
        new_op.next = op_back;
        self.outpt_arena.push(new_op);
        self.outpt_arena[op_back].prev = new_idx;
        self.outpt_arena[op_front].next = new_idx;

        if to_front {
            self.outrec_list[or_idx].pts = Some(new_idx);
        }
        new_idx
    }
}

impl ClipperBase {
    /// Start an open path output
    /// Direct port from clipper.engine.cpp StartOpenPath (line 1696)
    pub(crate) fn start_open_path(&mut self, e_idx: usize, pt: Point64) -> usize {
        let outrec_idx = self.new_out_rec();
        self.outrec_list[outrec_idx].is_open = true;

        if self.active_arena[e_idx].wind_dx > 0 {
            self.outrec_list[outrec_idx].front_edge = Some(e_idx);
            self.outrec_list[outrec_idx].back_edge = None;
        } else {
            self.outrec_list[outrec_idx].front_edge = None;
            self.outrec_list[outrec_idx].back_edge = Some(e_idx);
        }

        self.active_arena[e_idx].outrec = Some(outrec_idx);

        let op_idx = self.new_out_pt(pt, outrec_idx);
        self.outrec_list[outrec_idx].pts = Some(op_idx);
        op_idx
    }
}

impl ClipperBase {
    /// Add a local minimum polygon
    /// Direct port from clipper.engine.cpp AddLocalMinPoly (line 1332)
    pub(crate) fn add_local_min_poly(
        &mut self,
        e1_idx: usize,
        e2_idx: usize,
        pt: Point64,
        is_new: bool,
    ) -> usize {
        let outrec_idx = self.new_out_rec();
        self.active_arena[e1_idx].outrec = Some(outrec_idx);
        self.active_arena[e2_idx].outrec = Some(outrec_idx);

        if self.is_open_idx(e1_idx) {
            self.outrec_list[outrec_idx].owner = None;
            self.outrec_list[outrec_idx].is_open = true;
            if self.active_arena[e1_idx].wind_dx > 0 {
                self.set_sides(outrec_idx, e1_idx, e2_idx);
            } else {
                self.set_sides(outrec_idx, e2_idx, e1_idx);
            }
        } else {
            let prev_hot = self.get_prev_hot_edge(e1_idx);
            if let Some(prev_hot_idx) = prev_hot {
                if self.using_polytree {
                    if let Some(prev_or) = self.active_arena[prev_hot_idx].outrec {
                        set_owner(&mut self.outrec_list, outrec_idx, prev_or);
                    }
                }
                if outrec_is_ascending(prev_hot_idx, &self.outrec_list, &self.active_arena)
                    == is_new
                {
                    self.set_sides(outrec_idx, e2_idx, e1_idx);
                } else {
                    self.set_sides(outrec_idx, e1_idx, e2_idx);
                }
            } else {
                self.outrec_list[outrec_idx].owner = None;
                if is_new {
                    self.set_sides(outrec_idx, e1_idx, e2_idx);
                } else {
                    self.set_sides(outrec_idx, e2_idx, e1_idx);
                }
            }
        }

        let op_idx = self.new_out_pt(pt, outrec_idx);
        self.outrec_list[outrec_idx].pts = Some(op_idx);
        op_idx
    }
}
