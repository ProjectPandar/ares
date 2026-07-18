use super::super::ClosedClipper;
use super::super::predicates::slopes_equal_three;
use super::super::types::{Join, OutPointId, OutRecId};
use crate::geometry::Point;

#[derive(Clone, Copy, Eq, PartialEq)]
enum HorizontalDirection {
    LeftToRight,
    RightToLeft,
}

struct HorizontalJoin {
    first: OutPointId,
    first_end: OutPointId,
    second: OutPointId,
    second_end: OutPointId,
    point: Point,
    discard_left: bool,
}

impl ClosedClipper {
    pub(super) fn join_points(
        &mut self,
        join: &mut Join,
        first_rec: OutRecId,
        second_rec: OutRecId,
    ) -> bool {
        let first_point = self.out_points.point(join.first).point;
        let second_point = self.out_points.point(join.second).point;
        let horizontal = first_point.y() == join.offset.y();

        if horizontal && join.offset == first_point && join.offset == second_point {
            return self.join_strictly_simple(join, first_rec, second_rec);
        }
        if horizontal {
            return self.join_horizontal_points(join);
        }
        self.join_nonhorizontal_points(join, first_rec, second_rec)
    }

    fn join_strictly_simple(
        &mut self,
        join: &mut Join,
        first_rec: OutRecId,
        second_rec: OutRecId,
    ) -> bool {
        if first_rec != second_rec {
            return false;
        }

        let first = join.first;
        let second = join.second;
        let first_next = self.next_distinct_from(first, join.offset);
        let second_next = self.next_distinct_from(second, join.offset);
        let reverse_first = self.out_points.point(first_next).point.y() > join.offset.y();
        let reverse_second = self.out_points.point(second_next).point.y() > join.offset.y();
        if reverse_first == reverse_second {
            return false;
        }

        let new_fragment = if reverse_first {
            let first_duplicate = self.duplicate_out_point(first, false);
            let second_duplicate = self.duplicate_out_point(second, true);
            self.out_points.point_mut(first).previous = second;
            self.out_points.point_mut(second).next = first;
            self.out_points.point_mut(first_duplicate).next = second_duplicate;
            self.out_points.point_mut(second_duplicate).previous = first_duplicate;
            first_duplicate
        } else {
            let first_duplicate = self.duplicate_out_point(first, true);
            let second_duplicate = self.duplicate_out_point(second, false);
            self.out_points.point_mut(first).next = second;
            self.out_points.point_mut(second).previous = first;
            self.out_points.point_mut(first_duplicate).previous = second_duplicate;
            self.out_points.point_mut(second_duplicate).next = first_duplicate;
            first_duplicate
        };
        join.first = first;
        join.second = new_fragment;
        true
    }

    fn join_horizontal_points(&mut self, join: &mut Join) -> bool {
        let mut first = join.first;
        let mut first_end = first;
        while {
            let previous = self.out_points.point(first).previous;
            self.out_points.point(previous).point.y() == self.out_points.point(first).point.y()
                && previous != first_end
                && previous != join.second
        } {
            first = self.out_points.point(first).previous;
        }
        while {
            let next = self.out_points.point(first_end).next;
            self.out_points.point(next).point.y() == self.out_points.point(first_end).point.y()
                && next != first
                && next != join.second
        } {
            first_end = self.out_points.point(first_end).next;
        }
        let first_after = self.out_points.point(first_end).next;
        if first_after == first || first_after == join.second {
            return false;
        }

        let mut second = join.second;
        let mut second_end = second;
        while {
            let previous = self.out_points.point(second).previous;
            self.out_points.point(previous).point.y() == self.out_points.point(second).point.y()
                && previous != second_end
                && previous != first_end
        } {
            second = self.out_points.point(second).previous;
        }
        while {
            let next = self.out_points.point(second_end).next;
            self.out_points.point(next).point.y() == self.out_points.point(second_end).point.y()
                && next != second
                && next != first
        } {
            second_end = self.out_points.point(second_end).next;
        }
        let second_after = self.out_points.point(second_end).next;
        if second_after == second || second_after == first {
            return false;
        }

        let first_start_x = self.out_points.point(first).point.x();
        let first_end_x = self.out_points.point(first_end).point.x();
        let second_start_x = self.out_points.point(second).point.x();
        let second_end_x = self.out_points.point(second_end).point.x();
        let Some((left, right)) = overlap(first_start_x, first_end_x, second_start_x, second_end_x)
        else {
            return false;
        };

        let (point, discard_left) = if (left..=right).contains(&first_start_x) {
            (
                self.out_points.point(first).point,
                first_start_x > first_end_x,
            )
        } else if (left..=right).contains(&second_start_x) {
            (
                self.out_points.point(second).point,
                second_start_x > second_end_x,
            )
        } else if (left..=right).contains(&first_end_x) {
            (
                self.out_points.point(first_end).point,
                first_end_x > first_start_x,
            )
        } else {
            (
                self.out_points.point(second_end).point,
                second_end_x > second_start_x,
            )
        };

        join.first = first;
        join.second = second;
        self.join_horizontal(HorizontalJoin {
            first,
            first_end,
            second,
            second_end,
            point,
            discard_left,
        })
    }

