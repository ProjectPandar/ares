// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Add a trial horizontal join
    /// Direct port from clipper.engine.cpp AddTrialHorzJoin
    #[inline]
    pub fn add_trial_horz_join(&mut self, op_idx: usize) {
        // Match C++: check is_open, then just push to the list.
        // Do NOT check or set op.horz here - that is only done inside
        // convert_horz_segs_to_joins (via UpdateHorzSegment logic).
        let or_idx = self.outpt_arena[op_idx].outrec;
        if self.outrec_list[or_idx].is_open {
            return;
        }
        self.horz_seg_list.push(HorzSegment::with_op(op_idx));
    }
}

impl ClipperBase {
    /// Push a horizontal edge onto the horz stack
    /// Direct port from clipper.engine.cpp PushHorz
    #[inline]
    pub fn push_horz(&mut self, e_idx: usize) {
        self.active_arena[e_idx].next_in_sel = self.sel;
        self.sel = Some(e_idx);
    }
}

impl ClipperBase {
    /// Pop a horizontal edge from the horz stack
    /// Direct port from clipper.engine.cpp PopHorz
    #[inline]
    pub fn pop_horz(&mut self) -> Option<usize> {
        let e = self.sel;
        if let Some(e_idx) = e {
            self.sel = self.active_arena[e_idx].next_in_sel;
        }
        e
    }
}

impl ClipperBase {
    /// Build Path64 output from OutRec
    /// Direct port from clipper.engine.cpp BuildPath64
    pub fn build_path64(&self, outrec: &OutRec) -> Option<Path64> {
        let op_start = outrec.pts?;
        let cnt = point_count(op_start, &self.outpt_arena);
        if cnt < 2 {
            return None;
        }

        let reverse = if outrec.is_open {
            false
        } else {
            let area = area_outpt(op_start, &self.outpt_arena);
            if area == 0.0 {
                return None;
            }
            (area < 0.0) != self.reverse_solution
        };

        let mut result = Path64::with_capacity(cnt as usize);
        if reverse {
            let mut op = op_start;
            loop {
                result.push(self.outpt_arena[op].pt);
                op = self.outpt_arena[op].prev;
                if op == op_start {
                    break;
                }
            }
        } else {
            let op_next = self.outpt_arena[op_start].next;
            let mut op = op_next;
            loop {
                result.push(self.outpt_arena[op].pt);
                op = self.outpt_arena[op].next;
                if op == op_next {
                    break;
                }
            }
        }

        // Remove collinear if not preserving
        if !self.preserve_collinear {
            // Strip collinear points
            let mut i = 0;
            while i < result.len() && result.len() > 2 {
                let prev = if i == 0 { result.len() - 1 } else { i - 1 };
                let next = (i + 1) % result.len();
                if is_collinear(result[prev], result[i], result[next]) {
                    result.remove(i);
                    i = i.saturating_sub(1);
                } else {
                    i += 1;
                }
            }
        }

        if result.len() < 2 { None } else { Some(result) }
    }
}

impl Default for ClipperBase {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipperBase {
    /// Check if an active edge is part of an open path
    #[inline]
    pub(crate) fn is_open_idx(&self, e_idx: usize) -> bool {
        self.minima_list[self.active_arena[e_idx].local_min].is_open
    }
}

impl ClipperBase {
    /// Check if an active edge is at a local maximum
    #[inline]
    pub(crate) fn is_maxima_idx(&self, e_idx: usize) -> bool {
        is_maxima_vertex(&self.vertex_arena[self.active_arena[e_idx].vertex_top])
    }
}

impl ClipperBase {
    /// Check if an active edge is horizontal
    #[inline]
    pub(crate) fn is_horizontal_idx(&self, e_idx: usize) -> bool {
        self.active_arena[e_idx].top.y == self.active_arena[e_idx].bot.y
    }
}

impl ClipperBase {
    /// Check if an active edge is the open end
    #[inline]
    pub(crate) fn is_open_end_idx(&self, e_idx: usize) -> bool {
        is_open_end_vertex(&self.vertex_arena[self.active_arena[e_idx].vertex_top])
    }
}

impl ClipperBase {
    /// Get polytype for an active edge
    #[inline]
    pub(crate) fn get_poly_type_idx(&self, e_idx: usize) -> PathType {
        self.minima_list[self.active_arena[e_idx].local_min].polytype
    }
}

impl ClipperBase {
    /// Check if two edges have same polytype
    #[inline]
    pub(crate) fn is_same_poly_type_idx(&self, e1: usize, e2: usize) -> bool {
        self.minima_list[self.active_arena[e1].local_min].polytype
            == self.minima_list[self.active_arena[e2].local_min].polytype
    }
}

impl ClipperBase {
    /// Get next vertex index for an active
    #[inline]
    pub(crate) fn next_vertex_idx(&self, e_idx: usize) -> usize {
        if self.active_arena[e_idx].wind_dx > 0 {
            self.vertex_arena[self.active_arena[e_idx].vertex_top].next
        } else {
            self.vertex_arena[self.active_arena[e_idx].vertex_top].prev
        }
    }
}

impl ClipperBase {
    /// Reset for a new execution
    /// Direct port from clipper.engine.cpp Reset (line 786)
    pub fn reset(&mut self) {
        self.sort_minima_list();
        // Insert all local minima y-values as scanlines
        for i in (0..self.minima_list.len()).rev() {
            let y = self.vertex_arena[self.minima_list[i].vertex].pt.y;
            self.insert_scanline(y);
        }
        self.current_locmin_idx = 0;
        self.actives = None;
        self.sel = None;
        self.succeeded = true;
    }
}

impl ClipperBase {
    /// Assign z to the out-point at ip_idx (inheriting from any matching edge
    /// vertex, else default_z), then invoke the user callback
    /// Direct port from clipper.engine.cpp line 805 (ClipperBase::SetZ)

