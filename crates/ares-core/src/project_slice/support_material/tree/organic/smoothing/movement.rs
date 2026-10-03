//! Source collision nudge, Laplacian smoothing and moved-count decision.

use super::{Smoothing, Sphere, constraints::norm};
use crate::geometry::FloatLineDistanceTree;

impl Smoothing<'_> {
    pub(super) fn move_sphere(
        &self,
        sphere: &mut Sphere,
        spheres: &[Sphere],
        cache: &[Option<FloatLineDistanceTree>],
    ) -> bool {
        let collision = self.collision(sphere, cache);
        let mut moved = false;
        if let Some((depth, point)) = collision.filter(|(depth, _)| *depth > 0.0) {
            moved = depth > 1e-4;
            let distance = (depth + 0.1).min(0.5);
            // The native subtraction is f32 before casting to Vec2d.
            let delta = [
                f64::from(sphere.position[0] - point[0]),
                f64::from(sphere.position[1] - point[1]),
            ];
            let squared = delta[0] * delta[0] + delta[1] * delta[1];
            let normal = if squared > 0.0 {
                let length = squared.sqrt();
                delta.map(|value| value / length)
            } else {
                delta
            };
            let nudge = normal.map(|value| value * distance);
            // Native applies nudge_dist twice; preserve the two products.
            let candidate = [
                f64::from(sphere.position[0]) + nudge[0] * distance,
                f64::from(sphere.position[1]) + nudge[1] * distance,
            ];
            let candidate = self.constrain(sphere, spheres, candidate);
            sphere.position[0] = candidate[0] as f32;
            sphere.position[1] = candidate[1] as f32;
        }
        self.laplacian(sphere, spheres);
        moved
    }

    fn collision(
        &self,
        sphere: &Sphere,
        cache: &[Option<FloatLineDistanceTree>],
    ) -> Option<(f64, [f32; 2])> {
        let mut collision: Option<(f64, [f32; 2])> = None;
        let layer = self.element(sphere).state.layer_index as u32;
        for index in sphere.layer_begin..sphere.layer_end {
            // uint32_t - LayerIndex(int) is unsigned in the native expression.
            let dz = f64::from(index.wrapping_sub(layer)) * self.slicing.layer_height;
            let radius_squared = f64::from(sphere.radius * sphere.radius) - dz * dz;
            if radius_squared <= 0.0 {
                continue;
            }
            let Some(tree) = &cache[index as usize] else {
                continue;
            };
            let Some(hit) = tree.nearest_within(
                [f64::from(sphere.position[0]), f64::from(sphere.position[1])],
                radius_squared,
            ) else {
                continue;
            };
            let depth = radius_squared.sqrt() - hit.squared_distance.sqrt();
            if depth > collision.as_ref().map_or(-f64::MAX, |&(depth, _)| depth) {
                collision = Some((depth, hit.point.map(|value| value as f32)));
            }
        }
        collision
    }

    fn laplacian(&self, sphere: &mut Sphere, spheres: &[Sphere]) {
        let element = self.element(sphere);
        let offset = self.links.layers[element.state.layer_index + 1];
        let mut average = [0.0; 2];
        let mut weight = 0.0;
        for &parent in &element.parents {
            let parent = &spheres[offset + parent as usize];
            let w = f64::from(sphere.radius);
            for (axis, value) in average.iter_mut().enumerate() {
                *value += w * f64::from(parent.previous[axis]);
            }
            weight += w;
        }
        if let Some(below) = self.links.elements[sphere.link].below {
            let below = &spheres[self.links.layers[element.state.layer_index - 1] + below];
            let w = weight;
            for (axis, value) in average.iter_mut().enumerate() {
                *value += w * f64::from(below.previous[axis]);
            }
            weight += w;
        }
        average = average.map(|value| value / weight);
        let old = [f64::from(sphere.position[0]), f64::from(sphere.position[1])];
        let new = [
            0.5 * old[0] + 0.5 * average[0],
            0.5 * old[1] + 0.5 * average[1],
        ];
        let shift = [new[0] - old[0], new[1] - old[1]];
        let length = norm(shift);
        if length > 0.0 {
            let distance = length.min(0.2);
            let candidate = [
                old[0] + shift[0] * (distance / length),
                old[1] + shift[1] * (distance / length),
            ];
            let candidate = self.constrain(sphere, spheres, candidate);
            sphere.position[0] = candidate[0] as f32;
            sphere.position[1] = candidate[1] as f32;
        }
    }
}
