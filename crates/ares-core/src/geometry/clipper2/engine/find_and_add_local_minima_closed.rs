// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Duplicate an OutPt, inserting after or before
    /// Direct port from clipper.engine.cpp DuplicateOp
    pub fn duplicate_op(&mut self, op_idx: usize, insert_after: bool) -> usize {
        let pt = self.outpt_arena[op_idx].pt;
        let outrec = self.outpt_arena[op_idx].outrec;
        let new_idx = self.outpt_arena.len();
        let mut result = OutPt::new(pt, outrec);

        if insert_after {
            let next = self.outpt_arena[op_idx].next;
            result.next = next;
            result.prev = op_idx;
            self.outpt_arena.push(result);
            self.outpt_arena[next].prev = new_idx;
            self.outpt_arena[op_idx].next = new_idx;
        } else {
            let prev = self.outpt_arena[op_idx].prev;
            result.prev = prev;
            result.next = op_idx;
            self.outpt_arena.push(result);
            self.outpt_arena[prev].next = new_idx;
            self.outpt_arena[op_idx].prev = new_idx;
        }

        new_idx
    }
}

impl ClipperBase {
    /// Dispose (unlink) an OutPt, return the next
    /// Direct port from clipper.engine.cpp DisposeOutPt
    pub fn dispose_out_pt(&mut self, op_idx: usize) -> usize {
        let result = self.outpt_arena[op_idx].next;
        let prev = self.outpt_arena[op_idx].prev;
        self.outpt_arena[prev].next = result;
        self.outpt_arena[result].prev = prev;
        // Note: we don't actually free the arena slot, just unlink
        result
    }
}

impl ClipperBase {
    /// Dispose all OutPts in a circular list, setting outrec.pts to None
    /// Direct port from clipper.engine.cpp DisposeOutPts
    pub fn dispose_out_pts(&mut self, outrec_idx: usize) {
        let point = self.outrec_list[outrec_idx].pts.unwrap();
        let previous = self.outpt_arena[point].prev;
        self.outpt_arena[previous].next = NONE;
        let mut current = Some(point);
        while let Some(point) = current {
            let next = self.outpt_arena[point].next;
            current = (next != NONE).then_some(next);
        }
        self.outrec_list[outrec_idx].pts = None;
    }
}

impl ClipperBase {
    /// Set the front and back edges of an OutRec
    /// Direct port from clipper.engine.cpp SetSides
    #[inline]
    pub fn set_sides(&mut self, outrec_idx: usize, start_edge: usize, end_edge: usize) {
        self.outrec_list[outrec_idx].front_edge = Some(start_edge);
        self.outrec_list[outrec_idx].back_edge = Some(end_edge);
    }
}

impl ClipperBase {
    /// Swap OutRecs between two active edges
    /// Direct port from clipper.engine.cpp SwapOutrecs
    pub fn swap_outrecs(&mut self, e1_idx: usize, e2_idx: usize) {
        let or1 = self.active_arena[e1_idx].outrec;
        let or2 = self.active_arena[e2_idx].outrec;

        if or1 == or2 {
            if let Some(or_idx) = or1 {
                let fe = self.outrec_list[or_idx].front_edge;
                let be = self.outrec_list[or_idx].back_edge;
                self.outrec_list[or_idx].front_edge = be;
                self.outrec_list[or_idx].back_edge = fe;
            }
            return;
        }

        if let Some(or1_idx) = or1 {
            if self.outrec_list[or1_idx].front_edge == Some(e1_idx) {
                self.outrec_list[or1_idx].front_edge = Some(e2_idx);
            } else {
                self.outrec_list[or1_idx].back_edge = Some(e2_idx);
            }
        }

        if let Some(or2_idx) = or2 {
            if self.outrec_list[or2_idx].front_edge == Some(e2_idx) {
                self.outrec_list[or2_idx].front_edge = Some(e1_idx);
            } else {
                self.outrec_list[or2_idx].back_edge = Some(e1_idx);
            }
        }

        self.active_arena[e1_idx].outrec = or2;
        self.active_arena[e2_idx].outrec = or1;
    }
}

impl ClipperBase {
    /// Check if an active edge is the front edge of its outrec
    /// Direct port from clipper.engine.cpp IsFront
    #[inline]
    pub fn is_front(&self, e_idx: usize) -> bool {
        if let Some(or_idx) = self.active_arena[e_idx].outrec {
            self.outrec_list[or_idx].front_edge == Some(e_idx)
        } else {
            false
        }
    }
}

impl ClipperBase {
    /// Get the previous hot edge in AEL
    /// Direct port from clipper.engine.cpp GetPrevHotEdge
    pub fn get_prev_hot_edge(&self, e_idx: usize) -> Option<usize> {
        let mut prev = self.active_arena[e_idx].prev_in_ael;
        while let Some(p_idx) = prev {
            if is_open_active(&self.active_arena[p_idx], &self.minima_list)
                || !is_hot_edge(&self.active_arena[p_idx])
            {
                prev = self.active_arena[p_idx].prev_in_ael;
            } else {
                return Some(p_idx);
            }
        }
        None
    }
}
