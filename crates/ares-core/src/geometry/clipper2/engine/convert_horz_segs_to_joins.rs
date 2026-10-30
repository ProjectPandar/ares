// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Convert horizontal segments to joins
    /// Direct port from clipper.engine.cpp ConvertHorzSegsToJoins (line 2218)
    pub(crate) fn convert_horz_segs_to_joins(&mut self) {
        // Update horizontal segments
        let mut valid_count = 0;
        for i in 0..self.horz_seg_list.len() {
            let op = match self.horz_seg_list[i].left_op {
                Some(op) => op,
                None => {
                    continue;
                }
            };
            let or_idx = self.outpt_arena[op].outrec;
            let real_or = get_real_outrec(&self.outrec_list, or_idx);
            if real_or.is_none() {
                continue;
            }
            let real_or_idx = real_or.unwrap();
            let has_edges = self.outrec_list[real_or_idx].front_edge.is_some();
            let curr_y = self.outpt_arena[op].pt.y;

            let mut op_p = op;
            let mut op_n = op;

            if has_edges {
                let op_a = self.outrec_list[real_or_idx].pts.unwrap();
                let op_z = self.outpt_arena[op_a].next;
                while op_p != op_z && self.outpt_arena[self.outpt_arena[op_p].prev].pt.y == curr_y {
                    op_p = self.outpt_arena[op_p].prev;
                }
                while op_n != op_a && self.outpt_arena[self.outpt_arena[op_n].next].pt.y == curr_y {
                    op_n = self.outpt_arena[op_n].next;
                }
            } else {
                while self.outpt_arena[op_p].prev != op_n
                    && self.outpt_arena[self.outpt_arena[op_p].prev].pt.y == curr_y
                {
                    op_p = self.outpt_arena[op_p].prev;
                }
                while self.outpt_arena[op_n].next != op_p
                    && self.outpt_arena[self.outpt_arena[op_n].next].pt.y == curr_y
                {
                    op_n = self.outpt_arena[op_n].next;
                }
            }

            if self.outpt_arena[op_p].pt.x == self.outpt_arena[op_n].pt.x {
                self.horz_seg_list[i].right_op = None;
                continue;
            }

            // Set heading
            if self.outpt_arena[op_p].pt.x < self.outpt_arena[op_n].pt.x {
                self.horz_seg_list[i].left_op = Some(op_p);
                self.horz_seg_list[i].right_op = Some(op_n);
                self.horz_seg_list[i].left_to_right = true;
            } else {
                self.horz_seg_list[i].left_op = Some(op_n);
                self.horz_seg_list[i].right_op = Some(op_p);
                self.horz_seg_list[i].left_to_right = false;
            }

            // Check if left_op already has a horz
            let left_op = self.horz_seg_list[i].left_op.unwrap();
            if self.outpt_arena[left_op].horz.is_some() {
                self.horz_seg_list[i].right_op = None;
                continue;
            }

            self.outpt_arena[left_op].horz = Some(i);
            valid_count += 1;
        }

        if valid_count < 2 {
            return;
        }

        // Sort by left_op x coordinate
        self.horz_seg_list.sort_by(|a, b| {
            let a_valid = a.right_op.is_some();
            let b_valid = b.right_op.is_some();
            if a_valid != b_valid {
                return if a_valid {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            if !a_valid {
                return std::cmp::Ordering::Equal;
            }
            let a_x = self.outpt_arena[a.left_op.unwrap()].pt.x;
            let b_x = self.outpt_arena[b.left_op.unwrap()].pt.x;
            a_x.cmp(&b_x)
        });

        // Find pairs and create joins
        for i in 0..self.horz_seg_list.len() {
            if self.horz_seg_list[i].right_op.is_none() {
                break;
            }
            for j in (i + 1)..self.horz_seg_list.len() {
                if self.horz_seg_list[j].right_op.is_none() {
                    break;
                }

                let hs1_left_x = self.outpt_arena[self.horz_seg_list[i].left_op.unwrap()]
                    .pt
                    .x;
                let hs1_right_x = self.outpt_arena[self.horz_seg_list[i].right_op.unwrap()]
                    .pt
                    .x;
                let hs2_left_x = self.outpt_arena[self.horz_seg_list[j].left_op.unwrap()]
                    .pt
                    .x;
                let hs2_right_x = self.outpt_arena[self.horz_seg_list[j].right_op.unwrap()]
                    .pt
                    .x;

                if hs2_left_x >= hs1_right_x
                    || self.horz_seg_list[j].left_to_right == self.horz_seg_list[i].left_to_right
                    || hs2_right_x <= hs1_left_x
                {
                    continue;
                }

                let curr_y = self.outpt_arena[self.horz_seg_list[i].left_op.unwrap()]
                    .pt
                    .y;
                let hs1_ltr = self.horz_seg_list[i].left_to_right;

                if hs1_ltr {
                    // C++ modifies hs1->left_op persistently
                    let mut lo1 = self.horz_seg_list[i].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo1].next].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo1].next].pt.x <= hs2_left_x
                    {
                        lo1 = self.outpt_arena[lo1].next;
                    }
                    self.horz_seg_list[i].left_op = Some(lo1);
                    let mut lo2 = self.horz_seg_list[j].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo2].prev].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo2].prev].pt.x
                            <= self.outpt_arena[lo1].pt.x
                    {
                        lo2 = self.outpt_arena[lo2].prev;
                    }
                    self.horz_seg_list[j].left_op = Some(lo2);
                    let dup1 = self.duplicate_op(lo1, true);
                    let dup2 = self.duplicate_op(lo2, false);
                    self.horz_join_list.push(HorzJoin::with_ops(dup1, dup2));
                } else {
                    // C++ modifies hs1->left_op persistently
                    let mut lo1 = self.horz_seg_list[i].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo1].prev].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo1].prev].pt.x <= hs2_left_x
                    {
                        lo1 = self.outpt_arena[lo1].prev;
                    }
                    self.horz_seg_list[i].left_op = Some(lo1);
                    let mut lo2 = self.horz_seg_list[j].left_op.unwrap();
                    while self.outpt_arena[self.outpt_arena[lo2].next].pt.y == curr_y
                        && self.outpt_arena[self.outpt_arena[lo2].next].pt.x
                            <= self.outpt_arena[lo1].pt.x
                    {
                        lo2 = self.outpt_arena[lo2].next;
                    }
                    self.horz_seg_list[j].left_op = Some(lo2);
                    let dup1 = self.duplicate_op(lo2, true);
                    let dup2 = self.duplicate_op(lo1, false);
                    self.horz_join_list.push(HorzJoin::with_ops(dup1, dup2));
                }
            }
        }
    }
}

