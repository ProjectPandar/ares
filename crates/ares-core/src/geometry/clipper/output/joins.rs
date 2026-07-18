use super::super::ClosedClipper;
use super::super::types::{Join, OutRecId};

struct HoleState {
    owner: OutRecId,
    is_hole: bool,
    first_left: Option<OutRecId>,
}

impl ClosedClipper {
    pub(in crate::geometry::clipper) fn join_common_edges(&mut self) {
        for mut join in std::mem::take(&mut self.joins) {
            let first_rec = self.join_owner(join.first);
            let second_rec = self.join_owner(join.second);
            if self.out_recs[first_rec.0].points.is_none()
                || self.out_recs[second_rec.0].points.is_none()
            {
                continue;
            }

            let hole_state = if first_rec == second_rec {
                first_rec
            } else if self.out_rec_right_of(first_rec, second_rec) {
                second_rec
            } else if self.out_rec_right_of(second_rec, first_rec) {
                first_rec
            } else {
                self.get_lowermost_rec(first_rec, second_rec)
            };
            let hole_state = HoleState {
                owner: hole_state,
                is_hole: self.out_recs[hole_state.0].is_hole,
                first_left: self.out_recs[hole_state.0].first_left,
            };

            if !self.join_points(&mut join, first_rec, second_rec) {
                continue;
            }
            if first_rec == second_rec {
                self.finish_split_join(join, first_rec);
            } else {
                self.finish_merged_join(first_rec, second_rec, hole_state);
            }
        }
    }

    fn join_owner(&self, point: super::super::types::OutPointId) -> OutRecId {
        self.get_out_rec(self.out_points.point(point).out_rec)
    }

    fn finish_split_join(&mut self, join: Join, first_rec: OutRecId) {
        self.out_recs[first_rec.0].points = Some(join.first);
        self.out_recs[first_rec.0].bottom_point = None;

        let second_rec = self.create_out_rec();
        self.out_recs[second_rec.0].points = Some(join.second);
        self.update_out_point_indices(second_rec);

        if self.poly_contains(join.second, join.first) {
            self.out_recs[second_rec.0].is_hole = !self.out_recs[first_rec.0].is_hole;
            self.out_recs[second_rec.0].first_left = Some(first_rec);
            if self.using_polytree {
                self.fixup_first_lefts2(second_rec, first_rec);
            }
            self.fix_split_orientation(second_rec);
        } else if self.poly_contains(join.first, join.second) {
            self.out_recs[second_rec.0].is_hole = self.out_recs[first_rec.0].is_hole;
            self.out_recs[first_rec.0].is_hole = !self.out_recs[second_rec.0].is_hole;
            self.out_recs[second_rec.0].first_left = self.out_recs[first_rec.0].first_left;
            self.out_recs[first_rec.0].first_left = Some(second_rec);
            if self.using_polytree {
                self.fixup_first_lefts2(first_rec, second_rec);
            }
            self.fix_split_orientation(first_rec);
        } else {
            self.out_recs[second_rec.0].is_hole = self.out_recs[first_rec.0].is_hole;
            self.out_recs[second_rec.0].first_left = self.out_recs[first_rec.0].first_left;
            if self.using_polytree {
                self.fixup_first_lefts1(first_rec, second_rec);
            }
        }
    }

    fn fix_split_orientation(&mut self, out_rec: OutRecId) {
        let points = self.out_recs[out_rec.0]
            .points
            .expect("split output record has points");
        let reverse = (self.out_recs[out_rec.0].is_hole ^ self.options.reverse_solution)
            == (self.out_ring_area(points) > 0.0);
        if reverse {
            self.reverse_out_ring(points);
        }
    }

    fn finish_merged_join(
        &mut self,
        first_rec: OutRecId,
        second_rec: OutRecId,
        hole_state: HoleState,
    ) {
        self.out_recs[second_rec.0].points = None;
        self.out_recs[second_rec.0].bottom_point = None;
        self.replace_out_rec_root(second_rec, first_rec);

        self.out_recs[first_rec.0].is_hole = hole_state.is_hole;
        if hole_state.owner == second_rec {
            self.out_recs[first_rec.0].first_left = hole_state.first_left;
        }
        self.out_recs[second_rec.0].first_left = Some(first_rec);
        if self.using_polytree {
            self.fixup_first_lefts3(second_rec, first_rec);
        }
    }
}
