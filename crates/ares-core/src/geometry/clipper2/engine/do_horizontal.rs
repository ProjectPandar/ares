// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Process a horizontal edge
    /// Direct port from clipper.engine.cpp DoHorizontal (line 2537)
    pub(crate) fn do_horizontal(&mut self, horz_idx: usize) {
        let horz_is_open = self.is_open_idx(horz_idx);
        let y = self.active_arena[horz_idx].bot.y;

        let vertex_max = if horz_is_open {
            get_curr_y_maxima_vertex_open(&self.active_arena[horz_idx], &self.vertex_arena)
        } else {
            get_curr_y_maxima_vertex(&self.active_arena[horz_idx], &self.vertex_arena)
        };

        let (mut horz_left, mut horz_right, mut is_left_to_right) =
            self.reset_horz_direction(horz_idx, vertex_max);

        if is_hot_edge(&self.active_arena[horz_idx]) {
            let curr_x = self.active_arena[horz_idx].curr_x;
            let op = self.add_out_pt(
                horz_idx,
                Point64 {
                    x: curr_x,
                    y,

                    z: self.active_arena[horz_idx].bot.z,
                },
            );
            let or = self.outpt_arena[op].outrec;
            if !self.outrec_list[or].is_open {
                self.add_trial_horz_join(op);
            }
        }

        loop {
            let e_start = if is_left_to_right {
                self.active_arena[horz_idx].next_in_ael
            } else {
                self.active_arena[horz_idx].prev_in_ael
            };

            let mut e_opt = e_start;
            while let Some(e_idx) = e_opt {
                // Check if we've reached the maxima vertex pair
                if Some(self.active_arena[e_idx].vertex_top) == vertex_max {
                    if is_hot_edge(&self.active_arena[horz_idx])
                        && is_joined(&self.active_arena[e_idx])
                    {
                        let top = self.active_arena[e_idx].top;
                        self.split(e_idx, top);
                    }

                    if is_hot_edge(&self.active_arena[horz_idx]) {
                        while Some(self.active_arena[horz_idx].vertex_top) != vertex_max {
                            let top = self.active_arena[horz_idx].top;
                            self.add_out_pt(horz_idx, top);
                            self.update_edge_into_ael(horz_idx);
                        }
                        if is_left_to_right {
                            let top = self.active_arena[horz_idx].top;
                            self.add_local_max_poly(horz_idx, e_idx, top);
                        } else {
                            let top = self.active_arena[horz_idx].top;
                            self.add_local_max_poly(e_idx, horz_idx, top);
                        }
                    }
                    self.delete_from_ael(e_idx);
                    self.delete_from_ael(horz_idx);
                    return;
                }

                // Check break conditions for non-maxima
                if vertex_max != Some(self.active_arena[horz_idx].vertex_top)
                    || self.is_open_end_idx(horz_idx)
                {
                    if (is_left_to_right && self.active_arena[e_idx].curr_x > horz_right)
                        || (!is_left_to_right && self.active_arena[e_idx].curr_x < horz_left)
                    {
                        break;
                    }

                    if self.active_arena[e_idx].curr_x == self.active_arena[horz_idx].top.x
                        && !self.is_horizontal_idx(e_idx)
                    {
                        let nv_idx = self.next_vertex_idx(horz_idx);
                        let nv_pt = self.vertex_arena[nv_idx].pt;

                        if is_left_to_right {
                            if self.is_open_idx(e_idx)
                                && !self.is_same_poly_type_idx(e_idx, horz_idx)
                                && !is_hot_edge(&self.active_arena[e_idx])
                            {
                                if top_x(&self.active_arena[e_idx], nv_pt.y) > nv_pt.x {
                                    break;
                                }
                            } else if top_x(&self.active_arena[e_idx], nv_pt.y) >= nv_pt.x {
                                break;
                            }
                        } else if self.is_open_idx(e_idx)
                            && !self.is_same_poly_type_idx(e_idx, horz_idx)
                            && !is_hot_edge(&self.active_arena[e_idx])
                        {
                            if top_x(&self.active_arena[e_idx], nv_pt.y) < nv_pt.x {
                                break;
                            }
                        } else if top_x(&self.active_arena[e_idx], nv_pt.y) <= nv_pt.x {
                            break;
                        }
                    }
                }

                let pt = Point64::new(
                    self.active_arena[e_idx].curr_x,
                    self.active_arena[horz_idx].bot.y,
                );
                if is_left_to_right {
                    self.intersect_edges(horz_idx, e_idx, pt);
                    self.swap_positions_in_ael(horz_idx, e_idx);
                    let pt2 = pt;
                    self.check_join_left(e_idx, pt2, false);
                    self.active_arena[horz_idx].curr_x = self.active_arena[e_idx].curr_x;
                    e_opt = self.active_arena[horz_idx].next_in_ael;
                } else {
                    self.intersect_edges(e_idx, horz_idx, pt);
                    self.swap_positions_in_ael(e_idx, horz_idx);
                    let pt2 = pt;
                    self.check_join_right(e_idx, pt2, false);
                    self.active_arena[horz_idx].curr_x = self.active_arena[e_idx].curr_x;
                    e_opt = self.active_arena[horz_idx].prev_in_ael;
                }

                if self.active_arena[horz_idx].outrec.is_some() {
                    if let Some(last_op) = get_last_op(
                        horz_idx,
                        &self.active_arena,
                        &self.outrec_list,
                        &self.outpt_arena,
                    ) {
                        self.add_trial_horz_join(last_op);
                    }
                }
            }

            // Check if finished with consecutive horizontals
            if horz_is_open && self.is_open_end_idx(horz_idx) {
                if is_hot_edge(&self.active_arena[horz_idx]) {
                    let top = self.active_arena[horz_idx].top;
                    self.add_out_pt(horz_idx, top);
                    if self.is_front(horz_idx) {
                        let or = self.active_arena[horz_idx].outrec.unwrap();
                        self.outrec_list[or].front_edge = None;
                    } else {
                        let or = self.active_arena[horz_idx].outrec.unwrap();
                        self.outrec_list[or].back_edge = None;
                    }
                    self.active_arena[horz_idx].outrec = None;
                }
                self.delete_from_ael(horz_idx);
                return;
            }

            let nv_idx = self.next_vertex_idx(horz_idx);
            if self.vertex_arena[nv_idx].pt.y != self.active_arena[horz_idx].top.y {
                break;
            }

            // Still more horizontals in bound
            if is_hot_edge(&self.active_arena[horz_idx]) {
                let top = self.active_arena[horz_idx].top;
                self.add_out_pt(horz_idx, top);
            }
            self.update_edge_into_ael(horz_idx);

            let result = self.reset_horz_direction(horz_idx, vertex_max);
            horz_left = result.0;
            horz_right = result.1;
            is_left_to_right = result.2;
        }

        if is_hot_edge(&self.active_arena[horz_idx]) {
            let top = self.active_arena[horz_idx].top;
            let op = self.add_out_pt(horz_idx, top);
            self.add_trial_horz_join(op);
        }

        self.update_edge_into_ael(horz_idx);
    }
}

