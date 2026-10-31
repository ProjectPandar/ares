// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

// The three control-flow exits of clipper.engine.cpp:1672–1706.
enum SelfIntersectionStep {
    Next(usize),
    Restart(usize),
    Finish,
}

impl ClipperBase {
    pub(crate) fn fix_self_intersects(&mut self, record: usize) {
        let mut current = self.outrec_list[record].pts.unwrap();
        if self.outpt_arena[current].prev == self.outpt_arena[self.outpt_arena[current].next].next {
            return;
        }
        loop {
            match self.self_intersection_step(record, current) {
                SelfIntersectionStep::Next(next) => current = next,
                SelfIntersectionStep::Restart(next) => {
                    current = next;
                    continue;
                }
                SelfIntersectionStep::Finish => break,
            }
            if Some(current) == self.outrec_list[record].pts {
                break;
            }
        }
    }
    fn self_intersection_step(&mut self, record: usize, current: usize) -> SelfIntersectionStep {
        let previous = self.outpt_arena[current].prev;
        let next = self.outpt_arena[current].next;
        let next_next = self.outpt_arena[next].next;
        if !segments_intersect(
            self.outpt_arena[previous].pt,
            self.outpt_arena[current].pt,
            self.outpt_arena[next].pt,
            self.outpt_arena[next_next].pt,
            false,
        ) {
            return SelfIntersectionStep::Next(next);
        }
        let third = self.outpt_arena[next_next].next;
        if segments_intersect(
            self.outpt_arena[previous].pt,
            self.outpt_arena[current].pt,
            self.outpt_arena[next_next].pt,
            self.outpt_arena[third].pt,
            false,
        ) {
            let duplicate = self.duplicate_op(current, false);
            let next_next = self.outpt_arena[self.outpt_arena[duplicate].next].next;
            let third = self.outpt_arena[next_next].next;
            self.outpt_arena[duplicate].pt = self.outpt_arena[third].pt;
            return SelfIntersectionStep::Next(self.outpt_arena[duplicate].next);
        }
        if Some(current) == self.outrec_list[record].pts
            || Some(next) == self.outrec_list[record].pts
        {
            let start = self.outrec_list[record].pts.unwrap();
            self.outrec_list[record].pts = Some(self.outpt_arena[start].prev);
        }
        self.do_split_op(record, current);
        let Some(restart) = self.outrec_list[record].pts else {
            return SelfIntersectionStep::Finish;
        };
        if self.outpt_arena[restart].prev == self.outpt_arena[self.outpt_arena[restart].next].next {
            return SelfIntersectionStep::Finish;
        }
        SelfIntersectionStep::Restart(restart)
    }
}

impl ClipperBase {
    /// Do split operation on self-intersecting polygon
    /// Direct port from clipper.engine.cpp DoSplitOp (line 1562)
    pub(crate) fn do_split_op(&mut self, outrec_idx: usize, split_op: usize) {
        let prev_op = self.outpt_arena[split_op].prev;
        let next_op = self.outpt_arena[split_op].next;
        let next_next_op = self.outpt_arena[next_op].next;
        self.outrec_list[outrec_idx].pts = Some(prev_op);

        let mut ip = self.outpt_arena[prev_op].pt;
        get_line_intersect_pt(
            self.outpt_arena[prev_op].pt,
            self.outpt_arena[split_op].pt,
            self.outpt_arena[next_op].pt,
            self.outpt_arena[next_next_op].pt,
            &mut ip,
        );

        if let Some(z_cb) = &mut self.z_callback {
            let oa = &self.outpt_arena;
            z_cb(
                &oa[prev_op].pt,
                &oa[split_op].pt,
                &oa[oa[split_op].next].pt,
                &oa[next_next_op].pt,
                &mut ip,
            );
        }

        let area1 = area_outpt(self.outrec_list[outrec_idx].pts.unwrap(), &self.outpt_arena);
        let abs_area1 = area1.abs();
        if abs_area1 < 2.0 {
            self.dispose_out_pts(outrec_idx);
            return;
        }

        let area2 = area_triangle(
            ip,
            self.outpt_arena[split_op].pt,
            self.outpt_arena[next_op].pt,
        );
        let abs_area2 = area2.abs();

        // De-link split_op and next_op, inserting intersection point
        if ip == self.outpt_arena[prev_op].pt || ip == self.outpt_arena[next_next_op].pt {
            self.outpt_arena[next_next_op].prev = prev_op;
            self.outpt_arena[prev_op].next = next_next_op;
        } else {
            let new_op2 = self.new_out_pt(ip, self.outpt_arena[prev_op].outrec);
            self.outpt_arena[new_op2].prev = prev_op;
            self.outpt_arena[new_op2].next = next_next_op;
            self.outpt_arena[next_next_op].prev = new_op2;
            self.outpt_arena[prev_op].next = new_op2;
        }

        if abs_area2 >= 1.0 && (abs_area2 > abs_area1 || (area2 > 0.0) == (area1 > 0.0)) {
            let new_or = self.new_out_rec();
            self.outrec_list[new_or].owner = self.outrec_list[outrec_idx].owner;

            self.outpt_arena[split_op].outrec = new_or;
            self.outpt_arena[next_op].outrec = new_or;

            let new_op = self.new_out_pt(ip, new_or);
            self.outpt_arena[new_op].prev = next_op;
            self.outpt_arena[new_op].next = split_op;
            self.outrec_list[new_or].pts = Some(new_op);
            self.outpt_arena[split_op].prev = new_op;
            self.outpt_arena[next_op].next = new_op;

            if self.using_polytree {
                self.register_split_owner(outrec_idx, new_or, prev_op, new_op);
            }
        }
        // Otherwise the split triangle is too small, just discard split_op and next_op
    }
}

