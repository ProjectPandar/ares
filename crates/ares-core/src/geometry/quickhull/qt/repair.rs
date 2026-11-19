//! Three-dimensional incidence updates and vertex reduction from Qhull 8.0.2
//! `merge_r.c` and `libqhull_r.c`.
//!
//! Copyright (c) 1993-2020 C.B. Barber.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use super::{Hull, Id, TAIL};

impl Hull {
    pub(super) fn add_structural_merge(&mut self, id: Id, into: Option<Id>) {
        if self.facets[id].visible || self.facets[id].redundant {
            return;
        }
        if let Some(other) = into {
            if self.facets[other].redundant || self.facets[other].visible {
                return;
            }
            self.facets[id].redundant = true;
            self.structural_merges.push_back((id, into));
        } else if !self.facets[id].degenerate {
            self.facets[id].degenerate = true;
            if self
                .structural_merges
                .back()
                .is_some_and(|&(_, other)| other.is_some())
            {
                self.structural_merges.push_front((id, None));
            } else {
                self.structural_merges.push_back((id, None));
            }
        }
    }

    fn subset_of(&self, id: Id, other: Id) -> bool {
        let set = &self.facets[other].vertices;
        self.facets[id]
            .vertices
            .iter()
            .all(|v| set.binary_search_by(|x| v.cmp(x)).is_ok())
    }

    pub(super) fn check_redundant_neighbors(&mut self, id: Id) {
        if self.facets[id].neighbors.len() < 3 {
            self.add_structural_merge(id, None);
            return;
        }
        for neighbor in self.facets[id].neighbors.clone() {
            let f = &self.facets[neighbor];
            if !f.degenerate
                && !f.redundant
                && (!self.facets[id].flipped || f.flipped)
                && self.subset_of(neighbor, id)
            {
                self.add_structural_merge(neighbor, Some(id));
            }
        }
    }

    pub(super) fn check_redundant(&mut self, id: Id) {
        if self.facets[id].visible || self.facets[id].flipped {
            return;
        }
        let other =
            self.facets[id].neighbors.iter().copied().find(|&n| {
                !self.facets[n].flipped && !self.facets[n].visible && self.subset_of(id, n)
            });
        if other.is_some() || self.facets[id].neighbors.len() < 3 {
            self.add_structural_merge(id, other);
        }
    }

    pub(super) fn merge_structural(&mut self) -> Option<bool> {
        let mut changed = false;
        while let Some((id, into)) = self.structural_merges.pop_back() {
            if self.facets[id].visible {
                continue;
            }
            self.facets[id].degenerate = false;
            self.facets[id].redundant = false;
            if let Some(other) = into {
                changed |= self.merge_redundant(id, other)?;
            } else if self.facets[id].neighbors.is_empty() {
                self.will_delete(id, TAIL);
                changed = true;
            } else if self.facets[id].neighbors.len() < 3 {
                let (target, _) = self.best_neighbor(id)?;
                self.merge_width(id, target)?;
                self.merge_facets(id, target, false)?;
                changed = true;
            }
        }
        Some(changed)
    }

    fn merge_redundant(&mut self, id: Id, other: Id) -> Option<bool> {
        let target = self.replacement(other)?;
        if target == TAIL {
            return None;
        }
        if target == id {
            return Some(false);
        }
        self.merge_facets(id, target, false)?;
        Some(true)
    }

    pub(super) fn replacement(&self, mut id: Id) -> Option<Id> {
        for _ in 0..self.facets.len() {
            if !self.facets[id].visible {
                return Some(id);
            }
            id = self.facets[id].replacement;
        }
        None
    }

    pub(super) fn note_ridge_loss(&mut self, ridge: Id) {
        for vertex in self.ridges[ridge].vertices {
            self.lost_ridge[vertex] = true;
        }
    }

    pub(super) fn delete_ridge(&mut self, ridge: Id) {
        let r = &self.ridges[ridge];
        let (top, bottom) = (r.top, r.bottom);
        if r.nonconvex
            && let Some(&remaining) = self.facets[top]
                .ridges
                .iter()
                .find(|&&r| r != ridge && self.ridges[r].other(top) == bottom)
        {
            self.ridges[remaining].nonconvex = true;
        }
        self.note_ridge_loss(ridge);
        for facet in [top, bottom] {
            let set = &mut self.facets[facet].ridges;
            let index = set.iter().position(|&r| r == ridge).unwrap();
            set.swap_remove(index);
        }
    }

    pub(super) fn record_lost_vertex(&mut self, vertex: Id) {
        if self.incidence[vertex].is_empty() && !self.deleted_vertices.contains(&vertex) {
            self.deleted_vertices.push(vertex);
        }
    }

    pub(super) fn remove_vertex(&mut self, facet: Id, vertex: Id) {
        let set = &mut self.facets[facet].vertices;
        let index = set.binary_search_by(|v| vertex.cmp(v)).unwrap();
        set.remove(index);
        self.incidence[vertex].remove(&facet);
        self.record_lost_vertex(vertex);
    }

