// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:2244–2291. Segment validity/count and stable sorting
    // precede the fixed valid-prefix pair walk; left_op mutations stay live.
    pub(crate) fn convert_horz_segs_to_joins(&mut self) {
        let mut valid = 0;
        for index in 0..self.horz_seg_list.len() {
            if self.update_horz_segment(index) {
                valid += 1;
            }
        }
        if valid < 2 {
            return;
        }
        self.horz_seg_list
            .sort_by(|a, b| match (a.right_op.is_some(), b.right_op.is_some()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => std::cmp::Ordering::Equal,
                (true, true) => self.outpt_arena[a.left_op.unwrap()]
                    .pt
                    .x
                    .cmp(&self.outpt_arena[b.left_op.unwrap()].pt.x),
            });
        for first in 0..valid - 1 {
            for second in first + 1..valid {
                self.join_horizontal_pair(first, second);
            }
        }
    }

    fn join_horizontal_pair(&mut self, first: usize, second: usize) {
        let left1 = self.outpt_arena[self.horz_seg_list[first].left_op.unwrap()]
            .pt
            .x;
        let right1 = self.outpt_arena[self.horz_seg_list[first].right_op.unwrap()]
            .pt
            .x;
        let left2 = self.outpt_arena[self.horz_seg_list[second].left_op.unwrap()]
            .pt
            .x;
        let right2 = self.outpt_arena[self.horz_seg_list[second].right_op.unwrap()]
            .pt
            .x;
        if left2 >= right1
            || self.horz_seg_list[second].left_to_right == self.horz_seg_list[first].left_to_right
            || right2 <= left1
        {
            return;
        }
        let y = self.outpt_arena[self.horz_seg_list[first].left_op.unwrap()]
            .pt
            .y;
        if self.horz_seg_list[first].left_to_right {
            let mut op1 = self.horz_seg_list[first].left_op.unwrap();
            while self.outpt_arena[self.outpt_arena[op1].next].pt.y == y
                && self.outpt_arena[self.outpt_arena[op1].next].pt.x <= left2
            {
                op1 = self.outpt_arena[op1].next;
            }
            self.horz_seg_list[first].left_op = Some(op1);
            let mut op2 = self.horz_seg_list[second].left_op.unwrap();
            while self.outpt_arena[self.outpt_arena[op2].prev].pt.y == y
                && self.outpt_arena[self.outpt_arena[op2].prev].pt.x <= self.outpt_arena[op1].pt.x
            {
                op2 = self.outpt_arena[op2].prev;
            }
            self.horz_seg_list[second].left_op = Some(op2);
            let duplicate1 = self.duplicate_op(op1, true);
            let duplicate2 = self.duplicate_op(op2, false);
            self.horz_join_list
                .push(HorzJoin::with_ops(duplicate1, duplicate2));
        } else {
            let mut op1 = self.horz_seg_list[first].left_op.unwrap();
            while self.outpt_arena[self.outpt_arena[op1].prev].pt.y == y
                && self.outpt_arena[self.outpt_arena[op1].prev].pt.x <= left2
            {
                op1 = self.outpt_arena[op1].prev;
            }
            self.horz_seg_list[first].left_op = Some(op1);
            let mut op2 = self.horz_seg_list[second].left_op.unwrap();
            while self.outpt_arena[self.outpt_arena[op2].next].pt.y == y
                && self.outpt_arena[self.outpt_arena[op2].next].pt.x <= self.outpt_arena[op1].pt.x
            {
                op2 = self.outpt_arena[op2].next;
            }
            self.horz_seg_list[second].left_op = Some(op2);
            let duplicate1 = self.duplicate_op(op2, true);
            let duplicate2 = self.duplicate_op(op1, false);
            self.horz_join_list
                .push(HorzJoin::with_ops(duplicate1, duplicate2));
        }
    }
}

// clipper.engine.cpp:1551–1586: the native loop can advance, restart or dispose.
enum CollinearStep {
    Advance(usize),
    Restart(usize),
    Disposed,
}

impl ClipperBase {
    pub fn clean_collinear(&mut self, record: usize) {
        let Some(record) = get_real_outrec(&self.outrec_list, record) else {
            return;
        };
        if self.outrec_list[record].is_open {
            return;
        }
        if !is_valid_closed_path(self.outrec_list[record].pts, &self.outpt_arena) {
            self.dispose_out_pts(record);
            return;
        }
        let mut start = self.outrec_list[record].pts.unwrap();
        let mut current = start;
        loop {
            match self.collinear_step(record, current) {
                CollinearStep::Advance(next) => current = next,
                CollinearStep::Restart(next) => {
                    current = next;
                    start = next;
                    continue;
                }
                CollinearStep::Disposed => return,
            }
            if current == start {
                break;
            }
        }
        self.fix_self_intersects(record);
    }
    fn collinear_step(&mut self, record: usize, current: usize) -> CollinearStep {
        let previous = self.outpt_arena[current].prev;
        let next = self.outpt_arena[current].next;
        let previous_point = self.outpt_arena[previous].pt;
        let point = self.outpt_arena[current].pt;
        let next_point = self.outpt_arena[next].pt;
        let removable = is_collinear(previous_point, point, next_point)
            && (point == previous_point
                || point == next_point
                || !self.preserve_collinear
                || dot_product_three_points(previous_point, point, next_point) < 0.0);
        if !removable {
            return CollinearStep::Advance(next);
        }
        if Some(current) == self.outrec_list[record].pts {
            self.outrec_list[record].pts = Some(previous);
        }
        let next = self.dispose_out_pt(current);
        if !is_valid_closed_path(Some(next), &self.outpt_arena) {
            self.dispose_out_pts(record);
            return CollinearStep::Disposed;
        }
        CollinearStep::Restart(next)
    }
}
