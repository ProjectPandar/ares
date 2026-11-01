//! ShortestPath.cpp:1198–1462, reference-start extrusion entity improvement.
mod moves;
mod nearest;
mod search;
use nearest::Nearest;
use std::collections::VecDeque;

pub(super) fn improve(
    chain: &mut Vec<(usize, bool)>,
    points: &[[f64; 2]],
    reversible: &[bool],
    start: Option<[f64; 2]>,
    epsilon: f64,
) {
    if chain.len() <= 1 {
        return;
    }
    if chain.len() <= 8 {
        *chain = crate::geometry::chain_segments_exact(points, Some(reversible), start);
        return;
    }
    local(chain, points, reversible, start, epsilon);
    if let Some(start) = start {
        let nearest = Nearest::new(points);
        let mut taken = vec![false; reversible.len()];
        let mut at = start;
        let mut candidate = Vec::with_capacity(chain.len());
        while candidate.len() < reversible.len() {
            let e = nearest.query::<1>(at, |i| {
                !taken[i / 2] && (i.is_multiple_of(2) || reversible[i / 2])
            })[0];
            taken[e / 2] = true;
            candidate.push((e / 2, !e.is_multiple_of(2)));
            at = points[e ^ 1];
        }
        if travel(&candidate, points, start) < travel(chain, points, start) - epsilon {
            local(&mut candidate, points, reversible, Some(start), epsilon);
            *chain = candidate;
        }
    }
}
fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let x = a[0] - b[0];
    let y = a[1] - b[1];
    (x * x + y * y).sqrt()
}
fn travel(chain: &[(usize, bool)], points: &[[f64; 2]], mut at: [f64; 2]) -> f64 {
    let mut result = 0.0;
    for &(i, flip) in chain {
        result += distance(at, points[2 * i + usize::from(flip)]);
        at = points[2 * i + usize::from(!flip)];
    }
    result
}
struct Search<'a> {
    points: &'a [[f64; 2]],
    reversible: &'a [bool],
    start: Option<[f64; 2]>,
    epsilon: f64,
    order: Vec<usize>,
    pos: Vec<usize>,
    fixed: Vec<usize>,
    flipped: Vec<bool>,
    neighbors: Vec<[usize; 32]>,
    start_neighbors: [usize; 32],
    queue: VecDeque<usize>,
    queued: Vec<bool>,
}
impl Search<'_> {
    fn refresh(&mut self, lo: usize, hi: usize) {
        for k in lo..=hi {
            self.pos[self.order[k]] = k;
            self.fixed[k + 1] = self.fixed[k] + usize::from(!self.reversible[self.order[k]]);
        }
    }
    fn entry_idx(&self, k: usize) -> usize {
        2 * self.order[k] + usize::from(self.flipped[self.order[k]])
    }
    fn exit_idx(&self, k: usize) -> usize {
        2 * self.order[k] + usize::from(!self.flipped[self.order[k]])
    }
    fn entry(&self, k: usize) -> [f64; 2] {
        self.points[self.entry_idx(k)]
    }
    fn exit(&self, k: usize) -> [f64; 2] {
        self.points[self.exit_idx(k)]
    }
    fn is_exit(&self, e: usize) -> bool {
        !e.is_multiple_of(2) != self.flipped[e / 2]
    }
    fn reversible(&self, i: usize, j: usize) -> bool {
        self.fixed[j + 1] == self.fixed[i]
    }
    fn prev(&self, k: usize) -> Option<[f64; 2]> {
        if k > 0 {
            Some(self.exit(k - 1))
        } else {
            self.start
        }
    }
    fn next(&self, k: usize) -> Option<[f64; 2]> {
        if k + 1 < self.order.len() {
            Some(self.entry(k + 1))
        } else {
            None
        }
    }
    fn enqueue(&mut self, k: usize) {
        if k < self.order.len() && !self.queued[self.order[k]] {
            self.queued[self.order[k]] = true;
            self.queue.push_back(self.order[k]);
        }
    }
    fn closer(&self, at: [f64; 2], q: usize, limit: f64) -> bool {
        q != usize::MAX && distance(self.points[q], at) < limit
    }
}
fn dist(a: Option<[f64; 2]>, b: Option<[f64; 2]>) -> f64 {
    a.zip(b).map_or(0.0, |(a, b)| distance(a, b))
}
fn local(
    chain: &mut [(usize, bool)],
    points: &[[f64; 2]],
    reversible: &[bool],
    start: Option<[f64; 2]>,
    epsilon: f64,
) {
    let n = chain.len();
    let nearest = Nearest::new(points);
    let mut state = Search {
        points,
        reversible,
        start,
        epsilon,
        order: chain.iter().map(|p| p.0).collect(),
        pos: vec![0; n],
        fixed: vec![0; n + 1],
        flipped: vec![false; n],
        neighbors: (0..points.len())
            .map(|e| nearest.query(points[e], |i| i / 2 != e / 2))
            .collect(),
        start_neighbors: start.map_or([usize::MAX; 32], |p| nearest.query(p, |_| true)),
        queue: VecDeque::new(),
        queued: vec![false; n],
    };
    for &(i, f) in chain.iter() {
        state.flipped[i] = f;
    }
    state.refresh(0, n - 1);
    for k in 0..n {
        state.enqueue(k);
    }
    let mut moved = false;
    let mut moves = 0;
    while moves < 50 * n {
        if state.queue.is_empty() {
            if !moved {
                break;
            }
            moved = false;
            for k in 0..n {
                state.enqueue(k);
            }
        }
        let segment = state.queue.pop_front().unwrap();
        state.queued[segment] = false;
        let p = state.pos[segment];
        if state.improve_at(p) {
            moves += 1;
            moved = true;
            state.enqueue(state.pos[segment]);
        }
    }
    for (k, p) in chain.iter_mut().enumerate() {
        *p = (state.order[k], state.flipped[state.order[k]]);
    }
}
