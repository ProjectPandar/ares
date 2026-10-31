// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
use super::*;

impl ClipperBase {
    // clipper.engine.cpp:1809–2123, unchanged open/closed dispatch and event order.
    pub(crate) fn intersect_edges(&mut self, first: usize, second: usize, point: Point64) {
        if self.has_open_paths && (self.is_open_idx(first) || self.is_open_idx(second)) {
            self.intersect_open_edges(first, second, point);
            return;
        }
        if is_joined(&self.active_arena[first]) {
            self.split(first, point);
        }
        if is_joined(&self.active_arena[second]) {
            self.split(second, point);
        }
        self.update_intersection_winding(first, second);
        let counts = self.normalized_winding((
            self.active_arena[first].wind_cnt,
            self.active_arena[second].wind_cnt,
        ));
        let first_in_01 = counts.0 == 0 || counts.0 == 1;
        let second_in_01 = counts.1 == 0 || counts.1 == 1;
        if (!is_hot_edge(&self.active_arena[first]) && !first_in_01)
            || (!is_hot_edge(&self.active_arena[second]) && !second_in_01)
        {
            return;
        }
        if is_hot_edge(&self.active_arena[first]) && is_hot_edge(&self.active_arena[second]) {
            self.intersect_hot_edges(first, second, point, counts);
        } else if is_hot_edge(&self.active_arena[first]) {
            self.intersect_single_hot_edge(first, second, point, first);
        } else if is_hot_edge(&self.active_arena[second]) {
            self.intersect_single_hot_edge(first, second, point, second);
        } else {
            self.intersect_cold_edges(first, second, point, counts);
        }
    }

    // IntersectEdges:1911–1940. Both edge writes retain their original order.
    fn update_intersection_winding(&mut self, first: usize, second: usize) {
        if self.is_same_poly_type_idx(first, second) {
            if self.fillrule == FillRule::EvenOdd {
                let count = self.active_arena[first].wind_cnt;
                self.active_arena[first].wind_cnt = self.active_arena[second].wind_cnt;
                self.active_arena[second].wind_cnt = count;
            } else {
                self.update_same_type_winding(first, second);
            }
        } else if self.fillrule != FillRule::EvenOdd {
            self.active_arena[first].wind_cnt2 += self.active_arena[second].wind_dx;
            self.active_arena[second].wind_cnt2 -= self.active_arena[first].wind_dx;
        } else {
            self.active_arena[first].wind_cnt2 = i32::from(self.active_arena[first].wind_cnt2 == 0);
            self.active_arena[second].wind_cnt2 =
                i32::from(self.active_arena[second].wind_cnt2 == 0);
        }
    }
    fn update_same_type_winding(&mut self, first: usize, second: usize) {
        let first_count = self.active_arena[first].wind_cnt;
        let second_direction = self.active_arena[second].wind_dx;
        let first_direction = self.active_arena[first].wind_dx;
        self.active_arena[first].wind_cnt = if first_count + second_direction == 0 {
            -first_count
        } else {
            first_count + second_direction
        };
        let second_count = self.active_arena[second].wind_cnt;
        self.active_arena[second].wind_cnt = if second_count - first_direction == 0 {
            -second_count
        } else {
            second_count - first_direction
        };
    }
    // IntersectEdges:1942–1963,2044–2063, the two identical fill-rule projections.
    fn normalized_winding(&self, (first, second): (i32, i32)) -> (i32, i32) {
        match self.fillrule {
            FillRule::EvenOdd | FillRule::NonZero => (first.abs(), second.abs()),
            FillRule::Positive => (first, second),
            FillRule::Negative => (-first, -second),
        }
    }

    // IntersectEdges:1975–2025, both contributing edges.
    fn intersect_hot_edges(
        &mut self,
        first: usize,
        second: usize,
        point: Point64,
        counts: (i32, i32),
    ) {
        if (counts.0 != 0 && counts.0 != 1)
            || (counts.1 != 0 && counts.1 != 1)
            || (!self.is_same_poly_type_idx(first, second) && self.cliptype != ClipType::Xor)
        {
            let result = self.add_local_max_poly(first, second, point);
            self.set_intersection_z(first, second, result);
            return;
        }
        if self.is_front(first)
            || self.active_arena[first].outrec == self.active_arena[second].outrec
        {
            let result = self.add_local_max_poly(first, second, point);
            let minimum = self.add_local_min_poly(first, second, point, false);
            self.set_intersection_z(first, second, result);
            if self.z_callback.is_some() {
                self.set_z(first, second, minimum);
            }
            return;
        }
        let result = self.add_out_pt(first, point);
        let other = self.add_out_pt(second, point);
        if self.z_callback.is_some() {
            self.set_z(first, second, result);
            self.set_z(first, second, other);
        }
        self.swap_outrecs(first, second);
    }
    fn set_intersection_z(&mut self, first: usize, second: usize, point: Option<usize>) {
        if self.z_callback.is_some()
            && let Some(point) = point
        {
            self.set_z(first, second, point);
        }
    }
    fn intersect_single_hot_edge(
        &mut self,
        first: usize,
        second: usize,
        point: Point64,
        hot: usize,
    ) {
        let result = self.add_out_pt(hot, point);
        if self.z_callback.is_some() {
            self.set_z(first, second, result);
        }
        self.swap_outrecs(first, second);
    }

    // IntersectEdges:2042–2123, neither edge contributes before this crossing.
    fn intersect_cold_edges(
        &mut self,
        first: usize,
        second: usize,
        point: Point64,
        counts: (i32, i32),
    ) {
        let counts2 = self.normalized_winding((
            self.active_arena[first].wind_cnt2,
            self.active_arena[second].wind_cnt2,
        ));
        let (first_count2, second_count2) = (i64::from(counts2.0), i64::from(counts2.1));
        if !self.is_same_poly_type_idx(first, second) {
            let result = self.add_local_min_poly(first, second, point, false);
            if self.z_callback.is_some() {
                self.set_z(first, second, result);
            }
            return;
        }
        if counts.0 != 1 || counts.1 != 1 {
            return;
        }
        let contributes = match self.cliptype {
            ClipType::Union => first_count2 <= 0 && second_count2 <= 0,
            ClipType::Difference => {
                (self.get_poly_type_idx(first) == PathType::Clip
                    && first_count2 > 0
                    && second_count2 > 0)
                    || (self.get_poly_type_idx(first) == PathType::Subject
                        && first_count2 <= 0
                        && second_count2 <= 0)
            }
            ClipType::Xor => true,
            ClipType::NoClip | ClipType::Intersection => first_count2 > 0 && second_count2 > 0,
        };
        if contributes {
            let result = self.add_local_min_poly(first, second, point, false);
            if self.z_callback.is_some() {
                self.set_z(first, second, result);
            }
        }
    }
}
