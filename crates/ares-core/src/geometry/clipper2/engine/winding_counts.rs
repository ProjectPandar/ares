// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.
// clipper.engine.cpp:1037–1148, SetWindCountForClosedPathEdge/OpenPathEdge.
use super::*;

impl ClipperBase {
    pub(crate) fn set_wind_count_for_closed_path_edge(&mut self, e_idx: usize) {
        let polytype = self.get_poly_type_idx(e_idx);
        let mut previous = self.active_arena[e_idx].prev_in_ael;
        while let Some(index) = previous {
            if self.get_poly_type_idx(index) == polytype && !self.is_open_idx(index) {
                break;
            }
            previous = self.active_arena[index].prev_in_ael;
        }
        let current = match previous {
            None => {
                self.active_arena[e_idx].wind_cnt = self.active_arena[e_idx].wind_dx;
                self.actives
            }
            Some(index) if self.fillrule == FillRule::EvenOdd => {
                self.active_arena[e_idx].wind_cnt = self.active_arena[e_idx].wind_dx;
                self.active_arena[e_idx].wind_cnt2 = self.active_arena[index].wind_cnt2;
                self.active_arena[index].next_in_ael
            }
            Some(index) => {
                self.active_arena[e_idx].wind_cnt = self.closed_wind_count(index, e_idx);
                self.active_arena[e_idx].wind_cnt2 = self.active_arena[index].wind_cnt2;
                self.active_arena[index].next_in_ael
            }
        };
        if self.fillrule == FillRule::EvenOdd {
            self.toggle_closed_count2(e_idx, current, polytype);
        } else {
            self.accumulate_closed_count2(e_idx, current, polytype);
        }
    }

    // clipper.engine.cpp:1063–1093: the outside-all-polygons case finishes
    // before the shared inside/outside direction test. No AEL mutation occurs here.
    fn closed_wind_count(&self, previous: usize, edge: usize) -> i32 {
        let count = self.active_arena[previous].wind_cnt;
        let previous_dx = self.active_arena[previous].wind_dx;
        let dx = self.active_arena[edge].wind_dx;
        let outside_all = count * previous_dx < 0 && count.abs() <= 1;
        if outside_all && self.is_open_idx(edge) {
            return 1;
        }
        if outside_all {
            return dx;
        }
        if previous_dx * dx < 0 {
            count
        } else {
            count + dx
        }
    }

    pub(crate) fn set_wind_count_for_open_path_edge(&mut self, edge: usize) {
        if self.fillrule == FillRule::EvenOdd {
            let (subject, clip) = self.count_open_crossings(edge);
            self.active_arena[edge].wind_cnt = i32::from(is_odd(subject));
            self.active_arena[edge].wind_cnt2 = i32::from(is_odd(clip));
        } else {
            self.accumulate_open_crossings(edge);
        }
    }

    // SetWindCountForClosedPathEdge:1096–1112, the source's two separate loops.
    fn toggle_closed_count2(
        &mut self,
        edge: usize,
        mut current: Option<usize>,
        polytype: PathType,
    ) {
        while current != Some(edge) {
            let index = current.unwrap();
            if self.get_poly_type_idx(index) != polytype && !self.is_open_idx(index) {
                let count = self.active_arena[edge].wind_cnt2;
                self.active_arena[edge].wind_cnt2 = i32::from(count == 0);
            }
            current = self.active_arena[index].next_in_ael;
        }
    }
    fn accumulate_closed_count2(
        &mut self,
        edge: usize,
        mut current: Option<usize>,
        polytype: PathType,
    ) {
        while current != Some(edge) {
            let index = current.unwrap();
            if self.get_poly_type_idx(index) != polytype && !self.is_open_idx(index) {
                self.active_arena[edge].wind_cnt2 += self.active_arena[index].wind_dx;
            }
            current = self.active_arena[index].next_in_ael;
        }
    }
    // SetWindCountForOpenPathEdge:1117–1147, same visit and write order.
    fn count_open_crossings(&self, edge: usize) -> (i32, i32) {
        let mut current = self.actives;
        let (mut subject, mut clip) = (0, 0);
        while current != Some(edge) {
            let index = current.unwrap();
            if self.get_poly_type_idx(index) == PathType::Clip {
                clip += 1;
            } else if !self.is_open_idx(index) {
                subject += 1;
            }
            current = self.active_arena[index].next_in_ael;
        }
        (subject, clip)
    }
    fn accumulate_open_crossings(&mut self, edge: usize) {
        let mut current = self.actives;
        while current != Some(edge) {
            let index = current.unwrap();
            if self.get_poly_type_idx(index) == PathType::Clip {
                self.active_arena[edge].wind_cnt2 += self.active_arena[index].wind_dx;
            } else if !self.is_open_idx(index) {
                self.active_arena[edge].wind_cnt += self.active_arena[index].wind_dx;
            }
            current = self.active_arena[index].next_in_ael;
        }
    }
}
