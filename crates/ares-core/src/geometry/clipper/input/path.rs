use super::super::predicates::{HI_RANGE, LO_RANGE, slopes_equal};
use super::super::types::{Edge, EdgeId, OutputIndex};
use super::super::{Clipper, ClipperError, PathRole, z::KernelPoint};

enum DuplicateStep {
    NotDuplicate,
    Removed { edge: EdgeId, start: EdgeId },
    Stop,
}

impl Clipper {
    pub(in crate::geometry) fn add_path<P: Copy + Into<KernelPoint>>(
        &mut self,
        points: &[P],
        role: PathRole,
        closed: bool,
    ) -> Result<bool, ClipperError> {
        let Some(high_index) = candidate_high_index(points, closed) else {
            return Ok(false);
        };
        self.validate_range(points, high_index)?;

        let checkpoint = self.edges.len();
        let count = high_index + 1;
        for (index, point) in points.iter().copied().take(count).enumerate() {
            let previous = EdgeId(checkpoint + (index + count - 1) % count);
            let next = EdgeId(checkpoint + (index + 1) % count);
            self.edges
                .push(Edge::new(point.into(), role, previous, next));
        }

        let start = EdgeId(checkpoint);
        let Some(start) = self.clean_path(start, closed) else {
            self.edges.truncate(checkpoint);
            return Ok(false);
        };
        if !closed {
            self.has_open_paths = true;
            let terminal = self.edges.edge(start).previous;
            self.edges.edge_mut(terminal).output = OutputIndex::Skipped;
        }

        let is_flat = self.initialize_edges(start);
        if is_flat {
            if closed {
                self.edges.truncate(checkpoint);
                return Ok(false);
            }
            self.build_flat_open_minimum(start);
            return Ok(true);
        }

        self.build_local_minima(start, closed);
        Ok(true)
    }

    fn validate_range<P: Copy + Into<KernelPoint>>(
        &mut self,
        points: &[P],
        high_index: usize,
    ) -> Result<(), ClipperError> {
        self.range_test(points[0].into())?;
        self.range_test(points[high_index].into())?;
        for index in (1..high_index).rev() {
            self.range_test(points[index].into())?;
        }
        Ok(())
    }

    fn range_test(&mut self, point: KernelPoint) -> Result<(), ClipperError> {
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

    fn clean_path(&mut self, mut start: EdgeId, closed: bool) -> Option<EdgeId> {
        let mut edge = start;
        let mut loop_stop = start;
        loop {
            match self.remove_duplicate_if_present(edge, start, closed) {
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
            if closed
                && self.is_collinear(previous, edge, next)
                && (!self.options.preserve_collinear
                    || !between(
                        self.edges.edge(previous).current,
                        self.edges.edge(edge).current,
                        self.edges.edge(next).current,
                    ))
            {
                (edge, start) = self.remove_collinear(edge, start, next);
                loop_stop = edge;
                continue;
            }

            edge = next;
            if edge == loop_stop || (!closed && self.edges.edge(edge).next == start) {
                break;
            }
        }

        let edge_state = self.edges.edge(edge);
        let invalid = if closed {
            edge_state.previous == edge_state.next
        } else {
            edge == edge_state.next
        };
        (!invalid).then_some(start)
    }

    fn remove_collinear(&mut self, edge: EdgeId, start: EdgeId, next: EdgeId) -> (EdgeId, EdgeId) {
        let start = if edge == start { next } else { start };
        let edge = self.edges.remove(edge);
        (self.edges.edge(edge).previous, start)
    }

    fn remove_duplicate_if_present(
        &mut self,
        edge: EdgeId,
        start: EdgeId,
        closed: bool,
    ) -> DuplicateStep {
        let next = self.edges.edge(edge).next;
        if self.edges.edge(edge).current != self.edges.edge(next).current
            || (!closed && next == start)
        {
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

fn candidate_high_index<P: Copy + Into<KernelPoint>>(points: &[P], closed: bool) -> Option<usize> {
    let mut high_index = points.len().checked_sub(1)?;
    if closed {
        while high_index > 0 && Into::<KernelPoint>::into(points[high_index]) == points[0].into() {
            high_index -= 1;
        }
    }
    while high_index > 0
        && Into::<KernelPoint>::into(points[high_index]) == points[high_index - 1].into()
    {
        high_index -= 1;
    }
    let valid = if closed {
        high_index >= 2
    } else {
        high_index >= 1
    };
    valid.then_some(high_index)
}

fn point_outside(point: KernelPoint, range: i64) -> bool {
    point.x() > range || point.x() < -range || point.y() > range || point.y() < -range
}

fn between(first: KernelPoint, middle: KernelPoint, last: KernelPoint) -> bool {
    if first == last || first == middle || last == middle {
        false
    } else if first.x() != last.x() {
        (middle.x() > first.x()) == (middle.x() < last.x())
    } else {
        (middle.y() > first.y()) == (middle.y() < last.y())
    }
}
