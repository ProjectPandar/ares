//! OrderingStrategies.cpp:147–193 uniform-grid crossing scan, retaining
//! smallest edge i then smallest j. OrcaSlicer
//! 345efec64c7b95a05ce5cf96df0dde66a158835c.

use super::{Point, crossings::edges_cross};

#[derive(Clone, Copy)]
struct Layout {
    min: Point,
    width: i64,
    height: i64,
    size: usize,
}

impl Layout {
    fn cells(self, a: Point, b: Point) -> impl Iterator<Item = usize> {
        let x0 = (a.x().min(b.x()) - self.min.x()) / self.width;
        let x1 = (a.x().max(b.x()) - self.min.x()) / self.width;
        let y0 = (a.y().min(b.y()) - self.min.y()) / self.height;
        let y1 = (a.y().max(b.y()) - self.min.y()) / self.height;
        (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| y as usize * self.size + x as usize))
    }
}

pub(super) struct CrossingGrid {
    layout: Layout,
    cells: Vec<Vec<usize>>,
}

impl CrossingGrid {
    pub(super) fn new(path: &[usize], points: &[Point]) -> Self {
        let mut min = points[path[0]];
        let mut max = min;
        for &index in path {
            let point = points[index];
            min = Point::new(min.x().min(point.x()), min.y().min(point.y()));
            max = Point::new(max.x().max(point.x()), max.y().max(point.y()));
        }
        let size = ((path.len() as f64).sqrt() as usize).clamp(1, 256);
        let layout = Layout {
            min,
            width: ((max.x() - min.x()) / size as i64 + 1).max(1),
            height: ((max.y() - min.y()) / size as i64 + 1).max(1),
            size,
        };
        Self {
            layout,
            cells: vec![Vec::new(); size * size],
        }
    }

    pub(super) fn find(&mut self, path: &[usize], points: &[Point]) -> Option<(usize, usize)> {
        self.cells.iter_mut().for_each(Vec::clear);
        for edge in 0..path.len() {
            for cell in self
                .layout
                .cells(points[path[edge]], points[path[(edge + 1) % path.len()]])
            {
                self.cells[cell].push(edge);
            }
        }
        for first in 0..path.len() {
            let last = self.first_crossing(first, path, points);
            if last != usize::MAX {
                return Some((first, last));
            }
        }
        None
    }

    fn first_crossing(&self, first: usize, path: &[usize], points: &[Point]) -> usize {
        let mut found = usize::MAX;
        let candidates = self
            .layout
            .cells(points[path[first]], points[path[(first + 1) % path.len()]])
            .flat_map(|cell| self.cells[cell].iter().copied());
        for last in candidates {
            if last < first + 2 || last >= found || (first == 0 && last + 1 == path.len()) {
                continue;
            }
            if edges_cross(path, points, first, last) {
                found = last;
            }
        }
        found
    }
}
