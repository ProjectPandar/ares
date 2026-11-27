use std::f32::consts::PI;

use crate::{CancelToken, SliceError};

use super::{
    mesh::{TriangleMesh, Vec3},
    sampling::{TriangleSamples, sample_uniform},
    spatial::{Hit, PointKdTree, TriangleBvh},
};

const RAYS_PER_AXIS: usize = 5;
const DIRECTION_COUNT: usize = RAYS_PER_AXIS * RAYS_PER_AXIS;

pub(super) struct GlobalVisibility {
    samples: TriangleSamples,
    values: Vec<f32>,
    sample_tree: PointKdTree,
    radius: f32,
}

impl GlobalVisibility {
    /// `front_bias` mirrors spAlignedBack's visibility adjustment
    /// (`SeamPlacer.cpp:165-170`): samples facing the front (-Y) get up to
    /// +1.0 visibility penalty so the seam prefers the back.
    pub(super) fn from_mesh(
        mesh: TriangleMesh,
        sample_count: usize,
        front_bias: bool,
        cancel: &CancelToken,
    ) -> Result<Self, SliceError> {
        let samples = sample_uniform(&mesh, sample_count);
        let tree = TriangleBvh::new(&mesh);
        let directions = hemisphere_directions();
        let contains_negative = mesh.negative_start < mesh.triangles.len();
        let mut values = vec![0.0; samples.positions.len()];
        let chunk = VisibilityChunk {
            mesh: &mesh,
            tree: &tree,
            directions: &directions,
            samples: &samples,
            contains_negative,
            front_bias,
            cancel,
        };
        // SeamPlacer.cpp:184-225 raycasts global visibility samples;
        // compute_global_occlusion is assembled around :986-1050.
        crate::parallel::par_chunks_mut_try(&mut values, 64, cancel, |offset, values| {
            chunk.fill(offset, values)
        })?;
        let density = sample_count as f32 / samples.total_area;
        let search_area = 4.0 / (-0.9_f32.ln() * density);
        // SeamPlacer.cpp:685 promotes search_area through double PI before sqrt.
        let radius = (f64::from(search_area) / std::f64::consts::PI).sqrt() as f32;
        let sample_tree = PointKdTree::new(&samples.positions);
        Ok(Self {
            samples,
            values,
            sample_tree,
            radius,
        })
    }

    pub(super) fn at(&self, position: Vec3) -> f32 {
        let mut found = false;
        let mut total_weight = 0.0;
        let mut total_visibility = 0.0;
        self.sample_tree.in_radius_for_each(
            &self.samples.positions,
            position,
            self.radius,
            |index| {
                found = true;
                let sample = self.samples.positions[index];
                let normal = self.samples.normals[index];
                let plane_distance = (position - sample).dot(normal).abs();
                // `SeamPlacer.cpp:341-342` adds two separately rounded contributions.
                let delta = position - sample;
                // SeamPlacer.cpp:342 Eigen squaredNorm: Native asm 0x1017941–0x1017965
                // reduces y²+z² before x²; keep this order local to the distance weight.
                let eigen_yz_then_x_squared_norm =
                    (delta.y * delta.y + delta.z * delta.z) + delta.x * delta.x;
                let weight = (self.radius - plane_distance)
                    + (self.radius - eigen_yz_then_x_squared_norm.sqrt());
                total_visibility += weight * self.values[index];
                total_weight += weight;
            },
        );
        if !found {
            return 1.0;
        }
        total_visibility / total_weight
    }
}

struct VisibilityChunk<'a> {
    mesh: &'a TriangleMesh,
    tree: &'a TriangleBvh,
    directions: &'a [Vec3],
    samples: &'a TriangleSamples,
    contains_negative: bool,
    front_bias: bool,
    cancel: &'a CancelToken,
}