    fn remove_extra_vertices(&mut self, id: Id) -> bool {
        let mut present = std::collections::BTreeSet::new();
        for &ridge in &self.facets[id].ridges {
            present.extend(self.ridges[ridge].vertices);
        }
        let extra: Vec<_> = self.facets[id]
            .vertices
            .iter()
            .copied()
            .filter(|v| !present.contains(v))
            .collect();
        for &vertex in &extra {
            self.remove_vertex(id, vertex);
        }
        !extra.is_empty()
    }

    // qh_reducevertices first removes facet-local extra vertices, then
    // processes shared vertices on new merged facets in list/vertex order.
    pub(super) fn reduce_vertices(&mut self) -> Vec<Id> {
        let new: Vec<_> = self
            .suffix(self.new.unwrap_or(TAIL))
            .filter(|&id| self.facets[id].new_merge)
            .collect();
        let mut changed = Vec::new();
        for &id in &new {
            if self.remove_extra_vertices(id) {
                changed.push(id);
            }
        }
        for id in new {
            self.reduce_shared(id, &mut changed);
        }
        changed
    }

    fn reduce_shared(&mut self, id: Id, changed: &mut Vec<Id>) {
        self.facets[id].new_merge = false;
        for vertex in self.facets[id].vertices.clone() {
            if self.lost_ridge[vertex]
                && self.incidence[vertex].len() == 2
                && let Some(other) = self.rename_shared(id, vertex)
            {
                changed.extend([other, id]);
            }
        }
    }

    // qh_rename_sharedvertex: in 3D a candidate must be on one of the
    // shared ridges and must not create a duplicate ridge.
    fn rename_shared(&mut self, id: Id, vertex: Id) -> Option<Id> {
        let other = self.incidence[vertex].iter().copied().find(|&f| f != id)?;
        let ridges: Vec<_> = self.facets[id]
            .ridges
            .iter()
            .copied()
            .filter(|&r| {
                self.ridges[r].other(id) == other && self.ridges[r].vertices.contains(&vertex)
            })
            .collect();
        let endpoint = |[a, b]: [Id; 2]| if a == vertex { b } else { a };
        let mut shared: Vec<_> = self.facets[id]
            .vertices
            .iter()
            .copied()
            .filter(|v| {
                self.facets[other]
                    .vertices
                    .binary_search_by(|p| v.cmp(p))
                    .is_ok()
            })
            .collect();
        let index = shared.iter().position(|&v| v == vertex)?;
        shared.swap_remove(index);
        let mut candidates: Vec<_> = shared
            .into_iter()
            .map(|v| {
                let count = ridges
                    .iter()
                    .filter(|&&r| endpoint(self.ridges[r].vertices) == v)
                    .count();
                (v, count)
            })
            .collect();
        // qh_find_newvertex removes vertices not in the shared ridges with
        // qh_setdelnth. This changes the input order of equal visit counts.
        let mut i = 0;
        while let Some(&(_, count)) = candidates.get(i) {
            if count == 0 {
                candidates.swap_remove(i);
            } else {
                i += 1;
            }
        }
        candidates.sort_by_key(|&(_, count)| count);
        let (renamed, _) = candidates
            .into_iter()
            .find(|&(candidate, _)| self.can_rename_ridges(candidate, vertex, &ridges))?;
        for ridge in ridges {
            let adjacent = endpoint(self.ridges[ridge].vertices);
            self.rename_ridge(ridge, adjacent, (vertex, renamed));
        }
        self.remove_vertex(id, vertex);
        self.remove_vertex(other, vertex);
        Some(other)
    }

    fn can_rename_ridges(&self, candidate: Id, vertex: Id, ridges: &[Id]) -> bool {
        self.incidence[candidate]
            .iter()
            .flat_map(|&face| &self.facets[face].ridges)
            .all(|&r| {
                let edge = self.ridges[r].vertices;
                !edge.contains(&candidate)
                    || !self.rename_duplicates_edge(vertex, candidate, ridges, edge)
            })
    }

    fn rename_duplicates_edge(
        &self,
        vertex: Id,
        candidate: Id,
        ridges: &[Id],
        edge: [Id; 2],
    ) -> bool {
        ridges.iter().any(|&old| {
            let [a, b] = self.ridges[old].vertices;
            let adjacent = if a == vertex { b } else { a };
            adjacent != candidate && edge.contains(&adjacent)
        })
    }

    fn rename_ridge(&mut self, ridge: Id, adjacent: Id, (vertex, renamed): (Id, Id)) {
        if adjacent == renamed {
            self.delete_ridge(ridge);
            return;
        }
        let r = &mut self.ridges[ridge];
        if (vertex > adjacent) != (renamed > adjacent) {
            std::mem::swap(&mut r.top, &mut r.bottom);
        }
        r.vertices = [renamed.max(adjacent), renamed.min(adjacent)];
    }
}
