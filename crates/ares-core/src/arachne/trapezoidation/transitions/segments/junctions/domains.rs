use std::collections::HashSet;

use crate::arachne::skeletal::EdgeId;

use super::super::super::SkeletalTrapezoidation;

const ABSENT: usize = usize::MAX;

struct DenseEdgeSet {
    values: Vec<EdgeId>,
    positions: Vec<usize>,
}

impl DenseEdgeSet {
    fn from_ordered(edges: impl IntoIterator<Item = EdgeId>) -> Self {
        let values = edges.into_iter().collect::<Vec<_>>();
        let mut positions = vec![ABSENT; values.iter().map(|edge| edge.0 + 1).max().unwrap_or(0)];
        for (position, edge) in values.iter().copied().enumerate() {
            positions[edge.0] = position;
        }
        Self { values, positions }
    }

    fn first(&self) -> Option<EdgeId> {
        self.values.first().copied()
    }

    fn remove(&mut self, edge: EdgeId) -> bool {
        let Some(position) = self
            .positions
            .get(edge.0)
            .copied()
            .filter(|&position| position != ABSENT)
        else {
            return false;
        };
        self.values.swap_remove(position);
        self.positions[edge.0] = ABSENT;
        if let Some(moved) = self.values.get(position) {
            self.positions[moved.0] = position;
        }
        true
    }

    #[cfg(test)]
    fn values(&self) -> &[EdgeId] {
        &self.values
    }
}

impl SkeletalTrapezoidation<'_> {
    pub(in crate::arachne::trapezoidation::transitions::segments) fn connect_junctions(&mut self) {
        let mut unprocessed = DenseEdgeSet::from_ordered(
            self.graph
                .active_edges()
                .filter(|&edge| self.graph.edge(edge).prev.is_none()),
        );
        let mut passed_odd_edges = HashSet::new();
        while let Some(domain_start) = unprocessed.first() {
            self.connect_junction_domain(domain_start, &mut unprocessed, &mut passed_odd_edges);
        }
    }

    fn connect_junction_domain(
        &mut self,
        domain_start: EdgeId,
        unprocessed: &mut DenseEdgeSet,
        passed_odd_edges: &mut HashSet<EdgeId>,
    ) {
        let mut quad_start = domain_start;
        let mut force_new_path = true;
        loop {
            assert!(unprocessed.remove(quad_start));
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
