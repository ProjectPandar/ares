use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use crate::{Project, ProjectVolumeType};

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
        self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z))
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
        self / self.norm()
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

    pub(super) fn area(self) -> f64 {
        let a = self.vertices[0];
        let b = self.vertices[1];
        let c = self.vertices[2];
        let ab = [
            f64::from(b.x - a.x),
            f64::from(b.y - a.y),
            f64::from(b.z - a.z),
        ];
        let ac = [
            f64::from(c.x - a.x),
            f64::from(c.y - a.y),
            f64::from(c.z - a.z),
        ];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        0.5 * cross
            .into_iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt()
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
}

impl TriangleMesh {
    #[cfg(test)]
    pub(super) fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    pub(super) fn from_project(project: &Project, center: (f64, f64)) -> Self {
        let object = project
            .objects()
            .first()
            .expect("a sliced project has a printable object");
        let instance_transform = object
            .instances()
            .first()
            .expect("a sliced project object has an instance")
            .transform();
        let mut triangles = Vec::new();
        for volume in object
            .volumes()
            .iter()
            .filter(|volume| volume.volume_type() == ProjectVolumeType::ModelPart)
        {
            let transform = instance_transform.then(volume.transform());
            let vertices = volume
                .mesh()
                .vertices()
                .iter()
                .map(|&vertex| {
                    let [x, y, z] = transform.transform_point_f32(vertex);
                    Vec3::new(x - center.0 as f32, y - center.1 as f32, z)
                })
                .collect::<Vec<_>>();
            triangles.extend(volume.mesh().triangles().iter().map(|indices| {
                Triangle::new(
                    vertices[indices[0] as usize],
                    vertices[indices[1] as usize],
                    vertices[indices[2] as usize],
                )
            }));
        }
        Self { triangles }
    }
}
