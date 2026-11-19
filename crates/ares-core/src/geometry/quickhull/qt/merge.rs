//! Ridge and facet updates from Qhull 8.0.2 `merge_r.c`.
//!
//! Copyright (c) 1993-2020 C.B. Barber.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use super::{Hull, Id, Ridge, TAIL};

impl Ridge {
    pub(super) fn other(&self, facet: Id) -> Id {
        if self.top == facet {
            self.bottom
        } else {
            self.top
        }
    }

    fn replace(&mut self, from: Id, to: Id) {
        if self.top == from {
            self.top = to;
        } else {
            self.bottom = to;
        }
    }
}

impl Hull {
    // qh_mergecycle_all: visit initially new facets, in list order. The
    // vertex at the apex is already known; the horizon keeps its plane.
    pub(super) fn merge_horizons(&mut self) -> Option<()> {
        let start = self.new?;
        let order: Vec<_> = self.suffix(start).collect();
        for id in order {
            if self.facets[id].has_plane || self.facets[id].visible {
                continue;
            }
            let horizon = self.facets[id].neighbors[0];
            if self.facets[id].cycle_next == id {
                self.merge_facets(id, horizon, true)?;
            } else {
                let cycle = self.horizon_cycle(id)?;
                self.merge_cycle(&cycle, horizon)?;
            }
        }
        Some(())
    }

    fn horizon_cycle(&self, id: Id) -> Option<Vec<Id>> {
        let mut cycle = Vec::new();
        let mut member = self.facets[id].cycle_next;
        while member != id {
            if member == TAIL || cycle.len() >= self.facets.len() {
                return None;
            }
            if !self.facets[member].has_plane {
                cycle.push(member);
            }
            member = self.facets[member].cycle_next;
        }
        cycle.push(id);
        Some(cycle)
    }

    // qh_makeridges: preserve earlier explicit ridges, then append absent
    // ridges in simplex neighbor-slot order.
    pub(super) fn make_ridges(&mut self, id: Id) {
        if !self.facets[id].simplicial {
            return;
        }
        self.facets[id].simplicial = false;
        let neighbors = self.facets[id].neighbors.clone();
        for (slot, neighbor) in neighbors.into_iter().enumerate() {
            if !self.facets[id]
                .ridges
                .iter()
                .any(|&r| self.ridges[r].other(id) == neighbor)
            {
                self.new_ridge(id, slot, id, neighbor);
            }
        }
    }

    fn new_ridge(&mut self, source: Id, skip: usize, side: Id, neighbor: Id) {
        let simplex = &self.facets[source];
        let [x, y] = match skip {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };
        let (top, bottom) = if simplex.top ^ !skip.is_multiple_of(2) {
            (side, neighbor)
        } else {
            (neighbor, side)
        };
        let id = self.ridges.len();
        self.ridges.push(Ridge {
            vertices: [simplex.vertices[x], simplex.vertices[y]],
            top,
            bottom,
            tested: self.facets[source].tested,
            nonconvex: false,
        });
        self.facets[side].push_ridge(id);
        self.facets[neighbor].push_ridge(id);
    }

    // qh_mergefacet, qh_mergesimplex, qh_mergeneighbors, qh_mergeridges.
    pub(super) fn merge_facets(&mut self, from: Id, into: Id, horizon: bool) -> Option<()> {
        self.make_ridges(from);
        self.make_ridges(into);
        self.update_tested(from, into);
        let source_vertices = self.facets[from].vertices.clone();
        if source_vertices.len() == 3 {
            self.merge_simplex_ridges(from, into)?;
        } else {
            self.merge_polygon_ridges(from, into)?;
        }
        let dest = &mut self.facets[into].vertices;
        for &vertex in &source_vertices {
            if let Err(index) = dest.binary_search_by(|v| vertex.cmp(v)) {
                dest.insert(index, vertex);
                self.incidence[vertex].insert(into);
            }
        }
        // qh_mergevertex_del removes a source vertex that now has only
        // the destination facet, before the next centrum test.
        for vertex in source_vertices {
            if self.incidence[vertex].len() == 2
                && self.incidence[vertex].contains(&from)
                && self.incidence[vertex].contains(&into)
            {
                self.remove_vertex(into, vertex);
            }
        }
        if !horizon {
            self.check_merge_neighbors(from, into);
        }
        self.finish_merge(into, &[from]);
        Some(())
    }

