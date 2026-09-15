//! Candidate point search over unchained segments (greedy-2 / two-opt
//! inner loops; upstream `ShortestPath.cpp`).

use super::{EndPoint, EquivalentChains, KdTree, MutablePriorityQueue, squared_distance};

pub(super) struct CandidateSearch<'a> {
    pub(super) tree: &'a KdTree,
    pub(super) positions: &'a [[f64; 2]],
}

impl CandidateSearch<'_> {
    pub(super) fn update(
        &self,
        first: usize,
        endpoints: &mut [EndPoint],
        equivalents: &mut EquivalentChains,
        queue: &mut MutablePriorityQueue,
    ) {
        endpoints[first].edge_out = None;
        let next = self
            .tree
            .find_closest(self.positions, self.positions[first], |candidate| {
                if (candidate ^ first) <= 1 || endpoints[candidate].chain_id != 0 {
                    return false;
                }
                let chain1 = equivalents.equivalent(endpoints[first ^ 1].chain_id);
                let chain2 = equivalents.equivalent(endpoints[candidate ^ 1].chain_id);
                chain1 == 0 || chain1 != chain2
            });
        endpoints[first].edge_out = Some(next);
        endpoints[first].distance_out =
            squared_distance(self.positions[first], self.positions[next]);
        queue.update(endpoints[first].heap_idx, endpoints);
    }
}
