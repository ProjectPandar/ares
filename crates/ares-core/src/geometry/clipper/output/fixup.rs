use super::super::ClosedClipper;
use super::super::predicates::slopes_equal_three;
use super::super::types::OutRecId;
use crate::geometry::{Point, Polygon};

impl ClosedClipper {
    pub(in crate::geometry::clipper) fn fixup_out_polygon(&mut self, out_rec: OutRecId) {
        let Some(mut point) = self.out_recs[out_rec.0].points else {
            return;
        };
        self.out_recs[out_rec.0].bottom_point = None;
        let preserve_collinear = self.options.preserve_collinear;
        let mut last_ok = None;

        loop {
            let current = *self.out_points.point(point);
            if current.previous == point || current.previous == current.next {
                self.dispose_out_ring(point);
                self.out_recs[out_rec.0].points = None;
                return;
            }

            let previous = self.out_points.point(current.previous).point;
            let next = self.out_points.point(current.next).point;
            let duplicate = current.point == next || current.point == previous;
            let removable_collinear =
                slopes_equal_three(previous, current.point, next, self.use_full_range)
                    && (!preserve_collinear || !point_between(previous, current.point, next));
            if duplicate || removable_collinear {
                last_ok = None;
                let removed = point;
                self.out_points.point_mut(current.previous).next = current.next;
                self.out_points.point_mut(current.next).previous = current.previous;
                point = current.previous;
                self.dispose_out_point(removed);
            } else if last_ok == Some(point) {
                break;
            } else {
                last_ok.get_or_insert(point);
                point = current.next;
            }
        }
        self.out_recs[out_rec.0].points = Some(point);
    }

    pub(in crate::geometry::clipper) fn build_paths(&self) -> Vec<Polygon> {
        let mut paths = Vec::with_capacity(self.out_recs.len());
        for out_rec in &self.out_recs {
            let Some(front) = out_rec.points else {
                continue;
            };
            let mut point = self.out_points.point(front).previous;
            let count = self.out_point_count(point);
            if count < 2 {
                continue;
            }

            let mut points = Vec::with_capacity(count);
            for _ in 0..count {
                let output = self.out_points.point(point);
                points.push(output.point);
                point = output.previous;
            }
            paths.push(Polygon::new(points));
        }
        paths
    }
}

fn point_between(first: Point, middle: Point, last: Point) -> bool {
    if first == last || first == middle || last == middle {
        false
    } else if first.x() != last.x() {
        (middle.x() > first.x()) == (middle.x() < last.x())
    } else {
        (middle.y() > first.y()) == (middle.y() < last.y())
    }
}
