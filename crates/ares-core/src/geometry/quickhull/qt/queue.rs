//! Default 3D premerge queue from Qhull 8.0.2 `merge_r.c`.
//!
//! Copyright (c) 1993-2020 C.B. Barber.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use std::{cmp::Ordering, collections::BTreeSet};

use super::{Hull, Id, Point, TAIL};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Kind {
    Coplanar,
    Concave,
    ConcaveCoplanar,
}

struct Merge {
    pair: [Id; 2],
    kind: Kind,
    distance: f64,
}

impl Hull {
    pub(super) fn premerge(&mut self) -> Option<()> {
        if self.check_zero() {
            return Some(());
        }
        self.merge_horizons()?;
        let horizons: Vec<_> = self
            .suffix(self.new.unwrap_or(TAIL))
            .filter(|&f| self.facets[f].coplanar_horizon)
            .collect();
        for id in horizons {
            self.check_redundant_neighbors(id);
            self.facets[id].coplanar_horizon = false;
        }
        self.merge_structural()?;

        let flipped: Vec<_> = self
            .suffix(self.new.unwrap_or(TAIL))
            .filter(|&id| self.facets[id].flipped)
            .collect();
        for id in flipped {
            if !self.facets[id].visible {
                let (into, _) = self.best_neighbor(id)?;
                self.merge_width(id, into)?;
                self.merge_facets(id, into, false)?;
            }
        }
        self.merge_structural()?;
        let mut queue = self.merge_set(true);
        loop {
            while let Some(merge) = queue.pop() {
                self.merge_candidate(merge)?;
            }
            queue = self.merge_set(false);
            if !queue.is_empty() {
                continue;
            }
            if !self.reduce_and_merge()? {
                break;
            }
            queue = self.merge_set(false);
        }
        Some(())
    }

    fn merge_candidate(
        &mut self,
        Merge {
            pair: [mut a, mut b],
            kind,
            ..
        }: Merge,
    ) -> Option<()> {
        let fa = &self.facets[a];
        let fb = &self.facets[b];
        if fa.visible
            || fb.visible
            || (kind == Kind::Coplanar && ((fa.new && !fa.tested) || (fb.new && !fb.tested)))
        {
            return Some(());
        }
        if !fa.new {
            std::mem::swap(&mut a, &mut b);
        }
        let (na, da) = self.best_neighbor(a)?;
        let (nb, db) = self.best_neighbor(b)?;
        let (from, into) = if da < db { (a, na) } else { (b, nb) };
        self.merge_width(from, into)?;
        self.merge_facets(from, into, false)?;
        self.merge_structural()?;
        Some(())
    }

    fn reduce_and_merge(&mut self) -> Option<bool> {
        for changed in self.reduce_vertices() {
            self.check_redundant(changed);
        }
        self.merge_structural()
    }

    // qh_checkzero tests individual vertices before creating centrums.
    fn check_zero(&self) -> bool {
        let start = self.new.unwrap_or(TAIL);
        let limit = -2.0 * self.precision.round;
        self.suffix(start).all(|id| {
            let facet = &self.facets[id];
            let horizon = &self.facets[facet.neighbors[0]];
            !facet.merge_horizon && !facet.flipped && horizon.simplicial
        }) && self.suffix(start).all(|id| {
            let facet = &self.facets[id];
            let horizon = &self.facets[facet.neighbors[0]];
            (1..3).all(|slot| {
                self.facets[facet.neighbors[slot]].distance(self.vertex(facet.vertices[slot]))
                    < limit
            }) && horizon
                .vertices
                .iter()
                .find(|v| !facet.vertices[1..].contains(v))
                .is_some_and(|&v| facet.distance(self.vertex(v)) < limit)
        })
    }

    fn centrum(&mut self, id: Id) -> Point {
        let facet = &self.facets[id];
        if let Some(centrum) = facet.centrum {
            return centrum;
        }
        let count = facet.vertices.len() as f64;
        let mut center = [0.0; 3];
        for &vertex in &facet.vertices {
            let point = self.vertex(vertex);
            for axis in 0..3 {
                center[axis] += point[axis];
            }
        }
        center = center.map(|coord| coord / count);
        let dist = facet.distance(center);
        let centrum = std::array::from_fn(|axis| center[axis] - dist * facet.plane.normal[axis]);
        self.facets[id].centrum = Some(centrum);
        centrum
    }

