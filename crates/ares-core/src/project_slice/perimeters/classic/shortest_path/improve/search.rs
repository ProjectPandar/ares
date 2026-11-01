use super::{Search, dist};
impl Search<'_> {
    pub(super) fn improve_at(&mut self, p: usize) -> bool {
        if self.incoming(p) || self.outgoing(p) || self.front(p) {
            return true;
        }
        let n = self.order.len();
        for last in p..(p + 3).min(n) {
            if self.or_opt(p, last, usize::MAX) {
                return true;
            }
            let removal = dist(self.prev(p), Some(self.entry(p)))
                + dist(Some(self.exit(last)), self.next(last))
                - dist(self.prev(p), self.next(last));
            if self.move_neighbors(p, last, self.entry_idx(p), removal)
                || self.move_neighbors(p, last, self.exit_idx(last), removal)
            {
                return true;
            }
        }
        false
    }
    fn move_neighbors(&mut self, p: usize, last: usize, e: usize, removal: f64) -> bool {
        for q in self.neighbors[e] {
            if !self.closer(self.points[e], q, removal) {
                break;
            }
            let m = self.pos[q / 2];
            let gap = if self.is_exit(q) {
                m
            } else if m == last + 1 {
                p.wrapping_sub(1)
            } else {
                m.wrapping_sub(1)
            };
            if (m < p || m > last) && self.or_opt(p, last, gap) {
                return true;
            }
        }
        false
    }
    fn incoming(&mut self, p: usize) -> bool {
        let Some(a) = self.prev(p) else {
            return false;
        };
        let old = dist(Some(a), Some(self.entry(p)));
        let candidates = if p > 0 {
            self.neighbors[self.exit_idx(p - 1)]
        } else {
            self.start_neighbors
        };
        for q in candidates {
            if !self.closer(a, q, old) {
                break;
            }
            if self.is_exit(q) && self.pos[q / 2] >= p && self.two_opt(p, self.pos[q / 2]) {
                return true;
            }
        }
        if p == 0 {
            return false;
        }
        for q in self.neighbors[self.entry_idx(p)] {
            if !self.closer(self.entry(p), q, old) {
                break;
            }
            if !self.is_exit(q) && self.pos[q / 2] < p && self.two_opt(self.pos[q / 2], p - 1) {
                return true;
            }
        }
        false
    }
    fn outgoing(&mut self, p: usize) -> bool {
        if p + 1 >= self.order.len() {
            return false;
        }
        let old = dist(Some(self.exit(p)), Some(self.entry(p + 1)));
        for q in self.neighbors[self.exit_idx(p)] {
            if !self.closer(self.exit(p), q, old) {
                break;
            }
            if self.is_exit(q) && self.pos[q / 2] > p && self.two_opt(p + 1, self.pos[q / 2]) {
                return true;
            }
        }
        for q in self.neighbors[self.entry_idx(p + 1)] {
            if !self.closer(self.entry(p + 1), q, old) {
                break;
            }
            if !self.is_exit(q) && self.pos[q / 2] <= p && self.two_opt(self.pos[q / 2], p) {
                return true;
            }
        }
        false
    }
    fn front(&mut self, p: usize) -> bool {
        if p != 0 {
            return false;
        }
        for q in self.start_neighbors {
            if q == usize::MAX {
                break;
            }
            let m = self.pos[q / 2];
            if m == 0 {
                continue;
            }
            let exit_near = self.is_exit(q);
            let range = if exit_near {
                1..m + 1
            } else {
                m..self.order.len()
            };
            let moved = if exit_near {
                range.into_iter().any(|k| self.or_opt(k, m, usize::MAX))
            } else {
                range.into_iter().any(|k| self.or_opt(m, k, usize::MAX))
            };
            if moved {
                return true;
            }
        }
        false
    }
}
