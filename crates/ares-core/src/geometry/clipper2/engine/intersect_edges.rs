// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Process intersection between two edges
    /// Direct port from clipper.engine.cpp IntersectEdges (line 1783)
    pub(crate) fn intersect_edges(&mut self, e1_idx: usize, e2_idx: usize, pt: Point64) {
        // MANAGE OPEN PATH INTERSECTIONS SEPARATELY
        if self.has_open_paths && (self.is_open_idx(e1_idx) || self.is_open_idx(e2_idx)) {
            if self.is_open_idx(e1_idx) && self.is_open_idx(e2_idx) {
                return;
            }
            let (edge_o, edge_c) = if self.is_open_idx(e1_idx) {
                (e1_idx, e2_idx)
            } else {
                (e2_idx, e1_idx)
            };

            if is_joined(&self.active_arena[edge_c]) {
                self.split(edge_c, pt);
            }

            if self.active_arena[edge_c].wind_cnt.abs() != 1 {
                return;
            }

            match self.cliptype {
                ClipType::Union => {
                    if !is_hot_edge(&self.active_arena[edge_c]) {
                        return;
                    }
                }
                _ => {
                    if self.minima_list[self.active_arena[edge_c].local_min].polytype
                        == PathType::Subject
                    {
                        return;
                    }
                }
            }

            match self.fillrule {
                FillRule::Positive => {
                    if self.active_arena[edge_c].wind_cnt != 1 {
                        return;
                    }
                }
                FillRule::Negative => {
                    if self.active_arena[edge_c].wind_cnt != -1 {
                        return;
                    }
                }
                _ => {
                    if self.active_arena[edge_c].wind_cnt.abs() != 1 {
                        return;
                    }
                }
            }

            let _result_op: usize;

            // toggle contribution
            if is_hot_edge(&self.active_arena[edge_o]) {
                _result_op = self.add_out_pt(edge_o, pt);
                if self.is_front(edge_o) {
                    let or = self.active_arena[edge_o].outrec.unwrap();
                    self.outrec_list[or].front_edge = None;
                } else {
                    let or = self.active_arena[edge_o].outrec.unwrap();
                    self.outrec_list[or].back_edge = None;
                }
                self.active_arena[edge_o].outrec = None;
            } else if pt == self.active_arena[edge_o].bot
                && pt
                    == self.vertex_arena
                        [self.minima_list[self.active_arena[edge_o].local_min].vertex]
                        .pt
                && !is_open_end_vertex(
                    &self.vertex_arena
                        [self.minima_list[self.active_arena[edge_o].local_min].vertex],
                )
            {
                let e3 = self.find_edge_with_matching_loc_min(edge_o);
                if let Some(e3_idx) = e3 {
                    if is_hot_edge(&self.active_arena[e3_idx]) {
                        let e3_or = self.active_arena[e3_idx].outrec.unwrap();
                        self.active_arena[edge_o].outrec = Some(e3_or);
                        if self.active_arena[edge_o].wind_dx > 0 {
                            self.set_sides(e3_or, edge_o, e3_idx);
                        } else {
                            self.set_sides(e3_or, e3_idx, edge_o);
                        }
                        return;
                    }
                }
                _result_op = self.start_open_path(edge_o, pt);
            } else {
                _result_op = self.start_open_path(edge_o, pt);
            }

            if self.z_callback.is_some() {
                self.set_z(edge_o, edge_c, _result_op);
            }
            return;
        }

        // MANAGING CLOSED PATHS FROM HERE ON
        if is_joined(&self.active_arena[e1_idx]) {
            self.split(e1_idx, pt);
        }
        if is_joined(&self.active_arena[e2_idx]) {
            self.split(e2_idx, pt);
        }

        // UPDATE WINDING COUNTS
        let old_e1_windcnt;
        let old_e2_windcnt;

        if self.is_same_poly_type_idx(e1_idx, e2_idx) {
            if self.fillrule == FillRule::EvenOdd {
                let tmp = self.active_arena[e1_idx].wind_cnt;
                self.active_arena[e1_idx].wind_cnt = self.active_arena[e2_idx].wind_cnt;
                self.active_arena[e2_idx].wind_cnt = tmp;
            } else {
                let e1_wc = self.active_arena[e1_idx].wind_cnt;
                let e2_wd = self.active_arena[e2_idx].wind_dx;
                let e1_wd = self.active_arena[e1_idx].wind_dx;
                if e1_wc + e2_wd == 0 {
                    self.active_arena[e1_idx].wind_cnt = -e1_wc;
                } else {
                    self.active_arena[e1_idx].wind_cnt = e1_wc + e2_wd;
                }
                let e2_wc = self.active_arena[e2_idx].wind_cnt;
                if e2_wc - e1_wd == 0 {
                    self.active_arena[e2_idx].wind_cnt = -e2_wc;
                } else {
                    self.active_arena[e2_idx].wind_cnt = e2_wc - e1_wd;
                }
            }
        } else if self.fillrule != FillRule::EvenOdd {
            self.active_arena[e1_idx].wind_cnt2 += self.active_arena[e2_idx].wind_dx;
            self.active_arena[e2_idx].wind_cnt2 -= self.active_arena[e1_idx].wind_dx;
        } else {
            let wc2_1 = self.active_arena[e1_idx].wind_cnt2;
            self.active_arena[e1_idx].wind_cnt2 = if wc2_1 == 0 { 1 } else { 0 };
            let wc2_2 = self.active_arena[e2_idx].wind_cnt2;
            self.active_arena[e2_idx].wind_cnt2 = if wc2_2 == 0 { 1 } else { 0 };
        }

        let fillpos = FillRule::Positive;
        match self.fillrule {
            FillRule::EvenOdd | FillRule::NonZero => {
                old_e1_windcnt = self.active_arena[e1_idx].wind_cnt.abs();
                old_e2_windcnt = self.active_arena[e2_idx].wind_cnt.abs();
            }
            _ => {
                if self.fillrule == fillpos {
                    old_e1_windcnt = self.active_arena[e1_idx].wind_cnt;
                    old_e2_windcnt = self.active_arena[e2_idx].wind_cnt;
                } else {
                    old_e1_windcnt = -self.active_arena[e1_idx].wind_cnt;
                    old_e2_windcnt = -self.active_arena[e2_idx].wind_cnt;
                }
            }
        }

        let e1_wc_in_01 = old_e1_windcnt == 0 || old_e1_windcnt == 1;
        let e2_wc_in_01 = old_e2_windcnt == 0 || old_e2_windcnt == 1;

        if (!is_hot_edge(&self.active_arena[e1_idx]) && !e1_wc_in_01)
            || (!is_hot_edge(&self.active_arena[e2_idx]) && !e2_wc_in_01)
        {
            return;
        }

        // NOW PROCESS THE INTERSECTION
        let mut _result_op: Option<usize> = None;
        if is_hot_edge(&self.active_arena[e1_idx]) && is_hot_edge(&self.active_arena[e2_idx]) {
            if (old_e1_windcnt != 0 && old_e1_windcnt != 1)
                || (old_e2_windcnt != 0 && old_e2_windcnt != 1)
                || (!self.is_same_poly_type_idx(e1_idx, e2_idx) && self.cliptype != ClipType::Xor)
            {
                _result_op = self.add_local_max_poly(e1_idx, e2_idx, pt);

                if self.z_callback.is_some() {
                    if let Some(res_op) = _result_op {
                        self.set_z(e1_idx, e2_idx, res_op);
                    }
                }
            } else if self.is_front(e1_idx)
                || self.active_arena[e1_idx].outrec == self.active_arena[e2_idx].outrec
            {
                _result_op = self.add_local_max_poly(e1_idx, e2_idx, pt);
                let _op2 = self.add_local_min_poly(e1_idx, e2_idx, pt, false);

                if self.z_callback.is_some() {
                    if let Some(res_op) = _result_op {
                        self.set_z(e1_idx, e2_idx, res_op);
                    }
                    self.set_z(e1_idx, e2_idx, _op2);
                }
            } else {
                _result_op = Some(self.add_out_pt(e1_idx, pt));
                let _op2 = self.add_out_pt(e2_idx, pt);

                if self.z_callback.is_some() {
                    self.set_z(e1_idx, e2_idx, _result_op.unwrap());
                    self.set_z(e1_idx, e2_idx, _op2);
                }
                self.swap_outrecs(e1_idx, e2_idx);
            }
        } else if is_hot_edge(&self.active_arena[e1_idx]) {
            _result_op = Some(self.add_out_pt(e1_idx, pt));

            if self.z_callback.is_some() {
                self.set_z(e1_idx, e2_idx, _result_op.unwrap());
            }
            self.swap_outrecs(e1_idx, e2_idx);
        } else if is_hot_edge(&self.active_arena[e2_idx]) {
            _result_op = Some(self.add_out_pt(e2_idx, pt));

            if self.z_callback.is_some() {
                self.set_z(e1_idx, e2_idx, _result_op.unwrap());
            }
            self.swap_outrecs(e1_idx, e2_idx);
        } else {
            // neither edge is hot
            let e1wc2;
            let e2wc2;
            match self.fillrule {
                FillRule::EvenOdd | FillRule::NonZero => {
                    e1wc2 = self.active_arena[e1_idx].wind_cnt2.abs() as i64;
                    e2wc2 = self.active_arena[e2_idx].wind_cnt2.abs() as i64;
                }
                _ => {
                    if self.fillrule == fillpos {
                        e1wc2 = self.active_arena[e1_idx].wind_cnt2 as i64;
                        e2wc2 = self.active_arena[e2_idx].wind_cnt2 as i64;
                    } else {
                        e1wc2 = -(self.active_arena[e1_idx].wind_cnt2 as i64);
                        e2wc2 = -(self.active_arena[e2_idx].wind_cnt2 as i64);
                    }
                }
            }

            if !self.is_same_poly_type_idx(e1_idx, e2_idx) {
                _result_op = Some(self.add_local_min_poly(e1_idx, e2_idx, pt, false));

                if self.z_callback.is_some() {
                    self.set_z(e1_idx, e2_idx, _result_op.unwrap());
                }
            } else if old_e1_windcnt == 1 && old_e2_windcnt == 1 {
                _result_op = None;
                match self.cliptype {
                    ClipType::Union => {
                        if e1wc2 <= 0 && e2wc2 <= 0 {
                            _result_op = Some(self.add_local_min_poly(e1_idx, e2_idx, pt, false));
                        }
                    }
                    ClipType::Difference => {
                        if (self.get_poly_type_idx(e1_idx) == PathType::Clip
                            && e1wc2 > 0
                            && e2wc2 > 0)
                            || (self.get_poly_type_idx(e1_idx) == PathType::Subject
                                && e1wc2 <= 0
                                && e2wc2 <= 0)
                        {
                            _result_op = Some(self.add_local_min_poly(e1_idx, e2_idx, pt, false));
                        }
                    }
                    ClipType::Xor => {
                        _result_op = Some(self.add_local_min_poly(e1_idx, e2_idx, pt, false));
                    }
                    _ => {
                        // Intersection
                        if e1wc2 > 0 && e2wc2 > 0 {
                            _result_op = Some(self.add_local_min_poly(e1_idx, e2_idx, pt, false));
                        }
                    }
                }

                if let Some(result_op) = _result_op {
                    if self.z_callback.is_some() {
                        self.set_z(e1_idx, e2_idx, result_op);
                    }
                }
            }
        }
    }
}
