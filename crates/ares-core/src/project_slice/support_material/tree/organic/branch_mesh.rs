//! Indexed branch extrusion — `TreeSupport3D.cpp::extrude_branch`.

use super::{TreeSettings, interface_placer::layer_z, support_element::SupportElement};
use crate::{geometry::CoordinateScale, project_slice::parameters::SlicingParameters};

mod caps;
mod circle;
mod triangulation;
mod vector;

use circle::discretize_circle;
use triangulation::triangulate_strip;
use vector::normalized_f64;

#[derive(Default)]
pub(super) struct IndexedTriangleSet {
    pub(super) vertices: Vec<[f32; 3]>,
    pub(super) indices: Vec<[u32; 3]>,
}

const EPS: f32 = 0.015;
type Strip = (usize, usize);

pub(super) struct BranchExtrusion<'a> {
    pub(super) config: &'a TreeSettings,
    pub(super) slicing: &'a SlicingParameters,
    pub(super) scale: CoordinateScale,
}

impl BranchExtrusion<'_> {
    pub(super) fn extrude_branch(
        &self,
        path: &[&SupportElement],
        has_root: bool,
        result: &mut IndexedTriangleSet,
    ) -> (f32, f32) {
        let mut previous_strip = None;
        let mut zmin = 0.0;
        let mut zmax = 0.0;
        for index in 1..path.len() {
            let previous = path[index - 1];
            let current = path[index];
            let p1 = self.position(previous);
            let p2 = self.position(current);
            let direction = normalized_f64(std::array::from_fn(|axis| p2[axis] - p1[axis]));
            if index == 1 {
                let (strip, bottom) = caps::bottom(
                    result,
                    p1,
                    direction,
                    self.radius(previous),
                    has_root && previous.state.layer_index == 0,
                );
                previous_strip = Some(strip);
                zmin = bottom;
            }
            let previous_strip_ref = previous_strip
                .as_mut()
                .expect("positioned branch contains at least two nodes");
            let current_radius = self.radius(current);
            if index + 1 == path.len() {
                zmax = caps::top(result, p2, direction, current_radius, *previous_strip_ref);
            } else {
                let p3 = self.position(path[index + 1]);
                let next_direction =
                    normalized_f64(std::array::from_fn(|axis| p3[axis] - p2[axis]));
                let normal = normalized_f64([
                    direction[0] + next_direction[0],
                    direction[1] + next_direction[1],
                    direction[2] + next_direction[2],
                ]);
                let strip = discretize_circle(
                    p2.map(|value| value as f32),
                    normal.map(|value| value as f32),
                    current_radius,
                    EPS,
                    &mut result.vertices,
                );
                triangulate_strip(result, *previous_strip_ref, strip);
                *previous_strip_ref = strip;
            }
        }
        (zmin, zmax)
    }

    fn position(&self, element: &SupportElement) -> [f64; 3] {
        let point = element
            .state
            .result_on_layer
            .expect("branch nodes have been positioned");
        [
            self.scale.unscale(point.x()),
            self.scale.unscale(point.y()),
            layer_z(self.slicing, self.config, element.state.layer_index),
        ]
    }

    fn radius(&self, element: &SupportElement) -> f32 {
        element.state.actual_radius(self.config) as f32 * self.scale.factor() as f32
    }
}
