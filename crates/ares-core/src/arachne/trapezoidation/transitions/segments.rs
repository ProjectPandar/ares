use std::{cell::RefCell, rc::Rc};

use crate::arachne::{beading::base::Beading, skeletal::BeadingPropagation};

use super::SkeletalTrapezoidation;

impl SkeletalTrapezoidation<'_> {
    pub(super) fn store_node_beadings(&mut self) {
        let nodes = self.graph.active_nodes().collect::<Vec<_>>();
        for node in nodes {
            let data = &self.graph.node(node).data;
            if data.bead_count <= 0 {
                continue;
            }
            let thickness = data.distance_to_boundary * 2;
            let beading = if data.transition_ratio == 0.0 {
                self.beading_strategy.compute(thickness, data.bead_count)
            } else {
                let lower = self.beading_strategy.compute(thickness, data.bead_count);
                let higher = self
                    .beading_strategy
                    .compute(thickness, data.bead_count + 1);
                interpolate_beading(&lower, 1.0 - f64::from(data.transition_ratio), &higher)
            };
            assert_eq!(beading.total_thickness, thickness);
            let storage = Rc::new(RefCell::new(BeadingPropagation::new(beading)));
            self.graph.node_mut(node).data.set_beading(&storage);
            self.beading_storage.push(storage);
        }
    }
}

fn interpolate_beading(left: &Beading, left_ratio: f64, right: &Beading) -> Beading {
    assert!((0.0..=1.0).contains(&left_ratio));
    let right_ratio = 1.0 - left_ratio;
    let mut result = if left.total_thickness > right.total_thickness {
        left.clone()
    } else {
        right.clone()
    };
    let shared_count = left.bead_widths.len().min(right.bead_widths.len());
    for index in 0..shared_count {
        result.bead_widths[index] = if left.bead_widths[index] == 0 || right.bead_widths[index] == 0
        {
            0
        } else {
            (left_ratio * left.bead_widths[index] as f64
                + right_ratio * right.bead_widths[index] as f64) as i64
        };
        result.toolpath_locations[index] = (left_ratio * left.toolpath_locations[index] as f64
            + right_ratio * right.toolpath_locations[index] as f64)
            as i64;
    }
    result
}

#[cfg(test)]
mod tests;
