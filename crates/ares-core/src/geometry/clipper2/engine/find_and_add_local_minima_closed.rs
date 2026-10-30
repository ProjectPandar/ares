// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

impl ClipperBase {
    /// Find and add local minima for a closed path
    pub(crate) fn find_and_add_local_minima_closed(
        &mut self,
        first_vert_idx: usize,
        vert_count: usize,
        polytype: PathType,
    ) {
        // Find all local minima in the closed path
        // A local minimum is where the path changes from descending to ascending
        // (in screen coordinates where Y increases downward)

        // First, find any vertex that is NOT a local minimum to start from
        // We need to find a vertex where prev.y != curr.y to know direction
        let mut start = first_vert_idx;
        let end = first_vert_idx + vert_count;

        // Skip vertices where prev has same y
        let mut found_start = false;
        for i in first_vert_idx..end {
            let prev = self.vertex_arena[i].prev;
            if self.vertex_arena[prev].pt.y != self.vertex_arena[i].pt.y {
                start = i;
                found_start = true;
                break;
            }
        }
        if !found_start {
            return; // All vertices at same y - degenerate
        }

        // Determine initial direction
        let prev = self.vertex_arena[start].prev;
        let mut going_up = self.vertex_arena[prev].pt.y > self.vertex_arena[start].pt.y;

        let mut curr = start;
        loop {
            let next = self.vertex_arena[curr].next;
            let curr_pt = self.vertex_arena[curr].pt;
            let next_pt = self.vertex_arena[next].pt;

            if next_pt.y > curr_pt.y && going_up {
                // Was going up, now going down => local max
                self.vertex_arena[curr].flags |= VertexFlags::LOCAL_MAX;
                going_up = false;
            } else if next_pt.y < curr_pt.y && !going_up {
                // Was going down, now going up => local min
                going_up = true;
                self.add_loc_min(curr, polytype, false);
            }

            // Skip horizontal edges (same y)
            if next_pt.y == curr_pt.y && next != start {
                curr = next;
                continue;
            }

            curr = next;
            if curr == start {
                break;
            }
        }
    }
}

impl ClipperBase {
    /// Find and add local minima for an open path
    pub(crate) fn find_and_add_local_minima_open(
        &mut self,
        first_vert_idx: usize,
        vert_count: usize,
        polytype: PathType,
    ) {
        let last_vert_idx = first_vert_idx + vert_count - 1;

        // For open paths, the first and last vertices are potential local minima
        let first_pt = self.vertex_arena[first_vert_idx].pt;
        let second_pt = self.vertex_arena[first_vert_idx + 1].pt;

        // The start is a local min if it goes up (in screen coords, y decreases)
        // or if horizontal then the next segment goes up
        let mut going_up;
        if first_pt.y > second_pt.y {
            // Going up from first vertex - first is a local min
            self.add_loc_min(first_vert_idx, polytype, true);
            going_up = true;
        } else if first_pt.y < second_pt.y {
            // Going down - first is NOT a local min, it's a local max
            self.vertex_arena[first_vert_idx].flags |= VertexFlags::LOCAL_MAX;
            going_up = false;
        } else {
            // Horizontal at start - need to keep scanning
            let mut i = first_vert_idx + 1;
            while i < last_vert_idx && self.vertex_arena[i].pt.y == first_pt.y {
                i += 1;
            }
            if i >= last_vert_idx {
                // Completely horizontal open path - still add as local minimum
                // so it participates in the sweep line (matches C++ Clipper2 behavior)
                self.add_loc_min(first_vert_idx, polytype, true);
                going_up = true;
            } else {
                going_up = self.vertex_arena[i].pt.y < first_pt.y;
                if going_up {
                    self.add_loc_min(first_vert_idx, polytype, true);
                } else {
                    self.vertex_arena[first_vert_idx].flags |= VertexFlags::LOCAL_MAX;
                }
            }
        }

        // Scan interior vertices
        for i in (first_vert_idx + 1)..last_vert_idx {
            let curr_pt = self.vertex_arena[i].pt;
            let next_pt = self.vertex_arena[i + 1].pt;

            if next_pt.y > curr_pt.y && going_up {
                self.vertex_arena[i].flags |= VertexFlags::LOCAL_MAX;
                going_up = false;
            } else if next_pt.y < curr_pt.y && !going_up {
                going_up = true;
                self.add_loc_min(i, polytype, true);
            }
        }

        // Handle last vertex
        let last_pt = self.vertex_arena[last_vert_idx].pt;
        let prev_pt = self.vertex_arena[last_vert_idx - 1].pt;
        if !going_up {
            // Was descending - last vertex is a local min
            self.add_loc_min(last_vert_idx, polytype, true);
        } else if last_pt.y < prev_pt.y || (last_pt.y == prev_pt.y && going_up) {
            self.vertex_arena[last_vert_idx].flags |= VertexFlags::LOCAL_MAX;
        }
    }
}

impl ClipperBase {
    /// Add multiple paths for clipping
    /// Direct port from clipper.engine.h AddPaths
    pub fn add_paths(&mut self, paths: &Paths64, polytype: PathType, is_open: bool) {
        for path in paths {
            self.add_path(path, polytype, is_open);
        }
    }
}

impl ClipperBase {
    /// Sort the local minima list
    pub fn sort_minima_list(&mut self) {
        if !self.minima_list_sorted {
            // Sort by y descending (bottom first), then by x ascending
            let vertex_arena = &self.vertex_arena;
            self.minima_list.sort_by(|a, b| {
                let a_pt = vertex_arena[a.vertex].pt;
                let b_pt = vertex_arena[b.vertex].pt;
                if a_pt.y != b_pt.y {
                    b_pt.y.cmp(&a_pt.y) // descending by y (larger y = lower on screen)
                } else {
                    a_pt.x.cmp(&b_pt.x) // ascending by x
                }
            });
            self.minima_list_sorted = true;
        }
    }
}

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
        if let Some(pts_idx) = self.outrec_list[outrec_idx].pts {
            // Unlink the circular list (don't actually free, arena-based)
            let prev = self.outpt_arena[pts_idx].prev;
            self.outpt_arena[prev].next = NONE; // break the circle
            // Walk and mark as disposed
            let mut op = Some(pts_idx);
            while let Some(idx) = op {
                if self.outpt_arena[idx].next == NONE {
                    break;
                }
                let next = self.outpt_arena[idx].next;
                if next == NONE || next == idx {
                    break;
                }
                op = Some(next);
            }
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
