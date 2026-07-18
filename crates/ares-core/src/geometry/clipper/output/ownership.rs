use super::super::ClosedClipper;
use super::super::predicates::get_dx;
use super::super::types::{EdgeId, OutPointId, OutRecId, OutputIndex};
use crate::geometry::Point;

impl ClosedClipper {
    pub(super) fn set_hole_state(&mut self, edge: EdgeId, out_rec: OutRecId) {
        let mut previous = self.edges.edge(edge).previous_in_ael;
        let mut candidate = None;
        while let Some(edge_id) = previous {
            let edge = self.edges.edge(edge_id);
            if let OutputIndex::Assigned(candidate_rec) = edge.output
                && edge.wind_delta != 0
            {
                candidate = next_hole_candidate(candidate, candidate_rec);
            }
            previous = edge.previous_in_ael;
        }

        if let Some(first_left) = candidate {
            self.out_recs[out_rec.0].first_left = Some(first_left);
            self.out_recs[out_rec.0].is_hole = !self.out_recs[first_left.0].is_hole;
        } else {
            self.out_recs[out_rec.0].first_left = None;
            self.out_recs[out_rec.0].is_hole = false;
        }
    }

    pub(super) fn get_out_rec(&self, mut out_rec: OutRecId) -> OutRecId {
        while self.out_recs[out_rec.0].root != out_rec {
            out_rec = self.out_recs[out_rec.0].root;
        }
        out_rec
    }

    pub(super) fn out_rec_right_of(&self, mut first: OutRecId, second: OutRecId) -> bool {
        while let Some(first_left) = self.out_recs[first.0].first_left {
            if first_left == second {
                return true;
            }
            first = first_left;
        }
        false
    }

    pub(super) fn get_lowermost_rec(&mut self, first: OutRecId, second: OutRecId) -> OutRecId {
        let first_bottom = self.bottom_point(first);
        let second_bottom = self.bottom_point(second);
        let first_point = *self.out_points.point(first_bottom);
        let second_point = *self.out_points.point(second_bottom);

        if first_point.point.y() > second_point.point.y() {
            first
        } else if first_point.point.y() < second_point.point.y() {
            second
        } else if first_point.point.x() < second_point.point.x() {
            first
        } else if first_point.point.x() > second_point.point.x() || first_point.next == first_bottom
        {
            second
        } else if second_point.next == second_bottom
            || self.first_is_bottom_point(first_bottom, second_bottom)
        {
            first
        } else {
            second
        }
    }

    pub(super) fn poly_contains(&self, inner: OutPointId, outer: OutPointId) -> bool {
        let mut point = inner;
        loop {
            let result = self.out_ring_point_in_polygon(self.out_points.point(point).point, outer);
            if result >= 0 {
                return result > 0;
            }
            point = self.out_points.point(point).next;
            if point == inner {
                return true;
            }
        }
    }

    pub(super) fn fixup_first_lefts1(&mut self, old: OutRecId, new: OutRecId) {
        let new_points = self.out_recs[new.0]
            .points
            .expect("new output record has points");
        for index in 0..self.out_recs.len() {
            let out_rec = self.out_recs[index];
            if let Some(points) = out_rec.points
                && self.parse_first_left(out_rec.first_left) == Some(old)
                && self.poly_contains(points, new_points)
            {
                self.out_recs[index].first_left = Some(new);
            }
        }
    }

    pub(super) fn fixup_first_lefts2(&mut self, inner: OutRecId, outer: OutRecId) {
        let outer_first_left = self.out_recs[outer.0].first_left;
        let inner_points = self.out_recs[inner.0]
            .points
            .expect("inner output record has points");
        let outer_points = self.out_recs[outer.0]
            .points
            .expect("outer output record has points");
        for index in 0..self.out_recs.len() {
            let out_rec = self.out_recs[index];
            if out_rec.points.is_none() || index == outer.0 || index == inner.0 {
                continue;
            }
            let first_left = self.parse_first_left(out_rec.first_left);
            if first_left != outer_first_left
                && first_left != Some(inner)
                && first_left != Some(outer)
            {
                continue;
            }
            let points = out_rec.points.expect("checked output points");
            if self.poly_contains(points, inner_points) {
                self.out_recs[index].first_left = Some(inner);
            } else if self.poly_contains(points, outer_points) {
                self.out_recs[index].first_left = Some(outer);
            } else if out_rec.first_left == Some(inner) || out_rec.first_left == Some(outer) {
                self.out_recs[index].first_left = outer_first_left;
            }
        }
    }

    pub(super) fn fixup_first_lefts3(&mut self, old: OutRecId, new: OutRecId) {
        for index in 0..self.out_recs.len() {
            let out_rec = self.out_recs[index];
            if out_rec.points.is_some() && self.parse_first_left(out_rec.first_left) == Some(old) {
                self.out_recs[index].first_left = Some(new);
            }
        }
    }

    fn parse_first_left(&self, mut first_left: Option<OutRecId>) -> Option<OutRecId> {
        while let Some(out_rec) = first_left {
            if self.out_recs[out_rec.0].points.is_some() {
                return Some(out_rec);
            }
            first_left = self.out_recs[out_rec.0].first_left;
        }
        None
    }

    fn bottom_point(&mut self, out_rec: OutRecId) -> OutPointId {
        if let Some(point) = self.out_recs[out_rec.0].bottom_point {
            return point;
        }
        let start = self.out_recs[out_rec.0]
            .points
            .expect("output record has points");
        let point = self.get_bottom_point(start);
        self.out_recs[out_rec.0].bottom_point = Some(point);
        point
    }