impl ClipperBase {
    /// Add a new intersect node
    /// Direct port from clipper.engine.cpp AddNewIntersectNode (line 2356)
    pub(crate) fn add_new_intersect_node(&mut self, e1_idx: usize, e2_idx: usize, top_y: i64) {
        let e1 = &self.active_arena[e1_idx];
        let e2 = &self.active_arena[e2_idx];
        let mut ip = Point64::new(e1.curr_x, top_y);
        get_line_intersect_pt(e1.bot, e1.top, e2.bot, e2.top, &mut ip);

        if ip.y > self.bot_y || ip.y < top_y {
            let abs_dx1 = e1.dx.abs();
            let abs_dx2 = e2.dx.abs();
            if abs_dx1 > 100.0 && abs_dx2 > 100.0 {
                if abs_dx1 > abs_dx2 {
                    ip = get_closest_point_on_segment(ip, e1.bot, e1.top);
                } else {
                    ip = get_closest_point_on_segment(ip, e2.bot, e2.top);
                }
            } else if abs_dx1 > 100.0 {
                ip = get_closest_point_on_segment(ip, e1.bot, e1.top);
            } else if abs_dx2 > 100.0 {
                ip = get_closest_point_on_segment(ip, e2.bot, e2.top);
            } else {
                if ip.y < top_y {
                    ip.y = top_y;
                } else {
                    ip.y = self.bot_y;
                }
                if abs_dx1 < abs_dx2 {
                    ip.x = top_x(&self.active_arena[e1_idx], ip.y);
                } else {
                    ip.x = top_x(&self.active_arena[e2_idx], ip.y);
                }
            }
        }
        self.intersect_nodes
            .push(IntersectNode::with_edges(e1_idx, e2_idx, ip));
    }
}