    fn merge_polygon_ridges(&mut self, from: Id, into: Id) -> Option<()> {
        let into_neighbors = self.facets[into].neighbors.clone();
        for neighbor in self.facets[from].neighbors.clone() {
            if into_neighbors.contains(&neighbor) {
                self.drop_shared_neighbor(neighbor, from, into)?;
            } else if neighbor != into {
                self.append_merge_neighbor(neighbor, from, into)?;
            }
        }
        remove(&mut self.facets[into].neighbors, from);
        let mut index = 0;
        while let Some(&ridge) = self.facets[into].ridges.get(index) {
            if self.ridges[ridge].other(into) == from {
                self.delete_ridge(ridge);
            } else {
                index += 1;
            }
        }
        for ridge in std::mem::take(&mut self.facets[from].ridges) {
            self.ridges[ridge].replace(from, into);
            self.facets[into].push_ridge(ridge);
        }
        Some(())
    }

    fn append_merge_neighbor(&mut self, neighbor: Id, from: Id, into: Id) -> Option<()> {
        self.facets[into].neighbors.push(neighbor);
        replace(&mut self.facets[neighbor].neighbors, from, into)
    }

    fn check_merge_neighbors(&mut self, from: Id, into: Id) {
        self.check_redundant_neighbors(into);
        for neighbor in self.facets[from].neighbors.clone() {
            if self.facets[neighbor].neighbors.len() < 3 {
                self.add_structural_merge(neighbor, None);
            }
        }
        self.check_redundant(into);
    }

    // qh_updatetested, qh_MAXnewcentrum (5) plus hull dimension.
    fn update_tested(&mut self, from: Id, into: Id) {
        for &ridge in &self.facets[from].ridges {
            self.ridges[ridge].tested = false;
        }
        let facet = &mut self.facets[into];
        facet.tested = false;
        if facet.centrum.is_none() {
            return;
        }
        let size = facet.vertices.len();
        facet.keep_centrum = size > 8 || (size != 3 && facet.keep_centrum);
        if !facet.keep_centrum {
            facet.centrum = None;
            for &ridge in &facet.ridges {
                self.ridges[ridge].tested = false;
            }
        }
    }

    fn merge_simplex_ridges(&mut self, from: Id, into: Id) -> Option<()> {
        let mut index = 0;
        while let Some(&ridge) = self.facets[from].ridges.get(index) {
            let neighbor = self.ridges[ridge].other(from);
            if neighbor == into {
                self.delete_ridge(ridge);
                remove(&mut self.facets[into].neighbors, from);
                continue;
            }
            self.facets[into].push_ridge(ridge);
            if self.facets[into].neighbors.contains(&neighbor) {
                self.drop_shared_neighbor(neighbor, from, into)?;
            } else {
                self.append_merge_neighbor(neighbor, from, into)?;
            }
            self.ridges[ridge].replace(from, into);
            index += 1;
        }
        Some(())
    }

    fn drop_shared_neighbor(&mut self, neighbor: Id, from: Id, into: Id) -> Option<()> {
        self.make_ridges(neighbor);
        let neighbors = &mut self.facets[neighbor].neighbors;
        if neighbors.first() == Some(&from) {
            remove(neighbors, into);
            replace(neighbors, from, into)?;
        } else {
            remove(neighbors, from);
        }
        Some(())
    }

