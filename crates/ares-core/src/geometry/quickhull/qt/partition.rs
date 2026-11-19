//! Ordered point ownership from Qhull 8.0.2 `libqhull_r.c` and `geom_r.c`.
//!
//! Copyright (c) 1993-2020 The Geometry Center.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use super::{Facet, Hull, Id, Point, TAIL, dot};

struct HorizonSearch {
    best: (Id, f64),
    limit: f64,
    span: f64,
    check_max: bool,
    remaining: Vec<Id>,
    next: Option<Id>,
}

impl HorizonSearch {
    fn visit(&mut self, id: Id, distance: Option<f64>) {
        if let Some(dist) = distance {
            if dist > self.best.1 {
                self.improve(id, dist);
            } else if dist < self.limit {
                return;
            }
        }
        if let Some(previous) = self.next.replace(id) {
            self.remaining.push(previous);
        }
    }

    fn improve(&mut self, id: Id, dist: f64) {
        if !self.check_max {
            self.limit = dist - self.span;
            if dist > self.best.1 + self.span {
                self.remaining.clear();
            }
        }
        self.best = (id, dist);
    }
}

#[derive(Clone, Copy)]
pub(super) enum Search {
    AllNew,
    Neighbor,
}

impl Hull {
    pub(super) fn partition_initial(&mut self, seed: &[Id; 4]) {
        let mut remaining: Vec<_> = (0..self.points.len())
            .filter(|p| !seed.contains(p))
            .collect();
        let limit = self.outside_limit();
        let mut id = self.first();
        while id != TAIL {
            let facet = &mut self.facets[id];
            let mut best = None;
            remaining.retain(|&point| {
                let distance = facet.distance(self.points[point]);
                Self::partition_initial_point(facet, point, (distance, limit), &mut best)
            });
            facet.outside.extend(best);
            id = facet.next;
        }
        for point in remaining {
            self.partition_point(point, self.first(), Search::AllNew);
        }
    }

    pub(super) fn partition_point(&mut self, point: Id, start: Id, search: Search) {
        let (id, dist) = if self.find_best_new || matches!(search, Search::AllNew) {
            self.find_new(point, start, false)
        } else {
            self.find_best(point, start, false, true)
        };
        let outside = self.precision.outside;
        if dist >= outside || (self.narrow && dist >= -self.precision.near_inside) {
            let facet = &mut self.facets[id];
            let mut new_outside = false;
            if facet.outside.is_empty() {
                facet.outside.push(point);
                facet.furthest = dist;
                new_outside = !self.narrow || dist > outside;
            } else if facet.furthest < dist {
                facet.outside.push(point);
                new_outside = self.narrow && facet.furthest < outside && dist >= outside;
                facet.furthest = dist;
            } else {
                insert_penultimate(&mut facet.outside, point);
            }
            if new_outside && self.cursor != id {
                self.move_outside_facet(id);
            }
        } else if dist >= -self.precision.near_inside {
            self.keep_coplanar(point, id, dist);
        }
    }

    fn move_outside_facet(&mut self, id: Id) {
        if self.facets[id].new {
            if self.facets[self.cursor].new {
                self.cursor = self.new.unwrap_or(TAIL);
            }
            return;
        }
        self.unlink(id);
        self.append(id);
        if self.new.is_some() {
            self.facets[id].new = true;
        }
    }

    pub(super) fn repartition_coplanar(&mut self, point: Id, start: Id, search: Search) {
        let (id, dist) = if self.find_best_new || matches!(search, Search::AllNew) {
            self.find_new(point, start, true)
        } else {
            self.find_best(point, start, true, false)
        };
        if dist < -self.precision.near_inside {
            return;
        }
        let from = self.facets[start].plane.normal;
        let to = self.facets[id].plane.normal;
        if dist > self.max_outside
            && ((start != id && (0.0 + dot(from, to)) < 0.0) || dist > 30.0 * self.outer_limit())
        {
            let previous = std::mem::take(&mut self.find_best_new);
            self.partition_point(point, id, Search::Neighbor);
            self.find_best_new = previous;
        } else {
            self.keep_coplanar(point, id, dist);
        }
    }

    pub(super) fn keep_coplanar(&mut self, point: Id, id: Id, dist: f64) {
        if dist > self.max_outside {
            self.max_outside = dist;
        }
        let facet = &mut self.facets[id];
        facet.max_outside = facet.max_outside.max(dist);
        if facet
            .coplanar
            .last()
            .is_some_and(|&last| facet.distance(self.points[last]) >= dist)
        {
            insert_penultimate(&mut facet.coplanar, point);
        } else {
            facet.coplanar.push(point);
        }
    }

    fn outside_limit(&self) -> f64 {
        self.search_multiplier() * (2.0 * self.precision.outside).max(self.max_outside)
    }

    fn search_multiplier(&self) -> f64 {
        if self.merges > 50 { 2.0 } else { 1.0 }
    }

    fn outer_limit(&self) -> f64 {
        self.snapshot_outside
    }

