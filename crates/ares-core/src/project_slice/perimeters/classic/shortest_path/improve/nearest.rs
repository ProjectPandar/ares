//! ShortestPath.cpp:1097–1190: index-tied nearest endpoints on uniform cells.
pub(super) struct Nearest<'a> {
    points: &'a [[f64; 2]],
    origin: [f64; 2],
    cell: f64,
    cols: usize,
    rows: usize,
    first: Vec<usize>,
    index: Vec<usize>,
}
impl<'a> Nearest<'a> {
    pub(super) fn new(points: &'a [[f64; 2]]) -> Self {
        let mut min = [f64::MAX; 2];
        let mut max = [-f64::MAX; 2];
        for p in points {
            for a in 0..2 {
                min[a] = min[a].min(p[a]);
                max[a] = max[a].max(p[a]);
            }
        }
        let size = [max[0] - min[0], max[1] - min[1]];
        let count = points.len().max(1) as f64;
        let cell = (2.0 * size[0] * size[1] / count)
            .sqrt()
            .max(size[0].max(size[1]) / count);
        let cell = if cell <= 0.0 { 1.0 } else { cell };
        let cols = (size[0] / cell) as usize + 1;
        let rows = (size[1] / cell) as usize + 1;
        let mut out = Self {
            points,
            origin: min,
            cell,
            cols,
            rows,
            first: vec![0; cols * rows + 1],
            index: vec![0; points.len()],
        };
        for p in points {
            let c = out.cell_of(*p);
            out.first[c + 1] += 1;
        }
        for c in 1..out.first.len() {
            out.first[c] += out.first[c - 1];
        }
        let mut next = out.first[..out.first.len() - 1].to_vec();
        for (i, p) in points.iter().enumerate() {
            let c = out.cell_of(*p);
            out.index[next[c]] = i;
            next[c] += 1;
        }
        out
    }
    fn cell_of(&self, p: [f64; 2]) -> usize {
        let x = ((p[0] - self.origin[0]) / self.cell).clamp(0.0, (self.cols - 1) as f64) as usize;
        let y = ((p[1] - self.origin[1]) / self.cell).clamp(0.0, (self.rows - 1) as f64) as usize;
        y * self.cols + x
    }
    fn visit<const K: usize>(
        &self,
        c: [i64; 2],
        q: [f64; 2],
        filter: &impl Fn(usize) -> bool,
        best: &mut Vec<(f64, usize)>,
    ) {
        let [x, y] = c;
        if x < 0 || y < 0 || x >= self.cols as i64 || y >= self.rows as i64 {
            return;
        }
        let c = y as usize * self.cols + x as usize;
        for k in self.first[c]..self.first[c + 1] {
            let i = self.index[k];
            if !filter(i) {
                continue;
            }
            let p = self.points[i];
            let candidate = (
                (p[0] - q[0]) * (p[0] - q[0]) + (p[1] - q[1]) * (p[1] - q[1]),
                i,
            );
            let pos = best.partition_point(|b| *b <= candidate);
            if pos < K {
                best.insert(pos, candidate);
                best.truncate(K);
            }
        }
    }
    fn visit_ring<const K: usize>(
        &self,
        ring: ([i64; 2], i64),
        q: [f64; 2],
        filter: &impl Fn(usize) -> bool,
        best: &mut Vec<(f64, usize)>,
    ) {
        let (c, r) = ring;
        let [x, y] = c;
        if r == 0 {
            self.visit::<K>(c, q, filter, best);
            return;
        }
        for d in -r..=r {
            self.visit::<K>([x + d, y - r], q, filter, best);
            self.visit::<K>([x + d, y + r], q, filter, best);
            if d != -r && d != r {
                self.visit::<K>([x - r, y + d], q, filter, best);
                self.visit::<K>([x + r, y + d], q, filter, best);
            }
        }
    }
    pub(super) fn query<const K: usize>(
        &self,
        q: [f64; 2],
        filter: impl Fn(usize) -> bool,
    ) -> [usize; K] {
        let cx = ((q[0] - self.origin[0]) / self.cell)
            .floor()
            .clamp(0.0, (self.cols - 1) as f64) as i64;
        let cy = ((q[1] - self.origin[1]) / self.cell)
            .floor()
            .clamp(0.0, (self.rows - 1) as f64) as i64;
        let mut best = Vec::<(f64, usize)>::with_capacity(K + 1);
        for r in 0..=self.cols.max(self.rows) as i64 {
            self.visit_ring::<K>(([cx, cy], r), q, &filter, &mut best);
            if best.len() == K && best[K - 1].0 <= (r as f64 * self.cell).powi(2) {
                break;
            }
        }
        let mut out = [usize::MAX; K];
        for (slot, (_, i)) in out.iter_mut().zip(best) {
            *slot = i;
        }
        out
    }
}
