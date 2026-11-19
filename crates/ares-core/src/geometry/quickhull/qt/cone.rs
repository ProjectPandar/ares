//! Horizon and cone traversal from Qhull 8.0.2 `libqhull_r.c`,
//! `poly_r.c`, and `poly2_r.c`.
//!
//! Copyright (c) 1993-2020 The Geometry Center.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use std::collections::BTreeMap;

use super::{Hull, Id, Point, TAIL, partition::Search};

impl Hull {
    pub(super) fn find_horizon(&mut self, point: Id, start: Id) {
        let sample = self.points[point];
        self.mark_visible(start);
        self.visible = Some(start);
        let visit = self.next_visit();
        self.facets[start].visit = visit;
        let mut id = start;
        while id != TAIL {
            for pos in 0..self.facets[id].neighbors.len() {
                let neighbor = self.facets[id].neighbors[pos];
                self.visit_horizon_neighbor(neighbor, sample, visit);
            }

            id = self.facets[id].next;
        }
    }

    fn visit_horizon_neighbor(&mut self, neighbor: Id, sample: Point, visit: usize) {
        let facet = &mut self.facets[neighbor];
        if facet.visit == visit {
            return;
        }
        facet.visit = visit;
        let dist = facet.distance(sample);
        if dist > self.precision.visible {
            self.mark_visible(neighbor);
            return;
        }
        let coplanar = dist >= -self.precision.visible;
        facet.coplanar_horizon = coplanar;
        if coplanar && dist > self.max_outside {
            self.max_outside = dist;
        }
    }

    fn mark_visible(&mut self, id: Id) {
        self.unlink(id);
        self.append(id);
        let facet = &mut self.facets[id];
        facet.visible = true;
        facet.replacement = TAIL;
    }

    pub(super) fn make_cone(&mut self, point: Id) -> Option<Id> {
        let apex = self.vertices.len();
        self.vertices.push(point);
        self.incidence.push(Default::default());
        self.lost_ridge.push(false);
        self.new = Some(TAIL);
        let mut id = self.visible?;
        let (mut last, mut last_ridge_facet) = (TAIL, TAIL);
        while id != TAIL && self.facets[id].visible {
            let neighbors = std::mem::take(&mut self.facets[id].neighbors);
            let ridges = std::mem::take(&mut self.facets[id].ridges);
            let mut represented = Vec::with_capacity(neighbors.len());

            // qh_makenewfacets calls the ridge maker for an allocated
            // set even after its last ridge has been removed.
            if self.facets[id].ridge_set_exists {
                last_ridge_facet = self.cone_from_ridges(id, apex, ridges, &mut represented)?;
            }
            if self.facets[id].simplicial {
                last = self.cone_from_simplex(id, apex, neighbors, &represented)?;
            }

            // qh_makenewfacets keeps the most recent result of each maker;
            // qh_getreplacement follows this pointer if that facet later merges.
            if last_ridge_facet != TAIL {
                last = last_ridge_facet;
            }
            self.facets[id].replacement = last;
            for &vertex in &self.facets[id].vertices {
                self.incidence[vertex].remove(&id);
            }
            id = self.facets[id].next;
        }
        let mut visible = self.visible.unwrap_or(TAIL);
        while visible != TAIL && self.facets[visible].visible {
            let vertices = self.facets[visible].vertices.clone();
            for vertex in vertices {
                self.record_lost_vertex(vertex);
            }
            visible = self.facets[visible].next;
        }
        Some(apex)
    }

    fn cone_from_ridges(
        &mut self,
        id: Id,
        apex: Id,
        ridges: Vec<Id>,
        represented: &mut Vec<Id>,
    ) -> Option<Id> {
        let mut cycle_first = BTreeMap::new();
        let mut last = TAIL;
        for ridge in ridges {
            let r = &self.ridges[ridge];
            let top = r.top == id;
            let horizon = r.other(id);
            let duplicate = represented.contains(&horizon);
            represented.push(horizon);
            if self.facets[horizon].visible {
                self.note_ridge_loss(ridge);
                continue;
            }
            let [a, b] = r.vertices;
            last = self.push_facet(vec![apex, a, b], vec![horizon, TAIL, TAIL], top);
            if self.facets[horizon].coplanar_horizon {
                let first = *cycle_first.entry(horizon).or_insert(last);
                self.link_horizon_cycle(last, first);
            }
            self.attach_ridge(last, (id, horizon), ridge, duplicate)?;
        }
        Some(last)
    }

