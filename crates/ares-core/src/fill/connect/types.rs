use crate::geometry::Point;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct BoundaryContour {
    pub(super) points: Vec<Point>,
    pub(super) params: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Intersection {
    pub(super) contour_index: Option<usize>,
    pub(super) point_index: usize,
    pub(super) param: f64,
    pub(super) prev: Option<usize>,
    pub(super) next: Option<usize>,
    pub(super) not_taken_prev: f64,
    pub(super) not_taken_next: f64,
    pub(super) consumed: bool,
    pub(super) prev_trimmed: bool,
    pub(super) next_trimmed: bool,
}

impl Intersection {
    pub(super) const fn unconnected() -> Self {
        Self {
            contour_index: None,
            point_index: usize::MAX,
            param: 0.0,
            prev: None,
            next: None,
            not_taken_prev: f64::MAX,
            not_taken_next: f64::MAX,
            consumed: false,
            prev_trimmed: false,
            next_trimmed: false,
        }
    }

    pub(super) const fn connected(contour_index: usize, point_index: usize) -> Self {
        Self {
            contour_index: Some(contour_index),
            point_index,
            ..Self::unconnected()
        }
    }

    pub(super) fn consume_prev(&mut self) {
        self.not_taken_prev = 0.0;
        self.prev_trimmed = true;
        self.consumed = true;
    }

    pub(super) fn consume_next(&mut self) {
        self.not_taken_next = 0.0;
        self.next_trimmed = true;
        self.consumed = true;
    }

    pub(super) fn trim_prev(&mut self, new_length: f64) {
        if new_length < self.not_taken_prev {
            self.not_taken_prev = new_length;
            self.prev_trimmed = true;
        }
    }

    pub(super) fn trim_next(&mut self, new_length: f64) {
        if new_length < self.not_taken_next {
            self.not_taken_next = new_length;
            self.next_trimmed = true;
        }
    }

    pub(super) fn could_take_prev(&self, scaled_epsilon: f64) -> bool {
        !self.consumed && self.not_taken_prev > scaled_epsilon
    }

    pub(super) fn could_take_next(&self, scaled_epsilon: f64) -> bool {
        !self.consumed && self.not_taken_next > scaled_epsilon
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct WorkingGraph {
    pub(super) boundary: Vec<BoundaryContour>,
    pub(super) intersections: Vec<Intersection>,
    pub(super) paths: Vec<Option<Vec<Point>>>,
    pub(super) parents: Vec<usize>,
    pub(super) line_half_width: f64,
}

impl WorkingGraph {
    pub(super) fn point(&self, intersection_index: usize) -> Point {
        let intersection = &self.intersections[intersection_index];
        self.boundary[intersection
            .contour_index
            .expect("a boundary point must be connected")]
        .points[intersection.point_index]
    }

    pub(super) const fn path_index_for_intersection(intersection_index: usize) -> usize {
        intersection_index / 2
    }

    pub(super) fn could_connect_prev(&self, intersection_index: usize) -> bool {
        let intersection = &self.intersections[intersection_index];
        let previous_index = intersection
            .prev
            .expect("a connected intersection has a previous link");
        !intersection.consumed
            && previous_index != intersection_index
            && !self.intersections[previous_index].consumed
            && !intersection.prev_trimmed
            && !self.intersections[previous_index].next_trimmed
    }

    pub(super) fn could_connect_next(&self, intersection_index: usize) -> bool {
        let intersection = &self.intersections[intersection_index];
        let next_index = intersection
            .next
            .expect("a connected intersection has a next link");
        !intersection.consumed
            && next_index != intersection_index
            && !self.intersections[next_index].consumed
            && !intersection.next_trimmed
            && !self.intersections[next_index].prev_trimmed
    }

    pub(super) fn root(&mut self, path_index: usize) -> usize {
        let mut last = path_index;
        loop {
            let lower = self.parents[last];
            debug_assert!(lower <= last);
            if lower == last {
                self.parents[path_index] = last;
                return last;
            }
            last = lower;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Arc {
    pub(super) intersection_index: usize,
    pub(super) length: f64,
}