    // qh_mergecycle_neighbors and qh_mergecycle_ridges operate in two passes;
    // doing a series of pairwise merges would alter the ridge/filter order.
    fn merge_cycle(&mut self, cycle: &[Id], into: Id) -> Option<()> {
        self.make_ridges(into);
        self.facets[into].neighbors.retain(|n| !cycle.contains(n));
        for &from in cycle {
            for neighbor in self.facets[from].neighbors.clone() {
                self.merge_cycle_neighbor(neighbor, from, into, cycle)?;
            }
        }
        let removed: Vec<_> = self.facets[into]
            .ridges
            .iter()
            .copied()
            .filter(|&r| cycle.contains(&self.ridges[r].other(into)))
            .collect();
        for ridge in removed {
            self.note_ridge_loss(ridge);
        }
        self.facets[into]
            .ridges
            .retain(|&r| !cycle.contains(&self.ridges[r].other(into)));
        for &from in cycle {
            self.merge_cycle_ridges(from, into, cycle);
        }
        let apex = self.facets[cycle[0]].vertices[0];
        let extra: Vec<_> = self.facets[into]
            .vertices
            .iter()
            .copied()
            .filter(|v| {
                !self.facets[into]
                    .ridges
                    .iter()
                    .any(|&r| self.ridges[r].vertices.contains(v))
            })
            .collect();
        for vertex in extra {
            self.remove_vertex(into, vertex);
        }
        if self.facets[into].vertices.first() != Some(&apex) {
            self.facets[into].vertices.insert(0, apex);
            self.incidence[apex].insert(into);
        }
        if self.facets[into].vertices.len() <= 8 {
            self.facets[into].centrum = None;
        }
        self.finish_merge(into, cycle);
        Some(())
    }

    fn merge_cycle_neighbor(
        &mut self,
        neighbor: Id,
        from: Id,
        into: Id,
        cycle: &[Id],
    ) -> Option<()> {
        if cycle.contains(&neighbor) || neighbor == into {
            return Some(());
        }
        let exists = self.facets[into].neighbors.contains(&neighbor);
        if self.facets[neighbor].simplicial && !exists {
            self.append_merge_neighbor(neighbor, from, into)?;
            let ridge = self.facets[neighbor]
                .ridges
                .iter()
                .find(|&&r| self.ridges[r].other(neighbor) == from);
            if let Some(&ridge) = ridge {
                self.ridges[ridge].replace(from, into);
            }
            return Some(());
        }
        self.make_ridges(neighbor);
        remove(&mut self.facets[neighbor].neighbors, from);
        if !exists {
            self.facets[neighbor].neighbors.push(into);
            self.facets[into].neighbors.push(neighbor);
        }
        Some(())
    }

    fn merge_cycle_ridges(&mut self, from: Id, into: Id, cycle: &[Id]) {
        for ridge in std::mem::take(&mut self.facets[from].ridges) {
            let r = &mut self.ridges[ridge];
            if r.top != from && r.bottom != from {
                self.facets[into].push_ridge(ridge);
                continue;
            }
            let neighbor = r.other(from);
            r.replace(from, into);
            if cycle.contains(&neighbor) {
                remove(&mut self.facets[neighbor].ridges, ridge);
            } else if neighbor != into {
                self.facets[into].push_ridge(ridge);
            }
        }
        if !self.facets[from].simplicial {
            return;
        }
        for (slot, neighbor) in self.facets[from].neighbors.clone().into_iter().enumerate() {
            if !cycle.contains(&neighbor) && self.facets[neighbor].simplicial {
                self.new_ridge(from, slot, into, neighbor);
            }
        }
    }

    fn finish_merge(&mut self, into: Id, deleted: &[Id]) {
        self.merges += 1;
        self.unlink(into);
        self.append(into);
        let dest = &mut self.facets[into];
        dest.new = true;
        dest.tested = false;
        dest.new_merge = true;
        dest.simplicial = false;
        for &from in deleted {
            self.will_delete(from, into);
        }
    }

    pub(super) fn will_delete(&mut self, id: Id, replacement: Id) {
        self.unlink(id);
        self.insert_before(id, self.visible.unwrap_or(TAIL));
        self.visible = Some(id);
        for vertex in self.facets[id].vertices.clone() {
            self.incidence[vertex].remove(&id);
            self.record_lost_vertex(vertex);
        }
        let facet = &mut self.facets[id];
        facet.visible = true;
        facet.replacement = replacement;
        facet.neighbors.clear();
        facet.ridges.clear();
    }
}

fn remove(set: &mut Vec<Id>, id: Id) {
    if let Some(index) = set.iter().position(|&i| i == id) {
        set.swap_remove(index);
    }
}

fn replace(set: &mut [Id], from: Id, into: Id) -> Option<()> {
    let index = set.iter().position(|&id| id == from)?;
    set[index] = into;
    Some(())
}
