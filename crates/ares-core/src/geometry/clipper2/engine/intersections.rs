// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:2382–2413, including its parallel-edge fallback.
    pub(crate) fn add_new_intersect_node(&mut self, first: usize, second: usize, top_y: i64) {
        let (a, b) = (&self.active_arena[first], &self.active_arena[second]);
        let mut point = Point64::new(a.curr_x, top_y);
        get_line_intersect_pt(a.bot, a.top, b.bot, b.top, &mut point);
        if point.y > self.bot_y || point.y < top_y {
            point = clamp_intersection_to_scanbeam(point, a, b, (self.bot_y, top_y));
        }
        self.intersect_nodes
            .push(IntersectNode::with_edges(first, second, point));
    }

    // clipper.engine.cpp:2416–2472, stable merge of the live SEL.
    pub(crate) fn build_intersect_list(&mut self, top_y: i64) -> bool {
        let Some(first) = self.actives else {
            return false;
        };
        if self.active_arena[first].next_in_ael.is_none() {
            return false;
        }
        self.adjust_curr_x_and_copy_to_sel(top_y);
        while let Some(left) = self.sel {
            if self.active_arena[left].jump.is_none() {
                break;
            }
            self.merge_sel_pass(top_y);
        }
        !self.intersect_nodes.is_empty()
    }
    fn merge_sel_pass(&mut self, top_y: i64) {
        let mut left = self.sel;
        let mut previous_base = None;
        while let Some(index) = left {
            if self.active_arena[index].jump.is_none() {
                break;
            }
            let (base, next) = self.merge_sel_runs(index, previous_base, top_y);
            previous_base = Some(base);
            left = next;
        }
    }
    fn merge_sel_runs(
        &mut self,
        mut left: usize,
        previous_base: Option<usize>,
        top_y: i64,
    ) -> (usize, Option<usize>) {
        let mut base = left;
        let right = self.active_arena[left].jump.unwrap();
        let mut left_end = Some(right);
        let right_end = self.active_arena[right].jump;
        self.active_arena[left].jump = right_end;
        let mut right = Some(right);
        while Some(left) != left_end && right != right_end {
            let right_index = right.unwrap();
            if self.active_arena[right_index].curr_x < self.active_arena[left].curr_x {
                self.add_sel_crossings(left, right_index, top_y);
                right = extract_from_sel(right_index, &mut self.active_arena);
                left_end = right;
                insert1_before2_in_sel(right_index, left, &mut self.active_arena);
                self.update_sel_run_base(left, &mut base, previous_base, (right_index, right_end));
            } else {
                left = self.active_arena[left].next_in_sel.unwrap();
            }
        }
        (base, right_end)
    }
    fn add_sel_crossings(&mut self, left: usize, right: usize, top_y: i64) {
        let mut previous = self.active_arena[right].prev_in_sel.unwrap();
        loop {
            self.add_new_intersect_node(previous, right, top_y);
            if previous == left {
                break;
            }
            previous = self.active_arena[previous].prev_in_sel.unwrap();
        }
    }
    fn update_sel_run_base(
        &mut self,
        left: usize,
        base: &mut usize,
        previous_base: Option<usize>,
        (inserted, right_end): (usize, Option<usize>),
    ) {
        if left != *base {
            return;
        }
        *base = inserted;
        self.active_arena[inserted].jump = right_end;
        match previous_base {
            Some(previous) => self.active_arena[previous].jump = Some(inserted),
            None => self.sel = Some(inserted),
        }
    }
}

// AddNewIntersectNode:2390–2410, retain the source's steep-edge priority,
// exact closest-point/TopX cast sites, and unchanged scanbeam bounds.
fn clamp_intersection_to_scanbeam(
    mut point: Point64,
    first: &Active,
    second: &Active,
    (bottom, top): (i64, i64),
) -> Point64 {
    let first_slope = first.dx.abs();
    let second_slope = second.dx.abs();
    if first_slope > 100.0 && second_slope > 100.0 {
        if first_slope > second_slope {
            point = get_closest_point_on_segment(point, first.bot, first.top);
        } else {
            point = get_closest_point_on_segment(point, second.bot, second.top);
        }
    } else if first_slope > 100.0 {
        point = get_closest_point_on_segment(point, first.bot, first.top);
    } else if second_slope > 100.0 {
        point = get_closest_point_on_segment(point, second.bot, second.top);
    } else {
        point.y = if point.y < top { top } else { bottom };
        point.x = if first_slope < second_slope {
            top_x(first, point.y)
        } else {
            top_x(second, point.y)
        };
    }
    point
}