impl ClipperBase {
    /// Process horizontal joins
    /// Direct port from clipper.engine.cpp ProcessHorzJoins (line 2279)
    pub(crate) fn process_horz_joins(&mut self) {
        for idx in 0..self.horz_join_list.len() {
            let op1 = match self.horz_join_list[idx].op1 {
                Some(op) => op,
                None => continue,
            };
            let op2 = match self.horz_join_list[idx].op2 {
                Some(op) => op,
                None => continue,
            };

            let or1 = get_real_outrec(&self.outrec_list, self.outpt_arena[op1].outrec);
            let or2 = get_real_outrec(&self.outrec_list, self.outpt_arena[op2].outrec);
            if or1.is_none() || or2.is_none() {
                continue;
            }
            let or1 = or1.unwrap();
            let or2_orig = or2.unwrap();

            let op1b = self.outpt_arena[op1].next;
            let op2b = self.outpt_arena[op2].prev;
            self.outpt_arena[op1].next = op2;
            self.outpt_arena[op2].prev = op1;
            self.outpt_arena[op1b].prev = op2b;
            self.outpt_arena[op2b].next = op1b;

            if or1 == or2_orig {
                // 'join' is really a split
                let or2_new = self.new_out_rec();
                self.outrec_list[or2_new].pts = Some(op1b);
                fix_outrec_pts(or2_new, &self.outrec_list, &mut self.outpt_arena);

                if self.outrec_list[or1]
                    .pts
                    .map(|p| self.outpt_arena[p].outrec)
                    == Some(or2_new)
                {
                    self.outrec_list[or1].pts = Some(op1);
                    self.outpt_arena[op1].outrec = or1;
                }

                if self.using_polytree {
                    if path2_contains_path1_outpt(
                        self.outrec_list[or1].pts.unwrap(),
                        self.outrec_list[or2_new].pts.unwrap(),
                        &self.outpt_arena,
                    ) {
                        let or1_pts = self.outrec_list[or1].pts;
                        let or2_pts = self.outrec_list[or2_new].pts;
                        self.outrec_list[or1].pts = or2_pts;
                        self.outrec_list[or2_new].pts = or1_pts;
                        fix_outrec_pts(or1, &self.outrec_list, &mut self.outpt_arena);
                        fix_outrec_pts(or2_new, &self.outrec_list, &mut self.outpt_arena);
                        self.outrec_list[or2_new].owner = Some(or1);
                    } else if path2_contains_path1_outpt(
                        self.outrec_list[or2_new].pts.unwrap(),
                        self.outrec_list[or1].pts.unwrap(),
                        &self.outpt_arena,
                    ) {
                        self.outrec_list[or2_new].owner = Some(or1);
                    } else {
                        self.outrec_list[or2_new].owner = self.outrec_list[or1].owner;
                    }

                    if self.outrec_list[or1].splits.is_none() {
                        self.outrec_list[or1].splits = Some(Vec::new());
                    }
                    self.outrec_list[or1].splits.as_mut().unwrap().push(or2_new);
                } else {
                    self.outrec_list[or2_new].owner = Some(or1);
                }
            } else {
                // joining, not splitting
                self.outrec_list[or2_orig].pts = None;
                if self.using_polytree {
                    set_owner(&mut self.outrec_list, or2_orig, or1);
                    if self.outrec_list[or2_orig].splits.is_some() {
                        move_splits(&mut self.outrec_list, or2_orig, or1);
                    }
                } else {
                    self.outrec_list[or2_orig].owner = Some(or1);
                }
            }
        }
    }
}

