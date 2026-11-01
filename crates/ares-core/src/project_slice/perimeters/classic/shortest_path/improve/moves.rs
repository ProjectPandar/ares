use super::{Search, dist};
impl Search<'_> {
    pub(super) fn two_opt(&mut self, i: usize, j: usize) -> bool {
        if !self.reversible(i, j) {
            return false;
        }
        let a = self.prev(i);
        let b = self.next(j);
        if dist(a, Some(self.exit(j))) + dist(Some(self.entry(i)), b)
            > dist(a, Some(self.entry(i))) + dist(Some(self.exit(j)), b) - 100.0
        {
            return false;
        }
        self.order[i..=j].reverse();
        for k in i..=j {
            self.flipped[self.order[k]] ^= true;
        }
        self.refresh(i, j);
        for k in [i.wrapping_sub(1), i, j, j + 1] {
            self.enqueue(k);
        }
        true
    }
    pub(super) fn or_opt(&mut self, i: usize, last: usize, gap: usize) -> bool {
        if gap.wrapping_add(1) == i || (gap >= i && gap <= last) {
            return false;
        }
        let a = if gap == usize::MAX {
            self.start
        } else {
            Some(self.exit(gap))
        };
        let b = if gap.wrapping_add(1) < self.order.len() {
            Some(self.entry(gap.wrapping_add(1)))
        } else {
            None
        };
        let removal = dist(self.prev(i), Some(self.entry(i)))
            + dist(Some(self.exit(last)), self.next(last))
            - dist(self.prev(i), self.next(last));
        let base = dist(a, b);
        let kept = dist(a, Some(self.entry(i))) + dist(Some(self.exit(last)), b) - base;
        let flipped = if self.reversible(i, last) {
            dist(a, Some(self.exit(last))) + dist(Some(self.entry(i)), b) - base
        } else {
            f64::MAX
        };
        if kept.min(flipped) > removal - 100.0 {
            return false;
        }
        let len = last - i + 1;
        let forward = gap > last && gap != usize::MAX;
        let first = if forward {
            gap + 1 - len
        } else {
            gap.wrapping_add(1)
        };
        let lo = if forward { i } else { first };
        let hi = if forward { gap } else { last };
        if forward {
            self.order[i..=gap].rotate_left(len);
        } else {
            self.order[first..=last].rotate_right(len);
        }
        if flipped < kept {
            self.order[first..first + len].reverse();
            for k in first..first + len {
                self.flipped[self.order[k]] ^= true;
            }
        }
        self.refresh(lo, hi);
        for k in [
            first.wrapping_sub(1),
            first,
            first + len - 1,
            first + len,
            if forward { i.wrapping_sub(1) } else { last },
            if forward { i } else { last + 1 },
        ] {
            self.enqueue(k);
        }
        true
    }
}
