// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:1811–1903, open-path IntersectEdges branch.
    pub(super) fn intersect_open_edges(&mut self, first: usize, second: usize, point: Point64) {
        if self.is_open_idx(first) && self.is_open_idx(second) {
            return;
        }
        let (open, closed) = if self.is_open_idx(first) {
            (first, second)
        } else {
            (second, first)
        };
        if is_joined(&self.active_arena[closed]) {
            self.split(closed, point);
        }
        if self.active_arena[closed].wind_cnt.abs() != 1 {
            return;
        }
        let contributes = match self.cliptype {
            ClipType::Union => is_hot_edge(&self.active_arena[closed]),
            ClipType::NoClip | ClipType::Intersection | ClipType::Difference | ClipType::Xor => {
                self.get_poly_type_idx(closed) != PathType::Subject
            }
        };
        if !contributes {
            return;
        }
        let fills = match self.fillrule {
            FillRule::Positive => self.active_arena[closed].wind_cnt == 1,
            FillRule::Negative => self.active_arena[closed].wind_cnt == -1,
            FillRule::EvenOdd | FillRule::NonZero => self.active_arena[closed].wind_cnt.abs() == 1,
        };
        if !fills {
            return;
        }
        let result = if is_hot_edge(&self.active_arena[open]) {
            let point_index = self.add_out_pt(open, point);
            let record = self.active_arena[open].outrec.unwrap();
            if self.is_front(open) {
                self.outrec_list[record].front_edge = None;
            } else {
                self.outrec_list[record].back_edge = None;
            }
            self.active_arena[open].outrec = None;
            point_index
        } else {
            if self.try_join_open_minimum(open, point) {
                return;
            }
            self.start_open_path(open, point)
        };
        if self.z_callback.is_some() {
            self.set_z(open, closed, result);
        }
    }

    // IntersectEdges:1867–1885. The pinned predicate uses the local-minimum
    // vertex, without the seed-only requirement that it also equal edge.bot.
    fn try_join_open_minimum(&mut self, edge: usize, point: Point64) -> bool {
        let vertex = self.minima_list[self.active_arena[edge].local_min].vertex;
        if point != self.vertex_arena[vertex].pt || is_open_end_vertex(&self.vertex_arena[vertex]) {
            return false;
        }
        let Some(other) = self.find_edge_with_matching_loc_min(edge) else {
            return false;
        };
        if !is_hot_edge(&self.active_arena[other]) {
            return false;
        }
        let record = self.active_arena[other].outrec.unwrap();
        self.active_arena[edge].outrec = Some(record);
        if self.active_arena[edge].wind_dx > 0 {
            self.set_sides(record, edge, other);
        } else {
            self.set_sides(record, other, edge);
        }
        true
    }
}
