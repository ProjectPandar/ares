//! Indexed native MutablePolygon and its inclusive unprocessed range.

use super::{delta, dot_f64};
use crate::geometry::{Point, Polygon};

struct LinkedPoint {
    point: Point,
    previous: usize,
    next: usize,
    free_next: Option<usize>,
}

#[derive(Default)]
pub(super) struct Ring {
    nodes: Vec<LinkedPoint>,
    pub(super) head: Option<usize>,
    free: Option<usize>,
    pub(super) len: usize,
}

impl Ring {
    pub(super) fn assign(&mut self, points: &[Point]) {
        self.clear();
        self.nodes.reserve(points.len() * 2);
        self.len = points.len();
        if points.is_empty() {
            return;
        }
        self.head = Some(0);
        for (index, &point) in points.iter().enumerate() {
            self.nodes.push(LinkedPoint {
                point,
                previous: if index == 0 {
                    points.len() - 1
                } else {
                    index - 1
                },
                next: if index + 1 == points.len() {
                    0
                } else {
                    index + 1
                },
                free_next: None,
            });
        }
    }

    pub(super) fn clear(&mut self) {
        self.nodes.clear();
        self.head = None;
        self.free = None;
        self.len = 0;
    }

    pub(super) fn previous(&self, index: usize) -> usize {
        self.nodes[index].previous
    }
    pub(super) fn next(&self, index: usize) -> usize {
        self.nodes[index].next
    }
    pub(super) fn point(&self, index: usize) -> Point {
        self.nodes[index].point
    }
    pub(super) fn set_point(&mut self, index: usize, point: Point) {
        self.nodes[index].point = point;
    }

    pub(super) fn remove(&mut self, index: usize) -> usize {
        let previous = self.previous(index);
        let next = self.next(index);
        self.nodes[index].free_next = self.free;
        self.free = Some(index);
        self.len -= 1;
        if self.len == 0 {
            self.head = None;
        } else {
            if self.head == Some(index) {
                self.head = Some(next);
            }
            self.nodes[previous].next = next;
            self.nodes[next].previous = previous;
        }
        next
    }

    pub(super) fn insert(&mut self, before: usize, point: Point) -> usize {
        let previous = self.previous(before);
        let node = LinkedPoint {
            point,
            previous,
            next: before,
            free_next: None,
        };
        let index = if let Some(free) = self.free {
            self.free = self.nodes[free].free_next;
            self.nodes[free] = node;
            free
        } else {
            let index = self.nodes.len();
            self.nodes.push(node);
            index
        };
        self.nodes[previous].next = index;
        self.nodes[before].previous = index;
        self.len += 1;
        index
    }

    pub(super) fn remove_duplicates(&mut self, epsilon: f64) {
        let Some(begin) = self.head else {
            return;
        };
        let mut index = self.next(begin);
        while index != begin {
            let vector = delta(self.point(index), self.point(self.previous(index)))
                .map(|value| value as f64);
            index = if dot_f64(vector, vector) < epsilon * epsilon {
                self.remove(index)
            } else {
                self.next(index)
            };
        }
    }

    pub(super) fn polygon(&self) -> Polygon {
        if self.len < 3 {
            return Polygon::new(Vec::new());
        }
        let mut points = Vec::with_capacity(self.len);
        let begin = self.head.expect("nonempty native polygon has a head");
        let mut index = begin;
        loop {
            points.push(self.point(index));
            index = self.next(index);
            if index == begin {
                break;
            }
        }
        Polygon::new(points)
    }
}

pub(super) struct Unprocessed {
    begin: Option<usize>,
    end: Option<usize>,
}

impl Unprocessed {
    pub(super) fn new(ring: &Ring) -> Self {
        Self {
            begin: ring.head,
            end: ring.head.map(|head| ring.previous(head)),
        }
    }

    pub(super) fn process_next(&mut self, ring: &Ring) -> Option<usize> {
        let index = self.begin?;
        self.advance(ring);
        Some(index)
    }

    fn advance(&mut self, ring: &Ring) {
        if self.begin == self.end {
            self.begin = None;
            self.end = None;
        } else {
            self.begin = Some(ring.next(self.begin.expect("unprocessed front exists")));
        }
    }

    pub(super) fn remove_front(&mut self, ring: &mut Ring, index: usize) -> usize {
        if self.begin == Some(index) {
            self.advance(ring);
        }
        ring.remove(index)
    }

    pub(super) fn remove_back(&mut self, ring: &mut Ring, index: usize) -> usize {
        if self.end == Some(index) {
            if self.begin == self.end {
                self.begin = None;
                self.end = None;
            } else {
                self.end = Some(ring.previous(index));
            }
        }
        ring.remove(index)
    }
}