impl ClipperBase {
    /// `clipper.engine.cpp:2416–2472`: stable merge walk of adjacent intersections.
    pub(crate) fn build_intersect_list(&mut self, top_y: i64) -> bool {
        if self.actives.is_none() {
            return false;
        }
        let actives_idx = self.actives.unwrap();
        if self.active_arena[actives_idx].next_in_ael.is_none() {
            return false;
        }

        self.adjust_curr_x_and_copy_to_sel(top_y);

        let mut left_opt = self.sel;
        // Check if we have a jump
        if left_opt.is_none() || self.active_arena[left_opt.unwrap()].jump.is_none() {
            return !self.intersect_nodes.is_empty();
        }

        while let Some(left_idx) = left_opt {
            if self.active_arena[left_idx].jump.is_none() {
                break;
            }

            let mut prev_base: Option<usize> = None;
            let mut left_inner = left_opt;

            while let Some(li) = left_inner {
                if self.active_arena[li].jump.is_none() {
                    break;
                }

                let mut curr_base = li;
                let right_idx = self.active_arena[li].jump.unwrap();
                let mut l_end = right_idx;
                let r_end = self.active_arena[right_idx].jump;
                self.active_arena[li].jump = r_end;

                let mut left_scan = li;
                let mut right_scan = right_idx;

                while left_scan != l_end && right_scan != r_end.unwrap_or(NONE) {
                    if self.active_arena[right_scan].curr_x < self.active_arena[left_scan].curr_x {
                        let mut tmp = self.active_arena[right_scan].prev_in_sel;
                        while let Some(tmp_idx) = tmp {
                            self.add_new_intersect_node(tmp_idx, right_scan, top_y);
                            if tmp_idx == left_scan {
                                break;
                            }
                            tmp = self.active_arena[tmp_idx].prev_in_sel;
                        }

                        let tmp_idx = right_scan;
                        right_scan =
                            extract_from_sel(tmp_idx, &mut self.active_arena).unwrap_or(NONE);
                        l_end = right_scan;

                        insert1_before2_in_sel(tmp_idx, left_scan, &mut self.active_arena);
                        if left_scan == curr_base {
                            curr_base = tmp_idx;
                            self.active_arena[curr_base].jump = r_end;
                            if let Some(pb) = prev_base {
                                self.active_arena[pb].jump = Some(curr_base);
                            } else {
                                self.sel = Some(curr_base);
                            }
                        }
                    } else {
                        left_scan = self.active_arena[left_scan].next_in_sel.unwrap_or(NONE);
                    }
                }

                prev_base = Some(curr_base);
                left_inner = r_end;
            }

            left_opt = self.sel;
        }

        !self.intersect_nodes.is_empty()
    }
}