    pub(crate) fn set_z(&mut self, e1_idx: usize, e2_idx: usize, ip_idx: usize) {
        if self.z_callback.is_none() {
            return;
        }
        let e1 = &self.active_arena[e1_idx];
        let e2 = &self.active_arena[e2_idx];
        // prioritize subject over clip vertices by passing
        // subject vertices before clip vertices in the callback
        if self.get_poly_type_idx(e1_idx) == PathType::Subject {
            let ip = &mut self.outpt_arena[ip_idx].pt;
            if *ip == e1.bot {
                ip.z = e1.bot.z;
            } else if *ip == e1.top {
                ip.z = e1.top.z;
            } else if *ip == e2.bot {
                ip.z = e2.bot.z;
            } else if *ip == e2.top {
                ip.z = e2.top.z;
            } else {
                ip.z = self.default_z;
            }
            self.z_callback.as_mut().unwrap()(&e1.bot, &e1.top, &e2.bot, &e2.top, ip);
        } else {
            let ip = &mut self.outpt_arena[ip_idx].pt;
            if *ip == e2.bot {
                ip.z = e2.bot.z;
            } else if *ip == e2.top {
                ip.z = e2.top.z;
            } else if *ip == e1.bot {
                ip.z = e1.bot.z;
            } else if *ip == e1.top {
                ip.z = e1.top.z;
            } else {
                ip.z = self.default_z;
            }
            self.z_callback.as_mut().unwrap()(&e2.bot, &e2.top, &e1.bot, &e1.top, ip);
        }
    }
}

impl ClipperBase {
    /// Clean up after execution
    pub fn clean_up(&mut self) {
        // In arena-based approach, just clear the runtime state
        self.active_arena.clear();
        self.scanline_list.clear();
        self.intersect_nodes.clear();
        // Dispose all output
        for i in 0..self.outrec_list.len() {
            self.outrec_list[i].pts = None;
        }
        self.outpt_arena.clear();
        self.outrec_list.clear();
        self.horz_seg_list.clear();
        self.horz_join_list.clear();
        self.actives = None;
        self.sel = None;
    }
}

impl ClipperBase {
    /// Check if a closed path edge is contributing to the solution
    /// Direct port from clipper.engine.cpp IsContributingClosed (line 908)
    pub(crate) fn is_contributing_closed(&self, e_idx: usize) -> bool {
        let e = &self.active_arena[e_idx];
        match self.fillrule {
            FillRule::EvenOdd => {}
            FillRule::NonZero => {
                if e.wind_cnt.abs() != 1 {
                    return false;
                }
            }
            FillRule::Positive => {
                if e.wind_cnt != 1 {
                    return false;
                }
            }
            FillRule::Negative => {
                if e.wind_cnt != -1 {
                    return false;
                }
            }
        }

        match self.cliptype {
            ClipType::NoClip => false,
            ClipType::Intersection => match self.fillrule {
                FillRule::Positive => e.wind_cnt2 > 0,
                FillRule::Negative => e.wind_cnt2 < 0,
                _ => e.wind_cnt2 != 0,
            },
            ClipType::Union => match self.fillrule {
                FillRule::Positive => e.wind_cnt2 <= 0,
                FillRule::Negative => e.wind_cnt2 >= 0,
                _ => e.wind_cnt2 == 0,
            },
            ClipType::Difference => {
                let result = match self.fillrule {
                    FillRule::Positive => e.wind_cnt2 <= 0,
                    FillRule::Negative => e.wind_cnt2 >= 0,
                    _ => e.wind_cnt2 == 0,
                };
                if self.get_poly_type_idx(e_idx) == PathType::Subject {
                    result
                } else {
                    !result
                }
            }
            ClipType::Xor => true,
        }
    }
}