    fn get_bottom_point(&self, start: OutPointId) -> OutPointId {
        let mut bottom = start;
        let mut duplicate = None;
        let mut point = self.out_points.point(start).next;
        while point != bottom {
            let current = *self.out_points.point(point);
            self.consider_bottom_candidate(point, &mut bottom, &mut duplicate);
            point = current.next;
        }
        duplicate.map_or(bottom, |duplicate| {
            self.resolve_bottom_duplicates(bottom, duplicate)
        })
    }

    fn consider_bottom_candidate(
        &self,
        point: OutPointId,
        bottom: &mut OutPointId,
        duplicate: &mut Option<OutPointId>,
    ) {
        let current = *self.out_points.point(point);
        let bottom_point = *self.out_points.point(*bottom);
        if current.point.y() > bottom_point.point.y()
            || current.point.y() == bottom_point.point.y()
                && current.point.x() < bottom_point.point.x()
        {
            *bottom = point;
            *duplicate = None;
        } else if current.point == bottom_point.point
            && current.next != *bottom
            && current.previous != *bottom
        {
            *duplicate = Some(point);
        }
    }

    fn resolve_bottom_duplicates(
        &self,
        mut bottom: OutPointId,
        mut duplicate: OutPointId,
    ) -> OutPointId {
        let start = bottom;
        while duplicate != start {
            if !self.first_is_bottom_point(start, duplicate) {
                bottom = duplicate;
            }
            duplicate = self.out_points.point(duplicate).next;
            while self.out_points.point(duplicate).point != self.out_points.point(bottom).point {
                duplicate = self.out_points.point(duplicate).next;
            }
        }
        bottom
    }

    fn first_is_bottom_point(&self, first: OutPointId, second: OutPointId) -> bool {
        let first_point = self.out_points.point(first).point;
        let second_point = self.out_points.point(second).point;
        let (first_previous, first_next) = self.distinct_neighbours(first);
        let (second_previous, second_next) = self.distinct_neighbours(second);
        let first_slopes = [
            get_dx(first_point, first_previous).abs(),
            get_dx(first_point, first_next).abs(),
        ];
        let second_slopes = [
            get_dx(second_point, second_previous).abs(),
            get_dx(second_point, second_next).abs(),
        ];
        let first_max = first_slopes[0].max(first_slopes[1]);
        let first_min = first_slopes[0].min(first_slopes[1]);
        let second_max = second_slopes[0].max(second_slopes[1]);
        let second_min = second_slopes[0].min(second_slopes[1]);
        if first_max == second_max && first_min == second_min {
            self.out_ring_area(first) > 0.0
        } else {
            (first_slopes[0] >= second_slopes[0] && first_slopes[0] >= second_slopes[1])
                || (first_slopes[1] >= second_slopes[0] && first_slopes[1] >= second_slopes[1])
        }
    }

    fn distinct_neighbours(&self, point: OutPointId) -> (Point, Point) {
        let coordinate = self.out_points.point(point).point;
        let mut previous = self.out_points.point(point).previous;
        while previous != point && self.out_points.point(previous).point == coordinate {
            previous = self.out_points.point(previous).previous;
        }
        let mut next = self.out_points.point(point).next;
        while next != point && self.out_points.point(next).point == coordinate {
            next = self.out_points.point(next).next;
        }
        (
            self.out_points.point(previous).point,
            self.out_points.point(next).point,
        )
    }

    fn out_ring_point_in_polygon(&self, point: Point, start: OutPointId) -> i32 {
        let mut result = 0;
        let mut current = start;
        loop {
            let current_point = self.out_points.point(current).point;
            let next = self.out_points.point(current).next;
            let next_point = self.out_points.point(next).point;
            match ring_crossing(current_point, next_point, point) {
                RingCrossing::None => {}
                RingCrossing::Toggle => result = 1 - result,
                RingCrossing::Boundary => return -1,
            }
            current = next;
            if current == start {
                return result;
            }
        }
    }
}

fn next_hole_candidate(candidate: Option<OutRecId>, out_rec: OutRecId) -> Option<OutRecId> {
    if candidate == Some(out_rec) {
        None
    } else {
        candidate.or(Some(out_rec))
    }
}

enum RingCrossing {
    None,
    Toggle,
    Boundary,
}

fn ring_crossing(current: Point, next: Point, point: Point) -> RingCrossing {
    if next.y() == point.y()
        && (next.x() == point.x()
            || (current.y() == point.y() && ((next.x() > point.x()) == (current.x() < point.x()))))
    {
        return RingCrossing::Boundary;
    }
    if (current.y() < point.y()) == (next.y() < point.y()) {
        return RingCrossing::None;
    }
    if current.x() >= point.x() && next.x() > point.x() {
        return RingCrossing::Toggle;
    }
    if current.x() < point.x() && next.x() <= point.x() {
        return RingCrossing::None;
    }
    let determinant = (current.x() - point.x()) as f64 * (next.y() - point.y()) as f64
        - (next.x() - point.x()) as f64 * (current.y() - point.y()) as f64;
    if determinant == 0.0 {
        RingCrossing::Boundary
    } else if (determinant > 0.0) == (next.y() > current.y()) {
        RingCrossing::Toggle
    } else {
        RingCrossing::None
    }
}