    fn link_horizon_cycle(&mut self, new: Id, first: Id) {
        self.facets[new].merge_horizon = true;
        if first == new {
            self.facets[new].cycle_next = new;
        } else {
            self.facets[new].cycle_next = self.facets[first].cycle_next;
            self.facets[first].cycle_next = new;
        }
    }

    fn cone_from_simplex(
        &mut self,
        id: Id,
        apex: Id,
        neighbors: Vec<Id>,
        represented: &[Id],
    ) -> Option<Id> {
        let mut last = TAIL;
        for horizon in neighbors {
            let side = &self.facets[horizon];
            if side.visible || represented.contains(&horizon) {
                continue;
            }
            let skip = side.neighbors.iter().position(|&f| f == id)?;
            let top = (!skip.is_multiple_of(2)) == side.top;
            let vertices = [apex]
                .into_iter()
                .chain(
                    side.vertices
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &v)| (i != skip).then_some(v)),
                )
                .collect();
            let coplanar = side.coplanar_horizon;
            last = self.push_facet(vertices, vec![horizon, TAIL, TAIL], top);
            self.facets[last].merge_horizon = coplanar;
            self.facets[last].cycle_next = last;
            self.facets[horizon].neighbors[skip] = last;
        }
        Some(last)
    }

    fn attach_ridge(
        &mut self,
        new: Id,
        (from, horizon): (Id, Id),
        ridge: Id,
        duplicate: bool,
    ) -> Option<()> {
        let facet = &mut self.facets[horizon];
        if duplicate {
            if facet.simplicial {
                return None;
            }
            facet.neighbors.push(new);
        } else {
            let i = facet.neighbors.iter().position(|&k| k == from)?;
            facet.neighbors[i] = new;
        }
        if facet.simplicial {
            let i = facet.ridges.iter().position(|&r| r == ridge)?;
            facet.ridges.swap_remove(i);
            self.note_ridge_loss(ridge);
        } else {
            let r = &mut self.ridges[ridge];
            if r.top == from {
                r.top = new;
            } else {
                r.bottom = new;
            }
            self.facets[new].push_ridge(ridge);
        }
        Some(())
    }

    pub(super) fn partition_visible(&mut self, search: Search) -> Option<()> {
        let mut id = self.visible?;
        while id != TAIL && self.facets[id].visible {
            self.partition_removed_facet(id, search)?;
            id = self.facets[id].next;
        }
        let deleted = std::mem::take(&mut self.deleted_vertices);
        for vertex in deleted {
            let start = self.new.filter(|&f| f != TAIL)?;
            self.repartition_coplanar(self.vertices[vertex], start, Search::AllNew);
        }
        Some(())
    }

    fn partition_removed_facet(&mut self, id: Id, search: Search) -> Option<()> {
        let outside = std::mem::take(&mut self.facets[id].outside);
        let coplanar = std::mem::take(&mut self.facets[id].coplanar);
        if outside.is_empty() && coplanar.is_empty() {
            return Some(());
        }
        let mut start = self.replacement(id)?;
        if start == TAIL {
            start = self.new.filter(|&f| f != TAIL)?;
        }
        for point in outside {
            self.partition_point(point, start, search);
        }
        for point in coplanar {
            self.repartition_coplanar(point, start, search);
        }
        Some(())
    }

    pub(super) fn delete_visible(&mut self) {
        let mut id = self.visible.take().unwrap_or(TAIL);
        while id != TAIL && self.facets[id].visible {
            let next = self.facets[id].next;
            self.unlink(id);
            id = next;
        }
    }
}