impl VisibilityChunk<'_> {
    fn fill(&self, offset: usize, values: &mut [f32]) -> Result<(), SliceError> {
        if self.contains_negative {
            let mut negative_caster = NegativeRaycaster {
                mesh: self.mesh,
                tree: self.tree,
                directions: self.directions,
                hits: Vec::new(),
            };
            for (local_index, value) in values.iter_mut().enumerate() {
                self.cancel.check()?;
                let sample_index = offset + local_index;
                let center = self.samples.positions[sample_index];
                let normal = self.samples.normals[sample_index];
                let front_bias = front_adjustment(normal, self.front_bias);
                *value = negative_caster.visibility(
                    center,
                    normal,
                    self.samples.triangle_indices[sample_index],
                    1.0 + front_bias,
                );
            }
        } else {
            for (local_index, value) in values.iter_mut().enumerate() {
                self.cancel.check()?;
                let sample_index = offset + local_index;
                let center = self.samples.positions[sample_index];
                let normal = self.samples.normals[sample_index];
                let front_bias = front_adjustment(normal, self.front_bias);
                let visibility =
                    sample_visibility(self.mesh, self.tree, center, normal, self.directions);
                *value = adjust_visibility(visibility, front_bias, self.front_bias);
            }
        }
        Ok(())
    }
}

fn front_adjustment(normal: Vec3, front_bias: bool) -> f32 {
    if front_bias {
        ((-normal.y + 1.2) * 0.5).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn adjust_visibility(visibility: f32, front_adjustment: f32, front_bias: bool) -> f32 {
    if front_bias {
        visibility + front_adjustment
    } else {
        visibility
    }
}

fn hemisphere_directions() -> [Vec3; DIRECTION_COUNT] {
    let step = 1.0 / RAYS_PER_AXIS as f32;
    std::array::from_fn(|index| {
        let x = index / RAYS_PER_AXIS;
        let y = index % RAYS_PER_AXIS;
        let sample_x = x as f32 * step + step * 0.5;
        let sample_y = y as f32 * step + step * 0.5;
        let angle = 2.0 * PI * sample_x;
        let radial = 2.0 * (sample_y - sample_y * sample_y).sqrt();
        Vec3::new(
            angle.cos() * radial,
            angle.sin() * radial,
            (1.0 - 2.0 * sample_y).abs(),
        )
    })
}

// `SeamPlacer.cpp:74-80` Frame::set_from_z and :160-161 decrease_step.
fn visibility_frame(normal: Vec3, direction_count: usize) -> ([Vec3; 3], f32) {
    let z = normal.normalized();
    let provisional_x = if z.x.abs() > 0.99 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let y = z.cross(provisional_x).normalized();
    let x = y.cross(z);
    ([x, y, z], 1.0 / direction_count as f32)
}

// `SeamPlacer.cpp:188-225`: negative-face samples cast inward; hit signs
// distinguish subtraction surfaces without constructing a Boolean mesh.
struct NegativeRaycaster<'a> {
    mesh: &'a TriangleMesh,
    tree: &'a TriangleBvh,
    directions: &'a [Vec3],
    hits: Vec<Hit>,
}

impl NegativeRaycaster<'_> {
    fn visibility(
        &mut self,
        center: Vec3,
        normal: Vec3,
        triangle_index: usize,
        mut visibility: f32,
    ) -> f32 {
        let mesh = self.mesh;
        let from_negative = triangle_index >= mesh.negative_start;
        let ([x, y, z], decrement) = visibility_frame(normal, self.directions.len());
        let origin = if from_negative {
            center - normal * 0.01
        } else {
            center + normal * 0.01
        };
        for &local in self.directions {
            let direction = x * local.x + y * local.y + z * local.z;
            let direction = if from_negative { -direction } else { direction };
            self.tree.all_hits(mesh, origin, direction, &mut self.hits);
            if self.hits.is_empty() {
                continue;
            }
            let mut counter = 0;
            for hit in self.hits.iter().rev() {
                let dot = mesh.triangles[hit.triangle].normal().dot(direction);
                let sign = i32::from(dot > 0.0) - i32::from(dot < 0.0);
                counter += sign * (1 - 2 * i32::from(hit.triangle >= mesh.negative_start));
            }
            if counter == 0 {
                visibility -= decrement;
            }
        }
        visibility
    }
}

fn sample_visibility(
    mesh: &TriangleMesh,
    tree: &TriangleBvh,
    center: Vec3,
    normal: Vec3,
    directions: &[Vec3],
) -> f32 {
    let ([x, y, z], decrement) = visibility_frame(normal, directions.len());
    let origin = center + normal * 0.01;
    let mut visibility = 1.0;
    for &local in directions {
        let direction = x * local.x + y * local.y + z * local.z;
        if let Some(index) = tree.first_hit(mesh, origin, direction)
            && mesh.triangles[index].normal().dot(direction) <= 0.0
        {
            visibility -= decrement;
        }
    }
    visibility
}