impl ClipperBase {
    /// Check bounds of an outrec
    /// Direct port from clipper.engine.cpp CheckBounds (line 2929)
    pub fn check_bounds(&mut self, outrec_idx: usize) -> bool {
        if self.outrec_list[outrec_idx].pts.is_none() {
            return false;
        }
        if !self.outrec_list[outrec_idx].bounds.is_empty() {
            return true;
        }
        self.clean_collinear(outrec_idx);
        if self.outrec_list[outrec_idx].pts.is_none() {
            return false;
        }

        let op_start = self.outrec_list[outrec_idx].pts.unwrap();
        let path =
            build_path64_from_outpt(op_start, self.reverse_solution, false, &self.outpt_arena);
        match path {
            None => {
                self.outrec_list[outrec_idx].path = Path64::new();
                false
            }
            Some(p) => {
                self.outrec_list[outrec_idx].bounds = get_bounds(&p);
                self.outrec_list[outrec_idx].path = p;
                true
            }
        }
    }
}

impl ClipperBase {
    /// `clipper.engine.cpp:2967–2994`: inspect the live split vector by index.
    pub(crate) fn check_split_owner(&mut self, outrec_idx: usize, splits_owner: usize) -> bool {
        let mut index = 0;
        while index
            < self.outrec_list[splits_owner]
                .splits
                .as_ref()
                .unwrap()
                .len()
        {
            let split_idx = self.outrec_list[splits_owner].splits.as_ref().unwrap()[index];
            index += 1;
            if self.outrec_list[split_idx].pts.is_none()
                && self.outrec_list[split_idx].splits.is_some()
                && self.check_split_owner(outrec_idx, split_idx)
            {
                return true;
            }
            let real_split = get_real_outrec(&self.outrec_list, split_idx);
            let split = match real_split {
                Some(s) if s != outrec_idx => s,
                _ => continue,
            };

            if self.outrec_list[split].recursive_split == Some(outrec_idx) {
                continue;
            }
            self.outrec_list[split].recursive_split = Some(outrec_idx);

            if self.outrec_list[split].splits.is_some() && self.check_split_owner(outrec_idx, split)
            {
                return true;
            }

            if !self.check_bounds(split) {
                continue;
            }
            let or_bounds = self.outrec_list[outrec_idx].bounds;
            if !self.outrec_list[split].bounds.contains_rect(&or_bounds) {
                continue;
            }

            let or_pts = self.outrec_list[outrec_idx].pts.unwrap();
            let split_pts = self.outrec_list[split].pts.unwrap();
            if !path2_contains_path1_outpt(or_pts, split_pts, &self.outpt_arena) {
                continue;
            }

            if !is_valid_owner(&self.outrec_list, outrec_idx, split) {
                self.outrec_list[split].owner = self.outrec_list[outrec_idx].owner;
            }

            self.outrec_list[outrec_idx].owner = Some(split);
            return true;
        }
        false
    }
}

impl ClipperBase {
    /// Recursively check and set owners for polytree building
    /// Direct port from clipper.engine.cpp RecursiveCheckOwners (line 2967)
    pub fn recursive_check_owners(&mut self, outrec_idx: usize, polytree: &mut PolyTree64) {
        if self.outrec_list[outrec_idx].polypath.is_some()
            || self.outrec_list[outrec_idx].bounds.is_empty()
        {
            return;
        }

        while let Some(owner) = self.outrec_list[outrec_idx].owner {
            if self.outrec_list[owner].splits.is_some() && self.check_split_owner(outrec_idx, owner)
            {
                break;
            }
            if self.outrec_list[owner].pts.is_some()
                && self.check_bounds(owner)
                && self.outrec_list[owner]
                    .bounds
                    .contains_rect(&self.outrec_list[outrec_idx].bounds)
                && path2_contains_path1_outpt(
                    self.outrec_list[outrec_idx].pts.unwrap(),
                    self.outrec_list[owner].pts.unwrap(),
                    &self.outpt_arena,
                )
            {
                break;
            }
            self.outrec_list[outrec_idx].owner = self.outrec_list[owner].owner;
        }

        let parent = if let Some(owner) = self.outrec_list[outrec_idx].owner {
            if self.outrec_list[owner].polypath.is_none() {
                self.recursive_check_owners(owner, polytree);
            }
            let owner = self.outrec_list[outrec_idx].owner.unwrap();
            self.outrec_list[owner].polypath.unwrap()
        } else {
            0
        };
        let path = self.outrec_list[outrec_idx].path.clone();
        let child = polytree.add_child(parent, path);
        self.outrec_list[outrec_idx].polypath = Some(child);
    }
}

impl ClipperBase {
    // clipper.engine.cpp:DoSplitOp, polytree split registration. Preserve an
    // allocated-but-cleared list; do not clone its live membership.
    fn register_split_owner(
        &mut self,
        record: usize,
        split: usize,
        previous_point: usize,
        split_point: usize,
    ) {
        if path2_contains_path1_outpt(previous_point, split_point, &self.outpt_arena) {
            self.outrec_list[split].splits = Some(vec![record]);
        } else {
            self.outrec_list[record]
                .splits
                .get_or_insert_with(Vec::new)
                .push(split);
        }
    }
}
