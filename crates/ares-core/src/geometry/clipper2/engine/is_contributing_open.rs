// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    // clipper.engine.cpp:1370–1398, closed AddLocalMinPoly side/owner setup.
    fn set_closed_minimum_sides(
        &mut self,
        record: usize,
        first: usize,
        second: usize,
        is_new: bool,
    ) {
        let Some(previous) = self.get_prev_hot_edge(first) else {
            self.outrec_list[record].owner = None;
            if is_new {
                self.set_sides(record, first, second);
            } else {
                self.set_sides(record, second, first);
            }
            return;
        };
        if self.using_polytree {
            let owner = self.active_arena[previous].outrec.unwrap();
            set_owner(&mut self.outrec_list, record, owner);
        }
        if outrec_is_ascending(previous, &self.outrec_list, &self.active_arena) == is_new {
            self.set_sides(record, second, first);
        } else {
            self.set_sides(record, first, second);
        }
    }
}

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
    // clipper.engine.cpp:InsertLeftEdge: empty/head cases finish before the
    // resident walk. A joined-right edge necessarily has a paired successor.
    pub(crate) fn insert_left_edge(&mut self, e_idx: usize) {
        let Some(first) = self.actives else {
            self.active_arena[e_idx].prev_in_ael = None;
            self.active_arena[e_idx].next_in_ael = None;
            self.actives = Some(e_idx);
            return;
        };
        if !is_valid_ael_order(
            &self.active_arena[first],
            &self.active_arena[e_idx],
            &self.vertex_arena,
            &self.minima_list,
        ) {
            self.active_arena[e_idx].prev_in_ael = None;
            self.active_arena[e_idx].next_in_ael = self.actives;
            self.active_arena[first].prev_in_ael = Some(e_idx);
            self.actives = Some(e_idx);
            return;
        }
        let mut previous = first;
        while let Some(next) = self.active_arena[previous].next_in_ael {
            if !is_valid_ael_order(
                &self.active_arena[next],
                &self.active_arena[e_idx],
                &self.vertex_arena,
                &self.minima_list,
            ) {
                break;
            }
            previous = next;
        }
        if self.active_arena[previous].join_with == JoinWith::Right {
            previous = self.active_arena[previous].next_in_ael.unwrap();
        }
        let next = self.active_arena[previous].next_in_ael;
        self.active_arena[e_idx].next_in_ael = next;
        if let Some(next) = next {
            self.active_arena[next].prev_in_ael = Some(e_idx);
        }
        self.active_arena[e_idx].prev_in_ael = Some(previous);
        self.active_arena[previous].next_in_ael = Some(e_idx);
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
            self.set_closed_minimum_sides(outrec_idx, e1_idx, e2_idx, is_new);
        }

        let op_idx = self.new_out_pt(pt, outrec_idx);
        self.outrec_list[outrec_idx].pts = Some(op_idx);
        op_idx
    }
}