impl ClipperBase {
    /// Clean collinear points from an OutRec
    /// Direct port from clipper.engine.cpp CleanCollinear (line 1525)
    pub fn clean_collinear(&mut self, outrec_idx: usize) {
        let real_or = get_real_outrec(&self.outrec_list, outrec_idx);
        let or_idx = match real_or {
            Some(idx) => idx,
            None => return,
        };
        if self.outrec_list[or_idx].is_open {
            return;
        }
        if !is_valid_closed_path(self.outrec_list[or_idx].pts, &self.outpt_arena) {
            self.dispose_out_pts(or_idx);
            return;
        }

        let mut start_op = self.outrec_list[or_idx].pts.unwrap();
        let mut op2 = start_op;
        loop {
            let prev = self.outpt_arena[op2].prev;
            let next = self.outpt_arena[op2].next;
            let prev_pt = self.outpt_arena[prev].pt;
            let op2_pt = self.outpt_arena[op2].pt;
            let next_pt = self.outpt_arena[next].pt;

            if is_collinear(prev_pt, op2_pt, next_pt)
                && (op2_pt == prev_pt
                    || op2_pt == next_pt
                    || !self.preserve_collinear
                    || dot_product_three_points(prev_pt, op2_pt, next_pt) < 0.0)
            {
                if Some(op2) == self.outrec_list[or_idx].pts {
                    self.outrec_list[or_idx].pts = Some(prev);
                }
                op2 = self.dispose_out_pt(op2);
                if !is_valid_closed_path(Some(op2), &self.outpt_arena) {
                    self.dispose_out_pts(or_idx);
                    return;
                }
                start_op = op2;
                continue;
            }
            op2 = next;
            if op2 == start_op {
                break;
            }
        }
        self.fix_self_intersects(or_idx);
    }
}
