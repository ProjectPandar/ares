use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use crate::{Point3d, ProjectObject, ProjectVolumeType, geometry::CoordinateScale};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Vec3 {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) z: f32,
}

impl Vec3 {
    pub(super) const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub(super) fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub(super) fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub(super) fn norm_squared(self) -> f32 {
        self.dot(self)
    }

    pub(super) fn norm(self) -> f32 {
        self.norm_squared().sqrt()
    }

    pub(super) fn normalized(self) -> Self {
        // Eigen normalized() returns the vector unchanged for zero vectors
        // (Eigen/src/Core/Dot.h:92-102): 0/0 stays zero instead of NaN.
        let z = self.dot(self);
        if z > 0.0 { self / z.sqrt() } else { self }
    }

    pub(super) fn axis(self, axis: usize) -> f32 {
        match axis {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => unreachable!(),
        }
    }

    pub(super) fn min(self, rhs: Self) -> Self {
        Self::new(self.x.min(rhs.x), self.y.min(rhs.y), self.z.min(rhs.z))
    }

    pub(super) fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y), self.z.max(rhs.z))
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Triangle {
    pub(super) vertices: [Vec3; 3],
}

impl Triangle {
    pub(super) const fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self {
            vertices: [a, b, c],
        }
    }

    pub(super) fn normal(self) -> Vec3 {
        (self.vertices[1] - self.vertices[0])
            .cross(self.vertices[2] - self.vertices[1])
            .normalized()
    }

    pub(super) fn area(self) -> f32 {
        let a = self.vertices[0];
        let b = self.vertices[1];
        let c = self.vertices[2];
        0.5 * (b - a).cross(c - a).norm()
    }

    pub(super) fn bounds(self) -> (Vec3, Vec3) {
        (
            self.vertices[0].min(self.vertices[1]).min(self.vertices[2]),
            self.vertices[0].max(self.vertices[1]).max(self.vertices[2]),
        )
    }

    pub(super) fn centroid(self) -> Vec3 {
        (self.vertices[0] + self.vertices[1] + self.vertices[2]) / 3.0
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct TriangleMesh {
    pub(super) triangles: Vec<Triangle>,
    pub(super) negative_start: usize,
}

impl TriangleMesh {
    #[cfg(test)]
    pub(super) fn new(triangles: Vec<Triangle>) -> Self {
        Self {
            negative_start: triangles.len(),
            triangles,
        }
    }

    /// `SeamPlacer.cpp:630` — the occlusion mesh is transformed by
    /// `PrintObject::trafo_centered()` = instance transform with zeroed XY
    /// translation (`PrintApply.cpp` zeroes `trafo` XY, moving it to `shift`)
    /// pretranslated by `-center_offset`. The group center follows
    /// `PrintObject.cpp:97-110`, including the source/group rotation difference.
    pub(super) fn from_object(
        object: &ProjectObject,
        group: &crate::project::effective_config::types::ResolvedPrintObjectConfig,
        scale: CoordinateScale,
    ) -> Self {
        let instance_transform = group.transform;
        let center = crate::project_slice::centering::center(object, group, scale)
            .expect("resolved project object bounds must be finite");
        let object_transform =
            instance_transform
                .without_xy_translation()
                .pretranslated(Point3d::new(
                    -scale.unscale(center.x()),
                    -scale.unscale(center.y()),
                    0.0,
                ));
        let mut mesh = super::short_edge_collapse::IndexedMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        let mut negative = super::short_edge_collapse::IndexedMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        for volume in object.volumes() {
            let mesh = match volume.volume_type() {
                ProjectVolumeType::ModelPart => &mut mesh,
                ProjectVolumeType::NegativeVolume => &mut negative,
                _ => continue,
            };
            // `SeamPlacer.cpp:638-640` — "Mirrored transforms flip winding,
            // keep normals outward": the volume matrix is applied with
            // fix_left_handed=true, which swaps indices 0 and 1
            // (`its_transform`, admesh stl.h:362-367). The merged set is
            // `its_merge` concatenation (SeamPlacer.cpp:640-647).
            let volume_left_handed = volume.transform().linear_determinant() < 0.0;
            let order: [u32; 3] = if volume_left_handed {
                [1, 0, 2]
            } else {
                [0, 1, 2]
            };
            let base = mesh.vertices.len() as u32;
            mesh.vertices
                .extend(volume.mesh().vertices().iter().map(|&vertex| {
                    let vertex = volume.transform().transform_point(vertex);
                    Vec3::new(vertex.x as f32, vertex.y as f32, vertex.z as f32)
                }));
            mesh.indices
                .extend(volume.mesh().triangles().iter().map(|&[a, b, c]| {
                    let corners = [a, b, c];
                    [
                        base + corners[order[0] as usize],
                        base + corners[order[1] as usize],
                        base + corners[order[2] as usize],
                    ]
                }));
        }
        // `SeamPlacer.cpp:650-659`: decimate separately before merging;
        // sampled triangle IDs retain the positive/negative split.
        super::short_edge_collapse::short_edge_collapse(
            &mut mesh,
            FAST_DECIMATION_TRIANGLE_COUNT_TARGET,
        );
        super::short_edge_collapse::short_edge_collapse(
            &mut negative,
            FAST_DECIMATION_TRIANGLE_COUNT_TARGET,
        );
        let negative_start = mesh.indices.len();
        let vertex_offset = mesh.vertices.len() as u32;
        mesh.vertices.extend(negative.vertices);
        mesh.indices.extend(
            negative
                .indices
                .into_iter()
                .map(|indices| indices.map(|index| index + vertex_offset)),
        );
        // `SeamPlacer.cpp:659-660` — "Mirroring flips normals, keep them
        // outward for visibility sampling": the whole merged set is
        // transformed by obj_transform with fix_left_handed=true
        // (`its_transform` swaps indices 0 and 1, admesh stl.h:362-367).
        let object_left_handed = object_transform.linear_determinant() < 0.0;
        for vertex in &mut mesh.vertices {
            let [x, y, z] = object_transform.transform_point_f32(Point3d::new(
                f64::from(vertex.x),
                f64::from(vertex.y),
                f64::from(vertex.z),
            ));
            *vertex = Vec3::new(x, y, z);
        }
        let mut triangles = mesh
            .indices
            .iter()
            .map(|&[a, b, c]| {
                Triangle::new(
                    mesh.vertices[a as usize],
                    mesh.vertices[b as usize],
                    mesh.vertices[c as usize],
                )
            })
            .collect::<Vec<_>>();
        if object_left_handed {
            for triangle in &mut triangles {
                triangle.vertices.swap(0, 1);
            }
        }
        Self {
            triangles,
            negative_start,
        }
    }
}

/// `SeamPlacer.hpp:115` `fast_decimation_triangle_count_target`.
const FAST_DECIMATION_TRIANGLE_COUNT_TARGET: usize = 16000;
