// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:2305–2370, one completed join at a time.
    pub(crate) fn process_horz_joins(&mut self) {
        for index in 0..self.horz_join_list.len() {
            let first = self.horz_join_list[index].op1.unwrap();
            let second = self.horz_join_list[index].op2.unwrap();
            self.process_horz_join(first, second);
        }
    }
    fn process_horz_join(&mut self, first: usize, second: usize) {
        let record1 = get_real_outrec(&self.outrec_list, self.outpt_arena[first].outrec).unwrap();
        let record2 = get_real_outrec(&self.outrec_list, self.outpt_arena[second].outrec).unwrap();
        let first_back = self.outpt_arena[first].next;
        let second_back = self.outpt_arena[second].prev;
        self.outpt_arena[first].next = second;
        self.outpt_arena[second].prev = first;
        self.outpt_arena[first_back].prev = second_back;
        self.outpt_arena[second_back].next = first_back;
        if record1 == record2 {
            let split = self.new_out_rec();
            self.outrec_list[split].pts = Some(first_back);
            fix_outrec_pts(split, &self.outrec_list, &mut self.outpt_arena);
            let original = self.outrec_list[record1].pts.unwrap();
            if self.outpt_arena[original].outrec == split {
                self.outrec_list[record1].pts = Some(first);
                self.outpt_arena[first].outrec = record1;
            }
            if self.using_polytree {
                self.assign_horz_split_owner(record1, split);
            } else {
                self.outrec_list[split].owner = Some(record1);
            }
        } else {
            self.outrec_list[record2].pts = None;
            self.assign_horz_join_owner(record2, record1);
        }
    }

    // ProcessHorzJoins:2333–2358. Point swap/fixups occur before owner and split writes.
    fn assign_horz_split_owner(&mut self, record: usize, split: usize) {
        if path2_contains_path1_outpt(
            self.outrec_list[record].pts.unwrap(),
            self.outrec_list[split].pts.unwrap(),
            &self.outpt_arena,
        ) {
            let points1 = self.outrec_list[record].pts;
            let points2 = self.outrec_list[split].pts;
            self.outrec_list[record].pts = points2;
            self.outrec_list[split].pts = points1;
            fix_outrec_pts(record, &self.outrec_list, &mut self.outpt_arena);
            fix_outrec_pts(split, &self.outrec_list, &mut self.outpt_arena);
            self.outrec_list[split].owner = Some(record);
        } else if path2_contains_path1_outpt(
            self.outrec_list[split].pts.unwrap(),
            self.outrec_list[record].pts.unwrap(),
            &self.outpt_arena,
        ) {
            self.outrec_list[split].owner = Some(record);
        } else {
            self.outrec_list[split].owner = self.outrec_list[record].owner;
        }
        self.outrec_list[record]
            .splits
            .get_or_insert_with(Vec::new)
            .push(split);
    }
    fn assign_horz_join_owner(&mut self, from: usize, to: usize) {
        if self.using_polytree {
            set_owner(&mut self.outrec_list, from, to);
            if self.outrec_list[from].splits.is_some() {
                move_splits(&mut self.outrec_list, from, to);
            }
        } else {
            self.outrec_list[from].owner = Some(to);
        }
    }
}
