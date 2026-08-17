use std::{cell::RefCell, rc::Rc};

use crate::arachne::skeletal::BeadingPropagation;

use super::SkeletalTrapezoidation;

impl SkeletalTrapezoidation<'_> {
    pub(super) fn store_node_beadings(&mut self) {
        let nodes = self.graph.active_nodes().collect::<Vec<_>>();
        for node in nodes {
            let data = &self.graph.node(node).data;
            if data.bead_count <= 0 || data.transition_ratio != 0.0 {
                continue;
            }
            let thickness = data.distance_to_boundary * 2;
            let beading = self.beading_strategy.compute(thickness, data.bead_count);
            assert_eq!(beading.total_thickness, thickness);
            let storage = Rc::new(RefCell::new(BeadingPropagation::new(beading)));
            self.graph.node_mut(node).data.set_beading(&storage);
            self.beading_storage.push(storage);
        }
    }
}

#[cfg(test)]
mod tests;
