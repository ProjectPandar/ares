use super::super::Clipper;
use super::super::predicates::slopes_equal_three;
use super::super::types::{OutPointId, OutRecId};

impl Clipper {
    // Clipper2 CleanCollinear (clipper.engine.cpp:1531-1565). This arena's
    // previous link is Clipper2's next, also used by BuildPath64's traversal.
    pub(in crate::geometry::clipper) fn fixup_positive_front(&mut self, out_rec: OutRecId) {
        let Some(mut point) = self.out_recs[out_rec.0].points else {
            return;
        };
        self.out_recs[out_rec.0].bottom_point = None;
        let mut start = point;
        loop {
            let current = *self.out_points.point(point);
            if current.previous == point || current.previous == current.next {
                self.dispose_out_ring(point);
                self.out_recs[out_rec.0].points = None;
                return;
            }
            let previous = self.out_points.point(current.previous).point;
            let next = self.out_points.point(current.next).point;
            if slopes_equal_three(previous, current.point, next, self.use_full_range) {
                point = self.remove_positive_collinear(out_rec, point);
                start = point;
                continue;
            }
            point = current.previous;
            if point == start {
                break;
            }
        }
    }

    fn remove_positive_collinear(&mut self, out_rec: OutRecId, point: OutPointId) -> OutPointId {
        let current = *self.out_points.point(point);
        if self.out_recs[out_rec.0].points == Some(point) {
            self.out_recs[out_rec.0].points = Some(current.next);
        }
        self.out_points.point_mut(current.previous).next = current.next;
        self.out_points.point_mut(current.next).previous = current.previous;
        self.dispose_out_point(point);
        current.previous
    }
}
