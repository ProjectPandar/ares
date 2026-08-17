use std::f32::consts::PI;

use super::{
    mesh::{TriangleMesh, Vec3},
    sampling::{TriangleSamples, sample_uniform},
    spatial::{PointKdTree, TriangleBvh},
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
    pub(super) fn from_mesh(mesh: TriangleMesh, sample_count: usize) -> Self {
        let samples = sample_uniform(&mesh, sample_count);
        let tree = TriangleBvh::new(&mesh);
        let directions = hemisphere_directions();
        let values = samples
            .positions
            .iter()
            .zip(&samples.normals)
            .map(|(&center, &normal)| sample_visibility(&mesh, &tree, center, normal, &directions))
            .collect();
        let density = sample_count as f32 / samples.total_area;
        let search_area = 4.0 / (-0.9_f32.ln() * density);
        let radius = (search_area / PI).sqrt();
        let sample_tree = PointKdTree::new(&samples.positions);
        Self {
            samples,
            values,
            sample_tree,
            radius,
        }
    }

    pub(super) fn at(&self, position: Vec3) -> f32 {
        let nearby = self
            .sample_tree
            .in_radius(&self.samples.positions, position, self.radius);
        if nearby.is_empty() {
            return 1.0;
        }
        let mut total_weight = 0.0;
        let mut total_visibility = 0.0;
        for index in nearby {
            let sample = self.samples.positions[index];
            let normal = self.samples.normals[index];
            let plane_distance = (position - sample).dot(normal).abs();
            let weight = self.radius - plane_distance + self.radius - (position - sample).norm();
            total_visibility += weight * self.values[index];
            total_weight += weight;
        }
        total_visibility / total_weight
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

fn sample_visibility(
    mesh: &TriangleMesh,
    tree: &TriangleBvh,
    center: Vec3,
    normal: Vec3,
    directions: &[Vec3],
) -> f32 {
    let z = normal.normalized();
    let provisional_x = if z.x.abs() > 0.99 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let y = z.cross(provisional_x).normalized();
    let x = y.cross(z);
    let origin = center + normal * 0.01;
    let decrement = 1.0 / directions.len() as f32;
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
