use super::super::Clipper;
use super::super::types::OutRecId;

impl Clipper {
    pub(in crate::geometry::clipper) fn fixup_out_polyline(&mut self, out_rec: OutRecId) {
        let Some(mut point) = self.out_recs[out_rec.0].points else {
            return;
        };
        let mut last = self.out_points.point(point).previous;
        while point != last {
            point = self.out_points.point(point).next;
            let previous = self.out_points.point(point).previous;
            if self.out_points.point(point).point == self.out_points.point(previous).point {
                self.remove_open_duplicate(point, previous, &mut last);
                point = previous;
            }
        }

        if point == self.out_points.point(point).previous {
            self.dispose_out_ring(point);
            self.out_recs[out_rec.0].points = None;
        }
    }

    fn remove_open_duplicate(
        &mut self,
        point: super::super::types::OutPointId,
        previous: super::super::types::OutPointId,
        last: &mut super::super::types::OutPointId,
    ) {
        if point == *last {
            *last = previous;
        }
        let next = self.out_points.point(point).next;
        self.out_points.point_mut(previous).next = next;
        self.out_points.point_mut(next).previous = previous;
        self.dispose_out_point(point);
    }
}
