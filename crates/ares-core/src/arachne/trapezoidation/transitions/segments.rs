use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use crate::arachne::{
    beading::base::Beading,
    skeletal::{BeadingPropagation, EdgeId},
};

use super::{SkeletalTrapezoidation, point_distance};

impl SkeletalTrapezoidation<'_> {
    pub(super) fn propagate_node_beadings(&mut self) {
        let upward_quad_mids = self.upward_quad_mids();
        self.store_node_beadings();
        self.propagate_beadings_upward(&upward_quad_mids);
        self.propagate_beadings_downward(&upward_quad_mids);
    }

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

    pub(super) fn upward_quad_mids(&self) -> Vec<EdgeId> {
        let mut edges = self
            .graph
            .active_edges()
            .filter(|&edge| {
                let edge_data = self.graph.edge(edge);
                edge_data.prev.is_some()
                    && edge_data.next.is_some()
                    && self.graph.edge_is_upward(edge)
            })
            .collect::<Vec<_>>();
        edges.sort_by(|left, right| self.compare_upward_quad_mids(*left, *right));
        edges
    }

    pub(super) fn propagate_beadings_upward(&mut self, upward_quad_mids: &[EdgeId]) {
        for &edge in upward_quad_mids.iter().rev() {
            let half_edge = self.graph.edge(edge);
            let from = half_edge.from.unwrap();
            let to = half_edge.to.unwrap();
            if self.graph.node(to).data.bead_count >= 0 {
                continue;
            }
            let Some(lower_storage) = self.graph.node(from).data.beading() else {
                continue;
            };
            if self.graph.node(to).data.has_beading() {
                continue;
            }
            let mut upper = lower_storage.borrow().clone();
            upper.dist_to_bottom_source +=
                point_distance(self.graph.node(from).point, self.graph.node(to).point);
            upper.is_upward_propagated_only = true;
            assert!(
                upper.beading.total_thickness <= self.graph.node(to).data.distance_to_boundary * 2
            );
            let storage = Rc::new(RefCell::new(upper));
            self.graph.node_mut(to).data.set_beading(&storage);
            self.beading_storage.push(storage);
        }
    }

    pub(super) fn propagate_beadings_downward(&mut self, upward_quad_mids: &[EdgeId]) {
        for &edge in upward_quad_mids {
            if self.graph.edge(edge).data.is_central() {
                continue;
            }
            let half_edge = self.graph.edge(edge);
            let from = half_edge.from.unwrap();
            let to = half_edge.to.unwrap();
            let edge_to_peak = if self.graph.node(from).data.distance_to_boundary
                == self.graph.node(to).data.distance_to_boundary
                && self.graph.node(from).data.has_beading()
                && !self.graph.node(to).data.has_beading()
            {
                half_edge.twin.unwrap()
            } else {
                edge
            };
            let lower = self.graph.edge(edge_to_peak).from.unwrap();
            let peak = self.graph.edge(edge_to_peak).to.unwrap();
            assert!(self.graph.node(peak).data.has_beading());
            if self.graph.node(lower).data.has_beading() {
                self.merge_beading_downward(edge_to_peak);
            } else {
                self.propagate_beading_downward_to_empty_lower(edge_to_peak);
            }
        }
    }

    pub(super) fn propagate_beading_downward_to_empty_lower(&mut self, edge: EdgeId) {
        let half_edge = self.graph.edge(edge);
        let from = half_edge.from.unwrap();
        let to = half_edge.to.unwrap();
        assert!(!self.graph.node(from).data.has_beading());
        let top_storage = self.graph.node(to).data.beading().unwrap();
        let mut propagated = top_storage.borrow().clone();
        assert!(!propagated.is_upward_propagated_only);
        assert!(
            propagated.beading.total_thickness >= self.graph.node(to).data.distance_to_boundary * 2
        );
        propagated.dist_from_top_source +=
            point_distance(self.graph.node(from).point, self.graph.node(to).point);
        assert!(
            propagated.beading.total_thickness
                >= self.graph.node(from).data.distance_to_boundary * 2
        );
        let storage = Rc::new(RefCell::new(propagated));
        self.graph.node_mut(from).data.set_beading(&storage);
        self.beading_storage.push(storage);
    }

    pub(super) fn merge_beading_downward(&mut self, edge: EdgeId) {
        let half_edge = self.graph.edge(edge);
        let from = half_edge.from.unwrap();
        let to = half_edge.to.unwrap();
        let length = point_distance(self.graph.node(from).point, self.graph.node(to).point);
        let top = self.graph.node(to).data.beading().unwrap().borrow().clone();
        assert!(!top.is_upward_propagated_only);
        let bottom_storage = self.graph.node(from).data.beading().unwrap();
        let mut bottom = bottom_storage.borrow_mut();
        let total_distance = top.dist_from_top_source + length + bottom.dist_to_bottom_source;
        let transition_distance =
            total_distance.min(self.config.beading_propagation_transition_dist);
        let top_ratio =
            f64::from(bottom.dist_to_bottom_source as f32 / transition_distance as f32).max(0.0);
        if top_ratio >= 1.0 {
            let distance_from_top = top.dist_from_top_source + length;
            *bottom = top;
            bottom.dist_from_top_source = distance_from_top;
            return;
        }
        let merged = interpolate_beading_at_radius(
            &top.beading,
            top_ratio,
            &bottom.beading,
            self.graph.node(from).data.distance_to_boundary,
        );
        assert!(merged.total_thickness >= self.graph.node(from).data.distance_to_boundary * 2);
        *bottom = BeadingPropagation::new(merged);
    }

    fn compare_upward_quad_mids(&self, left: EdgeId, right: EdgeId) -> Ordering {
        let left_edge = self.graph.edge(left);
        let right_edge = self.graph.edge(right);
        let left_from = left_edge.from.unwrap();
        let left_to = left_edge.to.unwrap();
        let right_from = right_edge.from.unwrap();
        let right_to = right_edge.to.unwrap();
        let left_to_radius = self.graph.node(left_to).data.distance_to_boundary;
        let right_to_radius = self.graph.node(right_to).data.distance_to_boundary;
        if left_to_radius == right_to_radius {
            let left_flat = self.graph.node(left_from).data.distance_to_boundary == left_to_radius;
            let right_flat =
                self.graph.node(right_from).data.distance_to_boundary == right_to_radius;
            match (left_flat, right_flat) {
                (true, true) => {
                    return self
                        .distance_before_rise(left)
                        .cmp(&self.distance_before_rise(right));
                }
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                (false, false) => {}
            }
        }
        right_to_radius.cmp(&left_to_radius)
    }

    fn distance_before_rise(&self, edge: EdgeId) -> i64 {
        let half_edge = self.graph.edge(edge);
        let twin = half_edge.twin.unwrap();
        let rise = self
            .graph
            .edge_dist_to_go_up(edge)
            .into_iter()
            .chain(self.graph.edge_dist_to_go_up(twin))
            .min()
            .unwrap_or(i64::MAX);
        let from = half_edge.from.unwrap();
        let to = half_edge.to.unwrap();
        rise.saturating_sub(point_distance(
            self.graph.node(from).point,
            self.graph.node(to).point,
        ))
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

fn interpolate_beading_at_radius(
    left: &Beading,
    left_ratio: f64,
    right: &Beading,
    switching_radius: i64,
) -> Beading {
    let result = interpolate_beading(left, left_ratio, right);
    let Some(next_inset) = left
        .toolpath_locations
        .iter()
        .rposition(|&location| switching_radius > location)
    else {
        return result;
    };
    if next_inset + 1 == left.toolpath_locations.len()
        || result.toolpath_locations[next_inset] <= switching_radius
    {
        return result;
    }
    let adjusted_ratio = ((switching_radius - right.toolpath_locations[next_inset]) as f32
        / (left.toolpath_locations[next_inset] - right.toolpath_locations[next_inset]) as f32
        + 0.1)
        .min(1.0);
    interpolate_beading(left, f64::from(adjusted_ratio), right)
}

#[cfg(test)]
mod tests;