    fn join_nonhorizontal_points(
        &mut self,
        join: &mut Join,
        first_rec: OutRecId,
        second_rec: OutRecId,
    ) -> bool {
        let first = join.first;
        let second = join.second;
        let (first_neighbour, reverse_first) =
            match self.valid_nonhorizontal_neighbour(first, join.offset) {
                Some(value) => value,
                None => return false,
            };
        let (second_neighbour, reverse_second) =
            match self.valid_nonhorizontal_neighbour(second, join.offset) {
                Some(value) => value,
                None => return false,
            };
        if first_neighbour == first
            || second_neighbour == second
            || first_neighbour == second_neighbour
            || (first_rec == second_rec && reverse_first == reverse_second)
        {
            return false;
        }

        let new_fragment = if reverse_first {
            let first_duplicate = self.duplicate_out_point(first, false);
            let second_duplicate = self.duplicate_out_point(second, true);
            self.out_points.point_mut(first).previous = second;
            self.out_points.point_mut(second).next = first;
            self.out_points.point_mut(first_duplicate).next = second_duplicate;
            self.out_points.point_mut(second_duplicate).previous = first_duplicate;
            first_duplicate
        } else {
            let first_duplicate = self.duplicate_out_point(first, true);
            let second_duplicate = self.duplicate_out_point(second, false);
            self.out_points.point_mut(first).next = second;
            self.out_points.point_mut(second).previous = first;
            self.out_points.point_mut(first_duplicate).previous = second_duplicate;
            self.out_points.point_mut(second_duplicate).next = first_duplicate;
            first_duplicate
        };
        join.first = first;
        join.second = new_fragment;
        true
    }

    fn next_distinct_from(&self, point: OutPointId, coordinate: Point) -> OutPointId {
        let mut next = self.out_points.point(point).next;
        while next != point && self.out_points.point(next).point == coordinate {
            next = self.out_points.point(next).next;
        }
        next
    }

    fn valid_nonhorizontal_neighbour(
        &self,
        point: OutPointId,
        offset: Point,
    ) -> Option<(OutPointId, bool)> {
        let coordinate = self.out_points.point(point).point;
        let next = self.next_distinct_from(point, coordinate);
        let reverse = self.out_points.point(next).point.y() > coordinate.y()
            || !slopes_equal_three(
                coordinate,
                self.out_points.point(next).point,
                offset,
                self.use_full_range,
            );
        if !reverse {
            return Some((next, false));
        }

        let mut previous = self.out_points.point(point).previous;
        while previous != point && self.out_points.point(previous).point == coordinate {
            previous = self.out_points.point(previous).previous;
        }
        let valid = self.out_points.point(previous).point.y() <= coordinate.y()
            && slopes_equal_three(
                coordinate,
                self.out_points.point(previous).point,
                offset,
                self.use_full_range,
            );
        valid.then_some((previous, true))
    }

    fn join_horizontal(&mut self, join: HorizontalJoin) -> bool {
        let mut first = join.first;
        let mut second = join.second;
        let first_direction = self.output_horizontal_direction(first, join.first_end);
        let second_direction = self.output_horizontal_direction(second, join.second_end);
        if first_direction == second_direction {
            return false;
        }

        let first_duplicate = self.insert_horizontal_join_point(
            &mut first,
            first_direction,
            join.point,
            join.discard_left,
        );
        let second_duplicate = self.insert_horizontal_join_point(
            &mut second,
            second_direction,
            join.point,
            join.discard_left,
        );

        if (first_direction == HorizontalDirection::LeftToRight) == join.discard_left {
            self.out_points.point_mut(first).previous = second;
            self.out_points.point_mut(second).next = first;
            self.out_points.point_mut(first_duplicate).next = second_duplicate;
            self.out_points.point_mut(second_duplicate).previous = first_duplicate;
        } else {
            self.out_points.point_mut(first).next = second;
            self.out_points.point_mut(second).previous = first;
            self.out_points.point_mut(first_duplicate).previous = second_duplicate;
            self.out_points.point_mut(second_duplicate).next = first_duplicate;
        }
        true
    }

    fn output_horizontal_direction(
        &self,
        start: OutPointId,
        end: OutPointId,
    ) -> HorizontalDirection {
        if self.out_points.point(start).point.x() > self.out_points.point(end).point.x() {
            HorizontalDirection::RightToLeft
        } else {
            HorizontalDirection::LeftToRight
        }
    }

    fn insert_horizontal_join_point(
        &mut self,
        point_id: &mut OutPointId,
        direction: HorizontalDirection,
        point: Point,
        discard_left: bool,
    ) -> OutPointId {
        loop {
            let current = self.out_points.point(*point_id).point;
            let next_id = self.out_points.point(*point_id).next;
            let next = self.out_points.point(next_id).point;
            let within = match direction {
                HorizontalDirection::LeftToRight => {
                    next.x() <= point.x() && next.x() >= current.x()
                }
                HorizontalDirection::RightToLeft => {
                    next.x() >= point.x() && next.x() <= current.x()
                }
            };
            if !within || next.y() != point.y() {
                break;
            }
            *point_id = next_id;
        }

        let current_x = self.out_points.point(*point_id).point.x();
        let advance = match direction {
            HorizontalDirection::LeftToRight => discard_left && current_x != point.x(),
            HorizontalDirection::RightToLeft => !discard_left && current_x != point.x(),
        };
        if advance {
            *point_id = self.out_points.point(*point_id).next;
        }

        let insert_after = match direction {
            HorizontalDirection::LeftToRight => !discard_left,
            HorizontalDirection::RightToLeft => discard_left,
        };
        let mut duplicate = self.duplicate_out_point(*point_id, insert_after);
        if self.out_points.point(duplicate).point != point {
            *point_id = duplicate;
            self.out_points.point_mut(*point_id).point = point;
            duplicate = self.duplicate_out_point(*point_id, insert_after);
        }
        duplicate
    }
}

fn overlap(
    first_start: i64,
    first_end: i64,
    second_start: i64,
    second_end: i64,
) -> Option<(i64, i64)> {
    let left = first_start.min(first_end).max(second_start.min(second_end));
    let right = first_start.max(first_end).min(second_start.max(second_end));
    (left < right).then_some((left, right))
}
