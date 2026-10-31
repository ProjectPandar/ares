// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:2193–2241, SetHorzSegHeadingForward/UpdateHorzSegment.
    pub(super) fn update_horz_segment(&mut self, index: usize) -> bool {
        let point = self.horz_seg_list[index].left_op.unwrap();
        let record = get_real_outrec(&self.outrec_list, self.outpt_arena[point].outrec).unwrap();
        let y = self.outpt_arena[point].pt.y;
        let (mut previous, mut next) = (point, point);
        if self.outrec_list[record].front_edge.is_some() {
            let first = self.outrec_list[record].pts.unwrap();
            let last = self.outpt_arena[first].next;
            while previous != last && self.outpt_arena[self.outpt_arena[previous].prev].pt.y == y {
                previous = self.outpt_arena[previous].prev;
            }
            while next != first && self.outpt_arena[self.outpt_arena[next].next].pt.y == y {
                next = self.outpt_arena[next].next;
            }
        } else {
            while self.outpt_arena[previous].prev != next
                && self.outpt_arena[self.outpt_arena[previous].prev].pt.y == y
            {
                previous = self.outpt_arena[previous].prev;
            }
            while self.outpt_arena[next].next != previous
                && self.outpt_arena[self.outpt_arena[next].next].pt.y == y
            {
                next = self.outpt_arena[next].next;
            }
        }
        if !self.set_horz_heading(index, previous, next) {
            self.horz_seg_list[index].right_op = None;
            return false;
        }
        let left = self.horz_seg_list[index].left_op.unwrap();
        if self.outpt_arena[left].horz.is_some() {
            self.horz_seg_list[index].right_op = None;
            return false;
        }
        self.outpt_arena[left].horz = Some(index);
        true
    }
    fn set_horz_heading(&mut self, index: usize, previous: usize, next: usize) -> bool {
        if self.outpt_arena[previous].pt.x == self.outpt_arena[next].pt.x {
            return false;
        }
        if self.outpt_arena[previous].pt.x < self.outpt_arena[next].pt.x {
            self.horz_seg_list[index].left_op = Some(previous);
            self.horz_seg_list[index].right_op = Some(next);
            self.horz_seg_list[index].left_to_right = true;
        } else {
            self.horz_seg_list[index].left_op = Some(next);
            self.horz_seg_list[index].right_op = Some(previous);
            self.horz_seg_list[index].left_to_right = false;
        }
        true
    }
}