    fn test_pair(&mut self, mut a: Id, mut b: Id) -> Option<Merge> {
        if self.facets[a].redundant || self.facets[b].redundant {
            return None;
        }
        let p = self.centrum(a);
        let q = self.centrum(b);
        let d = self.facets[b].distance(p);
        let e = self.facets[a].distance(q);
        let radius = 2.0 * self.precision.round;
        let concave = d > radius || e > radius;
        let coplanar = (-radius..=radius).contains(&d) || (-radius..=radius).contains(&e);
        let (kind, distance) = match (concave, coplanar) {
            (true, true) => {
                if d <= e {
                    std::mem::swap(&mut a, &mut b);
                }
                (Kind::ConcaveCoplanar, d.max(e))
            }
            (true, false) => (Kind::Concave, d.max(e)),
            (false, true) => (Kind::Coplanar, d.abs().min(e.abs())),
            (false, false) => return None,
        };
        Some(Merge {
            pair: [a, b],
            kind,
            distance,
        })
    }

    fn merge_set(&mut self, initial: bool) -> Vec<Merge> {
        let mut result = Vec::new();
        let visit = self.next_visit();
        let mut id = self.new.unwrap_or(TAIL);
        while id != TAIL {
            if initial || !self.facets[id].tested {
                self.facets[id].visit = visit;
                self.facet_candidates(id, visit, initial, &mut result);
                self.facets[id].tested = true;
            }
            id = self.facets[id].next;
        }
        // Qhull's comparator takes the left element on equal keys in the
        // reference libc's mergesort. Preserve insertion order before popping.
        result.sort_by(|a, b| {
            b.kind.cmp(&a.kind).then_with(|| {
                b.distance
                    .partial_cmp(&a.distance)
                    .unwrap_or(Ordering::Equal)
            })
        });
        result
    }

    fn facet_candidates(&mut self, id: Id, visit: usize, initial: bool, result: &mut Vec<Merge>) {
        if initial {
            self.initial_candidates(id, visit, result);
            return;
        }
        let mut seen = BTreeSet::new();
        for index in 0..self.facets[id].ridges.len() {
            let ridge = self.facets[id].ridges[index];
            if self.ridges[ridge].tested && !self.ridges[ridge].nonconvex {
                continue;
            }
            let neighbor = self.ridges[ridge].other(id);
            let merged = if seen.contains(&neighbor) {
                None
            } else if self.facets[neighbor].visit != visit {
                seen.insert(neighbor);
                self.test_pair(id, neighbor)
            } else {
                continue;
            };
            self.ridges[ridge].nonconvex = merged.is_some();
            self.ridges[ridge].tested = true;
            result.extend(merged);
        }
    }

    fn initial_candidates(&mut self, id: Id, visit: usize, result: &mut Vec<Merge>) {
        for index in 0..self.facets[id].neighbors.len() {
            let neighbor = self.facets[id].neighbors[index];
            if self.facets[neighbor].visit == visit {
                continue;
            }
            let Some(merged) = self.test_pair(id, neighbor) else {
                continue;
            };
            let ridge = self.facets[neighbor]
                .ridges
                .iter()
                .find(|&&r| self.ridges[r].other(neighbor) == id);
            if let Some(&ridge) = ridge {
                self.ridges[ridge].nonconvex = true;
            }
            result.push(merged);
        }
        for &ridge in &self.facets[id].ridges {
            self.ridges[ridge].tested = true;
        }
    }

    pub(super) fn best_neighbor(&mut self, id: Id) -> Option<(Id, f64)> {
        let size = self.facets[id].vertices.len();
        let centrum = (size > 26).then(|| self.centrum(id));
        let facet = &self.facets[id];
        let mut best = None;
        let mut best_distance = f64::MAX;
        let nonconvex = facet
            .ridges
            .iter()
            .filter(|&&ridge| self.ridges[ridge].nonconvex)
            .map(|&ridge| self.ridges[ridge].other(id));
        let all = facet.neighbors.iter().copied();
        for neighbor in (size > 18)
            .then_some(nonconvex)
            .into_iter()
            .flatten()
            .chain(all)
        {
            if facet.flipped
                && self.facets[neighbor].flipped
                && best.is_some_and(|f: Id| !self.facets[f].flipped)
            {
                continue;
            }
            if size > 18
                && best.is_some()
                && self.facets[id]
                    .ridges
                    .iter()
                    .any(|&r| self.ridges[r].nonconvex)
                && !facet
                    .ridges
                    .iter()
                    .any(|&r| self.ridges[r].nonconvex && self.ridges[r].other(id) == neighbor)
            {
                continue;
            }
            let other = &self.facets[neighbor];
            let distance = if let Some(center) = centrum {
                (3.0 * other.distance(center)).abs()
            } else {
                facet
                    .vertices
                    .iter()
                    .filter(|v| !other.vertices.contains(v))
                    .map(|&v| other.distance(self.vertex(v)).abs())
                    .fold(0.0_f64, f64::max)
            };
            if distance < best_distance {
                best = Some(neighbor);
                best_distance = distance;
            }
        }
        best.map(|f| (f, best_distance))
    }
}
