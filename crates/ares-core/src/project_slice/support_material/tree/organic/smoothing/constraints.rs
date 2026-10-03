//! Source linked-layer reachability constraint on each local candidate.

use super::{Smoothing, Sphere};

impl Smoothing<'_> {
    pub(super) fn constrain(
        &self,
        sphere: &Sphere,
        spheres: &[Sphere],
        mut candidate: [f64; 2],
    ) -> [f64; 2] {
        let element = self.element(sphere);
        let link = self.links.elements[sphere.link];
        let layer = element.state.layer_index;
        let current = [f64::from(sphere.position[0]), f64::from(sphere.position[1])];
        let radius = element.state.actual_radius(self.config) as f64;
        let slow = self.config.maximum_move_distance_slow as f64;
        if let Some(below) = link.below.filter(|_| layer > 0) {
            let lower_id = self.links.layers[layer - 1] + below;
            if lower_id < spheres.len() {
                let lower = &spheres[lower_id];
                let lower_radius = self.element(lower).state.actual_radius(self.config) as f64;
                let allowed = ((lower_radius - radius).max(0.0) + slow) * self.scale.factor();
                candidate = constrain_to_anchor(
                    candidate,
                    current,
                    [f64::from(lower.previous[0]), f64::from(lower.previous[1])],
                    allowed,
                );
            }
        }
        if element.parents.is_empty() || layer + 1 >= self.links.layers.len() {
            return candidate;
        }
        let offset = self.links.layers[layer + 1];
        for &parent in &element.parents {
            let upper_id = offset + parent as usize;
            if upper_id >= spheres.len() {
                continue;
            }
            let upper = &spheres[upper_id];
            let upper_radius = self.element(upper).state.actual_radius(self.config) as f64;
            let allowed = ((radius - upper_radius).max(0.0) + slow) * self.scale.factor();
            candidate = constrain_to_anchor(
                candidate,
                current,
                [f64::from(upper.previous[0]), f64::from(upper.previous[1])],
                allowed,
            );
        }
        candidate
    }
}

fn constrain_to_anchor(
    candidate: [f64; 2],
    current: [f64; 2],
    anchor: [f64; 2],
    allowed: f64,
) -> [f64; 2] {
    let delta = [candidate[0] - anchor[0], candidate[1] - anchor[1]];
    let distance = norm(delta);
    let allowed = allowed.max(norm([current[0] - anchor[0], current[1] - anchor[1]]));
    if distance > allowed && distance > 1e-4 {
        [
            anchor[0] + delta[0] * (allowed / distance),
            anchor[1] + delta[1] * (allowed / distance),
        ]
    } else {
        candidate
    }
}

pub(super) fn norm(vector: [f64; 2]) -> f64 {
    (vector[0] * vector[0] + vector[1] * vector[1]).sqrt()
}
