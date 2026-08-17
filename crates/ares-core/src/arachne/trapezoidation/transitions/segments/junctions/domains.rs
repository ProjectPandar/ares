use std::collections::HashSet;

use crate::arachne::skeletal::EdgeId;

use super::super::super::SkeletalTrapezoidation;

impl SkeletalTrapezoidation<'_> {
    pub(super) fn connect_junctions(&mut self) {
        let mut unprocessed = self
            .graph
            .active_edges()
            .filter(|&edge| self.graph.edge(edge).prev.is_none())
            .collect::<HashSet<_>>();
        while let Some(domain_start) = unprocessed.iter().next().copied() {
            self.connect_junction_domain(domain_start, &mut unprocessed);
        }
    }

    fn connect_junction_domain(&mut self, domain_start: EdgeId, unprocessed: &mut HashSet<EdgeId>) {
        let mut quad_start = domain_start;
        let mut force_new_path = true;
        loop {
            assert!(unprocessed.remove(&quad_start));
            self.connect_quad_junctions(quad_start, force_new_path);
            force_new_path = false;
            quad_start = self.graph.next_unconnected(quad_start).unwrap();
            if quad_start == domain_start {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests;
