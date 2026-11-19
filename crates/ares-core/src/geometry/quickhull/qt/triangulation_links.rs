//! Qt neighbor-slot checks from Qhull 8.0.2 `poly2_r.c`.
//!
//! Copyright (c) 1993-2020 The Geometry Center.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use std::collections::BTreeMap;

use super::{Hull, HullError, Id, TAIL};

type Spoke = (Id, usize, bool);

#[derive(Default)]
struct TriangulationLinks {
    neighbors: Vec<[Id; 3]>,
    nulls: Vec<Id>,
    opposing: Vec<Id>,
}

impl Hull {
    pub(super) fn check_qt_mirror(&self) -> Result<(), HullError> {
        let mut links = TriangulationLinks {
            neighbors: vec![[TAIL; 3]; self.facets.len()],
            opposing: vec![TAIL; self.ridges.len()],
            ..Default::default()
        };
        for parent in self.suffix(self.first()) {
            if !self.facets[parent].simplicial {
                self.link_qt_children(parent, &mut links)?;
                continue;
            }
            self.link_simplicial_neighbors(parent, &mut links);
        }
        for null in std::mem::take(&mut links.nulls) {
            links.splice_null(null)?;
        }
        Ok(())
    }

    fn link_simplicial_neighbors(&self, parent: Id, links: &mut TriangulationLinks) {
        for (slot, &peer) in self.facets[parent].neighbors.iter().enumerate() {
            if self.facets[peer].simplicial {
                links.neighbors[parent][slot] = peer;
            }
        }
    }

    fn link_qt_children(
        &self,
        parent: Id,
        links: &mut TriangulationLinks,
    ) -> Result<(), HullError> {
        let facet = &self.facets[parent];
        let apex = facet.vertices[0];
        let mut spokes = BTreeMap::new();
        for &ridge in &facet.ridges {
            let edge = &self.ridges[ridge];
            let [a, b] = edge.vertices;
            let node = links.append(self, parent, ridge)?;
            if a == apex {
                links.nulls.push(node);
            }
            for (slot, endpoint) in [(1, b), (2, a)] {
                let orientation = (edge.top == parent) != (slot == 1);
                links.link_spoke(&mut spokes, endpoint, (node, slot, orientation))?;
            }
        }
        Ok(())
    }
}

impl TriangulationLinks {
    fn link_spoke(
        &mut self,
        spokes: &mut BTreeMap<Id, Spoke>,
        endpoint: Id,
        spoke: Spoke,
    ) -> Result<(), HullError> {
        let Some(&(other, skip, other_orientation)) = spokes.get(&endpoint) else {
            spokes.insert(endpoint, spoke);
            return Ok(());
        };
        let (node, slot, orientation) = spoke;
        if orientation == other_orientation || self.neighbors[other][skip] != TAIL {
            return Err(HullError::DuplicateRidge);
        }
        self.neighbors[node][slot] = other;
        self.neighbors[other][skip] = node;
        Ok(())
    }

    fn append(&mut self, hull: &Hull, parent: Id, ridge: Id) -> Result<Id, HullError> {
        let node = self.neighbors.len();
        let other = hull.ridges[ridge].other(parent);
        let facet = &hull.facets[other];
        let peer = if facet.simplicial {
            let slot = facet
                .neighbors
                .iter()
                .position(|&id| id == parent)
                .ok_or(HullError::Topology)?;
            self.neighbors[other][slot] = node;
            other
        } else {
            let peer = self.opposing[ridge];
            if peer == TAIL {
                self.opposing[ridge] = node;
            } else {
                self.neighbors[peer][0] = node;
            }
            peer
        };
        self.neighbors.push([peer, TAIL, TAIL]);
        Ok(node)
    }

    fn splice_null(&mut self, null: Id) -> Result<(), HullError> {
        let [a, b, _] = self.neighbors[null];
        // qh_triangulate_link schedules MRGmirror if these facets are
        // already neighbors. The separate mirror deletion is not ported.
        if a == b || self.neighbors[a].contains(&b) || self.neighbors[b].contains(&a) {
            return Err(HullError::Topology);
        }
        let (ai, bi) = (self.slot(a, null)?, self.slot(b, null)?);
        self.neighbors[a][ai] = b;
        self.neighbors[b][bi] = a;
        Ok(())
    }

    fn slot(&self, node: Id, target: Id) -> Result<usize, HullError> {
        self.neighbors[node]
            .iter()
            .position(|&id| id == target)
            .ok_or(HullError::Topology)
    }
}
