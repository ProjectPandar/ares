//! Native locked spheres and propagated support-branch Z spans.

use super::super::{interface_placer::layer_z, layer_index};
use super::{Smoothing, Sphere};

impl Smoothing<'_> {
    pub(super) fn spheres(&self, cache_layers: usize) -> Vec<Sphere> {
        let mut spheres: Vec<Sphere> = Vec::with_capacity(self.links.elements.len());
        for (index, link) in self.links.elements.iter().enumerate() {
            let element = &self.graph[link.layer][link.index];
            let point = element
                .state
                .result_on_layer
                .expect("organic branch node has a position");
            let position = [
                point.x() as f32 * self.scale.factor() as f32,
                point.y() as f32 * self.scale.factor() as f32,
                layer_z(self.slicing, self.config, element.state.layer_index) as f32,
            ];
            let min_z = link.below.map_or(position[2], |below| {
                spheres[self.links.layers[link.layer - 1] + below].min_z
            });
            spheres.push(Sphere {
                link: index,
                locked: element.parents.is_empty()
                    || (link.below.is_none() && element.state.layer_index > 0),
                radius: element.state.actual_radius(self.config) as f32
                    * self.scale.factor() as f32,
                position,
                previous: position,
                min_z,
                max_z: f32::MAX,
                layer_begin: 0,
                layer_end: 0,
            });
        }
        for index in (0..spheres.len()).rev() {
            spheres[index].max_z = self.maximum_z(&spheres[index], &spheres);
        }
        for sphere in &mut spheres {
            let layer = self.element(sphere).state.layer_index;
            sphere.min_z = sphere.min_z.max(sphere.position[2] - sphere.radius);
            sphere.max_z = sphere.max_z.min(sphere.position[2] + sphere.radius);
            sphere.layer_begin = layer.min(layer_index::ceil(
                self.slicing,
                self.config,
                f64::from(sphere.min_z),
            )) as u32;
            sphere.layer_end = cache_layers.min(
                layer.max(layer_index::floor(
                    self.slicing,
                    self.config,
                    f64::from(sphere.max_z),
                )) + 1,
            ) as u32;
        }
        spheres
    }

    fn maximum_z(&self, sphere: &Sphere, spheres: &[Sphere]) -> f32 {
        let element = self.element(sphere);
        if element.parents.is_empty() {
            return sphere.position[2];
        }
        let offset = self.links.layers[element.state.layer_index + 1];
        let mut maximum = f32::MAX;
        for &parent in &element.parents {
            maximum = maximum.min(spheres[offset + parent as usize].max_z);
        }
        maximum
    }
}
