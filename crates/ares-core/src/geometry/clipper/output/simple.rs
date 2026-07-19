use super::super::ClosedClipper;
#[cfg(test)]
use super::super::SimpleRepair;
use super::super::types::{OutPointId, OutRecId};

impl ClosedClipper {
    pub(in crate::geometry::clipper) fn do_simple_polygons(&mut self) {
        let mut index = 0;
        while index < self.out_recs.len() {
            let out_rec = OutRecId(index);
            index += 1;
            let Some(point) = self.out_recs[out_rec.0].points else {
                continue;
            };
            self.do_simple_polygon(out_rec, point);
        }
    }

    fn do_simple_polygon(&mut self, out_rec: OutRecId, mut point: OutPointId) {
        loop {
            self.split_repeated_point(out_rec, point);
            point = self.out_points.point(point).next;
            if point
                == self.out_recs[out_rec.0]
                    .points
                    .expect("simple output record remains live")
            {
                break;
            }
        }
    }

    fn split_repeated_point(&mut self, out_rec: OutRecId, point: OutPointId) {
        let mut duplicate = self.out_points.point(point).next;
        while duplicate
            != self.out_recs[out_rec.0]
                .points
                .expect("simple output record remains live")
        {
            let duplicate_point = *self.out_points.point(duplicate);
            if self.out_points.point(point).point == duplicate_point.point
                && duplicate_point.next != point
                && duplicate_point.previous != point
            {
                self.split_simple_polygon(out_rec, point, duplicate);
                duplicate = point;
            }
            duplicate = self.out_points.point(duplicate).next;
        }
    }

    fn split_simple_polygon(&mut self, old: OutRecId, first: OutPointId, second: OutPointId) {
        let first_previous = self.out_points.point(first).previous;
        let second_previous = self.out_points.point(second).previous;
        self.out_points.point_mut(first).previous = second_previous;
        self.out_points.point_mut(second_previous).next = first;
        self.out_points.point_mut(second).previous = first_previous;
        self.out_points.point_mut(first_previous).next = second;

        self.out_recs[old.0].points = Some(first);
        let new = self.create_out_rec();
        self.out_recs[new.0].points = Some(second);
        self.update_out_point_indices(new);

        let old_is_hole = self.out_recs[old.0].is_hole;
        let old_first_left = self.out_recs[old.0].first_left;
        if self.poly_contains(second, first) {
            self.out_recs[new.0].is_hole = !old_is_hole;
            self.out_recs[new.0].first_left = Some(old);
            if self.using_polytree {
                #[cfg(test)]
                self.simple_repairs_for_test.push(SimpleRepair::FirstLefts2);
                self.fixup_first_lefts2(new, old);
            }
        } else if self.poly_contains(first, second) {
            self.out_recs[new.0].is_hole = old_is_hole;
            self.out_recs[old.0].is_hole = !old_is_hole;
            self.out_recs[new.0].first_left = old_first_left;
            self.out_recs[old.0].first_left = Some(new);
            if self.using_polytree {
                #[cfg(test)]
                self.simple_repairs_for_test.push(SimpleRepair::FirstLefts2);
                self.fixup_first_lefts2(old, new);
            }
        } else {
            self.out_recs[new.0].is_hole = old_is_hole;
            self.out_recs[new.0].first_left = old_first_left;
            if self.using_polytree {
                #[cfg(test)]
                self.simple_repairs_for_test.push(SimpleRepair::FirstLefts1);
                self.fixup_first_lefts1(old, new);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn simple_repairs_for_test(&self) -> &[SimpleRepair] {
        &self.simple_repairs_for_test
    }
}
