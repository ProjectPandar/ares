//! ShortestPath.cpp:1100–1187::NearestPoints, the source uniform-grid query.
//! OrcaSlicer 345efec64c7b95a05ce5cf96df0dde66a158835c.

use std::cmp::Ordering;

use super::squared;

#[derive(Clone, Copy)]
struct Layout {
    origin: [f64; 2],
    cell: f64,
    cols: i32,
    rows: i32,
}

impl Layout {
    fn new(points: &[[f64; 2]]) -> Self {
        let mut min = points[0];
        let mut max = points[0];
        for point in points {
            for axis in 0..2 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        let size = [max[0] - min[0], max[1] - min[1]];
        let count = points.len() as f64;
        let mut cell = (2.0 * size[0] * size[1] / count)
            .sqrt()
            .max(size[0].max(size[1]) / count);
        if cell <= 0.0 {
            cell = 1.0;
        }
        Self {
            origin: min,
            cell,
            cols: (size[0] / cell) as i32 + 1,
            rows: (size[1] / cell) as i32 + 1,
        }
    }

    fn cell(self, point: [f64; 2]) -> usize {
        let x = (((point[0] - self.origin[0]) / self.cell) as i32).clamp(0, self.cols - 1);
        let y = (((point[1] - self.origin[1]) / self.cell) as i32).clamp(0, self.rows - 1);
        y as usize * self.cols as usize + x as usize
    }

    fn query_cell(self, point: [f64; 2]) -> (i32, i32) {
        let x = ((point[0] - self.origin[0]) / self.cell)
            .floor()
            .clamp(0.0, f64::from(self.cols - 1)) as i32;
        let y = ((point[1] - self.origin[1]) / self.cell)
            .floor()
            .clamp(0.0, f64::from(self.rows - 1)) as i32;
        (x, y)
    }
}

pub(super) struct NearestPoints<'a, const K: usize> {
    points: &'a [[f64; 2]],
    layout: Layout,
    first: Vec<usize>,
    index: Vec<usize>,
}

impl<'a, const K: usize> NearestPoints<'a, K> {
    pub(super) fn new(points: &'a [[f64; 2]]) -> Self {
        let layout = Layout::new(points);
        let mut first = vec![0; layout.cols as usize * layout.rows as usize + 1];
        for &point in points {
            first[layout.cell(point) + 1] += 1;
        }
        for cell in 1..first.len() {
            first[cell] += first[cell - 1];
        }
        let mut index = vec![0; points.len()];
        let mut next = first[..first.len() - 1].to_vec();
        for (i, &point) in points.iter().enumerate() {
            let cell = layout.cell(point);
            index[next[cell]] = i;
            next[cell] += 1;
        }
        Self {
            points,
            layout,
            first,
            index,
        }
    }

    pub(super) fn query(&self, q: [f64; 2], filter: impl Fn(usize) -> bool) -> [usize; K] {
        let mut best = Best {
            entries: [(0.0, 0); K],
            found: 0,
        };
        let (cx, cy) = self.layout.query_cell(q);
        for ring in 0..=self.layout.cols.max(self.layout.rows) {
            if ring == 0 {
                self.visit((cx, cy), q, &filter, &mut best);
            } else {
                self.visit_ring((cx, cy, ring), q, &filter, &mut best);
            }
            let reach = f64::from(ring) * self.layout.cell;
            if best.found == K && best.entries[K - 1].0 <= reach * reach {
                break;
            }
        }
        let mut out = [usize::MAX; K];
        for (slot, entry) in out.iter_mut().zip(&best.entries[..best.found]) {
            *slot = entry.1;
        }
        out
    }

    fn visit_ring(
        &self,
        (cx, cy, ring): (i32, i32, i32),
        q: [f64; 2],
        filter: &impl Fn(usize) -> bool,
        best: &mut Best<K>,
    ) {
        for delta in -ring..=ring {
            self.visit((cx + delta, cy - ring), q, filter, best);
            self.visit((cx + delta, cy + ring), q, filter, best);
            if delta != -ring && delta != ring {
                self.visit((cx - ring, cy + delta), q, filter, best);
                self.visit((cx + ring, cy + delta), q, filter, best);
            }
        }
    }

    fn visit(
        &self,
        (x, y): (i32, i32),
        q: [f64; 2],
        filter: &impl Fn(usize) -> bool,
        best: &mut Best<K>,
    ) {
        if x < 0 || y < 0 || x >= self.layout.cols || y >= self.layout.rows {
            return;
        }
        let cell = y as usize * self.layout.cols as usize + x as usize;
        for slot in self.first[cell]..self.first[cell + 1] {
            let index = self.index[slot];
            if filter(index) {
                best.insert((squared(self.points[index], q), index));
            }
        }
    }
}

struct Best<const K: usize> {
    entries: [(f64, usize); K],
    found: usize,
}

impl<const K: usize> Best<K> {
    fn insert(&mut self, candidate: (f64, usize)) {
        if self.found == K && candidate.partial_cmp(&self.entries[K - 1]) != Some(Ordering::Less) {
            return;
        }
        let at = self.entries[..self.found]
            .partition_point(|entry| candidate.partial_cmp(entry) != Some(Ordering::Less));
        self.entries.copy_within(at..self.found.min(K - 1), at + 1);
        self.entries[at] = candidate;
        self.found = (self.found + 1).min(K);
    }
}
