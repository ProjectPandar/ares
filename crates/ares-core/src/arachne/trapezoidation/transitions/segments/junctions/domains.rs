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
        let mut passed_odd_edges = HashSet::new();
        while let Some(domain_start) = unprocessed.iter().next().copied() {
            self.connect_junction_domain(domain_start, &mut unprocessed, &mut passed_odd_edges);
        }
    }

    fn connect_junction_domain(
        &mut self,
        domain_start: EdgeId,
        unprocessed: &mut HashSet<EdgeId>,
        passed_odd_edges: &mut HashSet<EdgeId>,
    ) {
        let mut quad_start = domain_start;
        let mut force_new_path = true;
        loop {
            assert!(unprocessed.remove(&quad_start));
            self.connect_quad_junctions_with_passed(quad_start, force_new_path, passed_odd_edges);
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
