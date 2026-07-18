use super::super::predicates::{HI_RANGE, LO_RANGE, slopes_equal};
use super::super::types::{Edge, EdgeId};
use super::super::{ClipperError, ClosedClipper, PathRole};
use crate::geometry::Point;

enum DuplicateStep {
    NotDuplicate,
    Removed { edge: EdgeId, start: EdgeId },
    Stop,
}

impl ClosedClipper {
    pub(super) fn add_path(
        &mut self,
        points: &[Point],
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        let Some(high_index) = candidate_high_index(points) else {
            return Ok(false);
        };

        self.validate_range(points, high_index)?;

        let checkpoint = self.edges.len();
        let start = EdgeId(checkpoint);
        let count = high_index + 1;
        for (index, point) in points.iter().copied().take(count).enumerate() {
            let previous = EdgeId(checkpoint + (index + count - 1) % count);
            let next = EdgeId(checkpoint + (index + 1) % count);
            self.edges.push(Edge::new(point, role, previous, next));
        }

        let Some(start) = self.clean_path(start) else {
            self.edges.truncate(checkpoint);
            return Ok(false);
        };
        if self.initialize_edges(start) {
            self.edges.truncate(checkpoint);
            return Ok(false);
        }

        self.build_local_minima(start);
        Ok(true)
    }

    fn validate_range(&mut self, points: &[Point], high_index: usize) -> Result<(), ClipperError> {
        self.range_test(points[0])?;
        self.range_test(points[high_index])?;
        for index in (1..high_index).rev() {
            self.range_test(points[index])?;
        }
        Ok(())
    }

    fn range_test(&mut self, point: Point) -> Result<(), ClipperError> {
        if self.use_full_range {
            if point_outside(point, HI_RANGE) {
                return Err(ClipperError::CoordinateOutOfRange);
            }
        } else if point_outside(point, LO_RANGE) {
            self.use_full_range = true;
            if point_outside(point, HI_RANGE) {
                return Err(ClipperError::CoordinateOutOfRange);
            }
        }
        Ok(())
    }

    fn clean_path(&mut self, mut start: EdgeId) -> Option<EdgeId> {
        let mut edge = start;
        let mut loop_stop = start;

        loop {
            match self.remove_duplicate_if_present(edge, start) {
                DuplicateStep::NotDuplicate => {}
                DuplicateStep::Removed {
                    edge: next_edge,
                    start: next_start,
                } => {
                    edge = next_edge;
                    start = next_start;
                    loop_stop = edge;
                    continue;
                }
                DuplicateStep::Stop => break,
            }

            let next = self.edges.edge(edge).next;
            let previous = self.edges.edge(edge).previous;
            if previous == next {
                break;
            }
            if self.is_collinear(previous, edge, next)
                && (!self.options.preserve_collinear
                    || !between(
                        self.edges.edge(previous).current,
                        self.edges.edge(edge).current,
                        self.edges.edge(next).current,
                    ))
            {
                (edge, start) = self.remove_collinear(edge, start);
                edge = self.edges.edge(edge).previous;
                loop_stop = edge;
                continue;
            }

            edge = next;
            if edge == loop_stop {
                break;
            }
        }

        (self.edges.edge(edge).previous != self.edges.edge(edge).next).then_some(start)
    }

    fn remove_duplicate_if_present(&mut self, edge: EdgeId, start: EdgeId) -> DuplicateStep {
        let next = self.edges.edge(edge).next;
        if self.edges.edge(edge).current != self.edges.edge(next).current {
            return DuplicateStep::NotDuplicate;
        }
        if edge == next {
            return DuplicateStep::Stop;
        }
        let next_start = if edge == start { next } else { start };
        DuplicateStep::Removed {
            edge: self.edges.remove(edge),
            start: next_start,
        }
    }

    fn remove_collinear(&mut self, edge: EdgeId, start: EdgeId) -> (EdgeId, EdgeId) {
        let next = self.edges.edge(edge).next;
        let next_start = if edge == start { next } else { start };
        (self.edges.remove(edge), next_start)
    }

    fn is_collinear(&self, previous: EdgeId, edge: EdgeId, next: EdgeId) -> bool {
        let first = self.edges.edge(previous).current;
        let middle = self.edges.edge(edge).current;
        let last = self.edges.edge(next).current;
        slopes_equal(
            first.x() - middle.x(),
            first.y() - middle.y(),
            middle.x() - last.x(),
            middle.y() - last.y(),
            self.use_full_range,
        )
    }

    fn initialize_edges(&mut self, start: EdgeId) -> bool {
        let start_y = self.edges.edge(start).current.y();
        let mut is_flat = true;
        let mut edge = start;
        loop {
            let next = self.edges.edge(edge).next;
            let next_point = self.edges.edge(next).current;
            self.edges.edge_mut(edge).initialize_direction(next_point);
            edge = next;
            if is_flat && self.edges.edge(edge).current.y() != start_y {
                is_flat = false;
            }
            if edge == start {
                return is_flat;
            }
        }
    }
}

fn candidate_high_index(points: &[Point]) -> Option<usize> {
    let mut high_index = points.len().checked_sub(1)?;
    while high_index > 0 && points[high_index] == points[0] {
        high_index -= 1;
    }
    while high_index > 0 && points[high_index] == points[high_index - 1] {
        high_index -= 1;
    }
    (high_index >= 2).then_some(high_index)
}

fn point_outside(point: Point, range: i64) -> bool {
    point.x() > range || point.x() < -range || point.y() > range || point.y() < -range
}

fn between(first: Point, middle: Point, last: Point) -> bool {
    if first == last || first == middle || last == middle {
        false
    } else if first.x() != last.x() {
        (middle.x() > first.x()) == (middle.x() < last.x())
    } else {
        (middle.y() > first.y()) == (middle.y() < last.y())
    }
}