    // qh_findbest: ordered greedy walk, then a search of the near horizon.
    pub(super) fn find_best(
        &mut self,
        point: Id,
        start: Id,
        best_outside: bool,
        new_only: bool,
    ) -> (Id, f64) {
        let sample = self.points[point];
        let mut best = (start, -f64::MAX / 2.0);
        let mut has_best = false;
        let cycle = self.next_visit();
        let mut current = start;
        self.facets[start].visit = cycle;
        if !self.facets[start].flipped {
            best.1 = self.facets[start].distance(sample);
            if !best_outside && best.1 >= self.precision.outside {
                return best;
            }
            has_best = true;
        }
        while let Some(advance) =
            self.find_better_neighbor(current, sample, (cycle, new_only), best.1)
        {
            best = advance;
            if !best_outside && best.1 >= self.precision.outside {
                return best;
            }
            has_best = true;
            current = advance.0;
        }
        if new_only && !has_best {
            return self.find_new(point, self.new.unwrap_or(start), best_outside);
        }
        if new_only && !self.not_sharp && best.1 < -self.precision.round {
            if self.sharp_new_facets() {
                self.find_best_new = true;
                return self.find_new(point, best.0, best_outside);
            }
            self.not_sharp = true;
        }
        self.find_horizon_best(sample, best, false)
    }

    fn find_better_neighbor(
        &mut self,
        from: Id,
        sample: Point,
        (cycle, new_only): (usize, bool),
        best: f64,
    ) -> Option<(Id, f64)> {
        for i in 0..self.facets[from].neighbors.len() {
            let id = self.facets[from].neighbors[i];
            let f = &mut self.facets[id];
            if f.visit == cycle || (new_only && !f.new) {
                continue;
            }
            f.visit = cycle;
            if f.flipped {
                continue;
            }
            let distance = f.distance(sample);
            if distance > best {
                return Some((id, distance));
            }
        }
        None
    }

    fn sharp_new_facets(&self) -> bool {
        let Some(new) = self.new.filter(|&id| id != TAIL) else {
            return false;
        };
        let sign = self.facets[new].plane.normal.map(|x| x > 0.0);
        self.suffix(new)
            .skip(1)
            .any(|id| self.facets[id].plane.normal.map(|x| x > 0.0) != sign)
    }

    fn find_new(&mut self, point: Id, start: Id, best_outside: bool) -> (Id, f64) {
        let sample = self.points[point];
        let limit = self.outside_limit();
        let mut best = (start, -f64::MAX / 2.0);
        for id in self.suffix(start).chain(
            self.suffix(self.new.unwrap_or(TAIL))
                .take_while(|&id| id != start),
        ) {
            let facet = &self.facets[id];
            if facet.flipped {
                continue;
            }
            let dist = facet.distance(sample);
            let better = dist > best.1;
            if better {
                best = (id, dist);
            }
            if better && !best_outside && dist >= limit {
                return best;
            }
        }
        self.find_horizon_best(sample, best, false)
    }

    pub(super) fn find_horizon_best(
        &mut self,
        sample: Point,
        best: (Id, f64),
        check_max: bool,
    ) -> (Id, f64) {
        let span = self.search_multiplier()
            * (self.max_outside + 2.0 * self.precision.round + self.precision.visible);
        // The final max-outside check still visits coplanar facets when an
        // outside facet has already been found (qh_IScheckmax).
        let limit = if check_max {
            best.1.min(0.0) - span
        } else {
            best.1 - span
        };
        let cycle = self.next_visit();
        self.facets[best.0].visit = cycle;
        let mut walk = HorizonSearch {
            best,
            limit,
            span,
            check_max,
            remaining: Vec::new(),
            next: Some(best.0),
        };
        while let Some(id) = walk.next.take().or_else(|| walk.remaining.pop()) {
            for i in 0..self.facets[id].neighbors.len() {
                let neighbor = self.facets[id].neighbors[i];
                self.visit_horizon_candidate(neighbor, sample, cycle, &mut walk);
            }
        }
        walk.best
    }

    fn visit_horizon_candidate(
        &mut self,
        id: Id,
        sample: Point,
        cycle: usize,
        walk: &mut HorizonSearch,
    ) {
        let f = &mut self.facets[id];
        if f.visit != cycle {
            f.visit = cycle;
            walk.visit(id, (!f.flipped).then(|| f.distance(sample)));
        }
    }

    fn partition_initial_point(
        facet: &mut Facet,
        point: Id,
        (distance, limit): (f64, f64),
        best: &mut Option<Id>,
    ) -> bool {
        if distance < limit {
            return true;
        }
        if best.is_none() || distance > facet.furthest {
            if let Some(old) = best.replace(point) {
                facet.outside.push(old);
            }
            facet.furthest = distance;
        } else {
            facet.outside.push(point);
        }
        false
    }

    pub(super) fn next_visit(&mut self) -> usize {
        self.visit += 1;
        self.visit
    }
}

fn insert_penultimate(set: &mut Vec<Id>, point: Id) {
    let last = set.len() - 1;
    let previous = std::mem::replace(&mut set[last], point);
    set.push(previous);
}
