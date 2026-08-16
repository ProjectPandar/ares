use super::mesh::{TriangleMesh, Vec3};

const LEAF_TRIANGLES: usize = 8;

pub(super) struct TriangleBvh {
    root: BvhNode,
}

impl TriangleBvh {
    pub(super) fn new(mesh: &TriangleMesh) -> Self {
        let mut indices = (0..mesh.triangles.len()).collect::<Vec<_>>();
        let root = BvhNode::build(mesh, &mut indices);
        Self { root }
    }

    pub(super) fn first_hit(
        &self,
        mesh: &TriangleMesh,
        origin: Vec3,
        direction: Vec3,
    ) -> Option<usize> {
        let ray = Ray { origin, direction };
        let mut hit = Hit {
            distance: f64::INFINITY,
            triangle: None,
        };
        self.root.first_hit(mesh, ray, &mut hit);
        hit.triangle
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

struct Hit {
    distance: f64,
    triangle: Option<usize>,
}

impl Hit {
    fn consider(&mut self, triangle: usize, distance: f64) {
        if distance < self.distance {
            self.distance = distance;
            self.triangle = Some(triangle);
        }
    }
}

struct BvhNode {
    bounds: Bounds,
    kind: BvhKind,
}

enum BvhKind {
    Leaf(Vec<usize>),
    Branch(Box<[BvhNode; 2]>),
}

impl BvhNode {
    fn build(mesh: &TriangleMesh, indices: &mut [usize]) -> Self {
        let bounds = indices
            .iter()
            .map(|&index| {
                let (min, max) = mesh.triangles[index].bounds();
                Bounds { min, max }
            })
            .reduce(Bounds::union)
            .expect("a seam visibility mesh has triangles");
        if indices.len() <= LEAF_TRIANGLES {
            return Self {
                bounds,
                kind: BvhKind::Leaf(indices.to_vec()),
            };
        }
        let extent = bounds.max - bounds.min;
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        indices.sort_unstable_by(|&left, &right| {
            mesh.triangles[left]
                .centroid()
                .axis(axis)
                .total_cmp(&mesh.triangles[right].centroid().axis(axis))
        });
        let middle = indices.len() / 2;
        let (left, right) = indices.split_at_mut(middle);
        Self {
            bounds,
            kind: BvhKind::Branch(Box::new([
                Self::build(mesh, left),
                Self::build(mesh, right),
            ])),
        }
    }

    fn first_hit(&self, mesh: &TriangleMesh, ray: Ray, hit: &mut Hit) {
        if !self
            .bounds
            .intersects_ray(ray.origin, ray.direction, hit.distance)
        {
            return;
        }
        match &self.kind {
            BvhKind::Leaf(indices) => {
                if let Some((index, distance)) = closest_leaf_hit(mesh, indices, ray) {
                    hit.consider(index, distance);
                }
            }
            BvhKind::Branch(children) => {
                children[0].first_hit(mesh, ray, hit);
                children[1].first_hit(mesh, ray, hit);
            }
        }
    }
}

fn closest_leaf_hit(mesh: &TriangleMesh, indices: &[usize], ray: Ray) -> Option<(usize, f64)> {
    indices
        .iter()
        .filter_map(|&index| {
            triangle_hit(mesh, index, ray.origin, ray.direction).map(|distance| (index, distance))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

#[derive(Clone, Copy)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    fn union(self, rhs: Self) -> Self {
        Self {
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
        }
    }

    fn intersects_ray(self, origin: Vec3, direction: Vec3, limit: f64) -> bool {
        let mut near = 0.0_f64;
        let mut far = limit;
        for axis in 0..3 {
            let Some((axis_near, axis_far)) = axis_interval(
                f64::from(origin.axis(axis)),
                f64::from(direction.axis(axis)),
                f64::from(self.min.axis(axis)),
                f64::from(self.max.axis(axis)),
            ) else {
                return false;
            };
            near = near.max(axis_near);
            far = far.min(axis_far);
            if near > far {
                return false;
            }
        }
        far >= 0.0
    }
}

fn axis_interval(origin: f64, direction: f64, min: f64, max: f64) -> Option<(f64, f64)> {
    if direction == 0.0 {
        return (min..=max)
            .contains(&origin)
            .then_some((f64::NEG_INFINITY, f64::INFINITY));
    }
    let inverse = direction.recip();
    let first = (min - origin) * inverse;
    let second = (max - origin) * inverse;
    Some((first.min(second), first.max(second)))
}

fn triangle_hit(mesh: &TriangleMesh, index: usize, origin: Vec3, direction: Vec3) -> Option<f64> {
    let triangle = mesh.triangles[index];
    let a = to_f64(triangle.vertices[0]);
    let edge_one = sub(to_f64(triangle.vertices[1]), a);
    let edge_two = sub(to_f64(triangle.vertices[2]), a);
    let direction = to_f64(direction);
    let p = cross(direction, edge_two);
    let determinant = dot(edge_one, p);
    if determinant.abs() < f64::EPSILON {
        return None;
    }
    let inverse = determinant.recip();
    let offset = sub(to_f64(origin), a);
    let u = dot(offset, p) * inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(offset, edge_one);
    let v = dot(direction, q) * inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let distance = dot(edge_two, q) * inverse;
    (distance >= 0.0).then_some(distance)
}

fn to_f64(value: Vec3) -> [f64; 3] {
    [f64::from(value.x), f64::from(value.y), f64::from(value.z)]
}

fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0].mul_add(right[0], left[1].mul_add(right[1], left[2] * right[2]))
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

pub(super) struct PointKdTree {
    root: Option<Box<KdNode>>,
}

impl PointKdTree {
    pub(super) fn new(points: &[Vec3]) -> Self {
        let mut indices = (0..points.len()).collect::<Vec<_>>();
        let root = KdNode::build(points, &mut indices, 0);
        Self { root }
    }

    pub(super) fn in_radius(&self, points: &[Vec3], target: Vec3, radius: f32) -> Vec<usize> {
        let mut output = Vec::new();
        if let Some(root) = &self.root {
            root.in_radius(&mut RadiusQuery {
                points,
                target,
                radius,
                radius_squared: radius * radius,
                output: &mut output,
            });
        }
        output
    }
}

struct RadiusQuery<'a> {
    points: &'a [Vec3],
    target: Vec3,
    radius: f32,
    radius_squared: f32,
    output: &'a mut Vec<usize>,
}
struct KdNode {
    index: usize,
    axis: usize,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
}

impl KdNode {
    fn build(points: &[Vec3], indices: &mut [usize], depth: usize) -> Option<Box<Self>> {
        if indices.is_empty() {
            return None;
        }
        let axis = depth % 3;
        indices.sort_unstable_by(|&left, &right| {
            points[left].axis(axis).total_cmp(&points[right].axis(axis))
        });
        let middle = indices.len() / 2;
        let (left, rest) = indices.split_at_mut(middle);
        let (current, right) = rest.split_first_mut().expect("median exists");
        Some(Box::new(Self {
            index: *current,
            axis,
            left: Self::build(points, left, depth + 1),
            right: Self::build(points, right, depth + 1),
        }))
    }

    fn in_radius(&self, query: &mut RadiusQuery<'_>) {
        let delta = query.target - query.points[self.index];
        if delta.norm_squared() <= query.radius_squared {
            query.output.push(self.index);
        }
        let axis_delta = query.target.axis(self.axis) - query.points[self.index].axis(self.axis);
        if axis_delta <= query.radius
            && let Some(left) = &self.left
        {
            left.in_radius(query);
        }
        if axis_delta >= -query.radius
            && let Some(right) = &self.right
        {
            right.in_radius(query);
        }
    }
}
