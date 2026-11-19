//! Cone spoke matching from Qhull 8.0.2 `poly_r.c`.
//!
//! Copyright (c) 1993-2020 C.B. Barber.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use std::collections::BTreeMap;

use super::{Hull, HullError, Id, TAIL};

impl Hull {
    pub(super) fn match_cone(&mut self) -> Result<(), HullError> {
        let mut spokes: BTreeMap<Id, (Id, usize)> = BTreeMap::new();
        let start = self.new.unwrap_or(TAIL);
        let mut id = start;
        while id != TAIL {
            for slot in 1..3 {
                let edge = self.facets[id].vertices[3 - slot];
                self.match_spoke(&mut spokes, edge, (id, slot))?;
            }
            id = self.facets[id].next;
        }
        let mut id = start;
        while id != TAIL {
            if self.facets[id].neighbors.contains(&TAIL) {
                return Err(HullError::Topology);
            }
            if !self.facets[id].merge_horizon {
                self.set_plane(id);
            }
            id = self.facets[id].next;
        }
        Ok(())
    }

    fn match_spoke(
        &mut self,
        spokes: &mut BTreeMap<Id, (Id, usize)>,
        edge: Id,
        (id, slot): (Id, usize),
    ) -> Result<(), HullError> {
        let Some(&(other, skip)) = spokes.get(&edge) else {
            spokes.insert(edge, (id, slot));
            return Ok(());
        };
        let opposite = (slot == skip) != (self.facets[id].top == self.facets[other].top);
        // qh_matchneighbor marks these for qh_matchdupridge and forced merging.
        if !opposite || self.facets[other].neighbors[skip] != TAIL {
            return Err(HullError::DuplicateRidge);
        }
        self.facets[other].neighbors[skip] = id;
        self.facets[id].neighbors[slot] = other;
        Ok(())
    }
}
