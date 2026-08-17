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

const NONE: usize = usize::MAX;
const KD_EPSILON: f32 = 1e-4;

pub(super) struct PointKdTree {
    nodes: Vec<usize>,
}

impl PointKdTree {
    pub(super) fn new(points: &[Vec3]) -> Self {
        let mut indices = (0..points.len()).collect::<Vec<_>>();
        let mut nodes = vec![NONE; (indices.len() + 1).next_power_of_two()];
        if !indices.is_empty() {
            PointBuild {
                nodes: &mut nodes,
                input: &mut indices,
                points,
            }
            .build(0, 0, (0, points.len() - 1));
        }
        Self { nodes }
    }

    pub(super) fn in_radius(&self, points: &[Vec3], target: Vec3, radius: f32) -> Vec<usize> {
        let mut search = PointRadiusSearch {
            points,
            target,
            radius_squared: radius * radius,
            output: Vec::new(),
        };
        search.visit(self, 0, 0);
        search.output
    }
}

struct PointRadiusSearch<'a> {
    points: &'a [Vec3],
    target: Vec3,
    radius_squared: f32,
    output: Vec<usize>,
}

impl PointRadiusSearch<'_> {
    fn visit(&mut self, tree: &PointKdTree, node: usize, dimension: usize) {
        if node >= tree.nodes.len() || tree.nodes[node] == NONE {
            return;
        }
        let index = tree.nodes[node];
        if (self.target - self.points[index]).norm_squared() < self.radius_squared {
            self.output.push(index);
        }
        let delta = self.target.axis(dimension) - self.points[index].axis(dimension);
        let both = delta * delta < self.radius_squared + KD_EPSILON;
        let next_dimension = (dimension + 1) % 3;
        if both || delta <= 0.0 {
            self.visit(tree, node * 2 + 1, next_dimension);
        }
        if both || delta > 0.0 {
            self.visit(tree, node * 2 + 2, next_dimension);
        }
    }
}

struct PointBuild<'a> {
    nodes: &'a mut [usize],
    input: &'a mut [usize],
    points: &'a [Vec3],
}

impl PointBuild<'_> {
    fn build(&mut self, node: usize, dimension: usize, range: (usize, usize)) {
        let (left, right) = range;
        if left > right {
            return;
        }
        if left == right {
            self.nodes[node] = self.input[left];
            return;
        }
        let center = (left + right) / 2;
        self.partition(dimension, range, center);
        self.nodes[node] = self.input[center];
        let next_dimension = (dimension + 1) % 3;
        if center > left {
            self.build(node * 2 + 1, next_dimension, (left, center - 1));
        }
        self.build(node * 2 + 2, next_dimension, (center + 1, right));
    }

    fn partition(&mut self, dimension: usize, range: (usize, usize), target: usize) {
        let (mut left, mut right) = range;
        while left < right {
            let center = (left + right) / 2;
            let mut left_value = self.points[self.input[left]].axis(dimension);
            let mut center_value = self.points[self.input[center]].axis(dimension);
            let mut right_value = self.points[self.input[right]].axis(dimension);
            if left_value > center_value {
                self.input.swap(left, center);
                std::mem::swap(&mut left_value, &mut center_value);
            }
            if left_value > right_value {
                self.input.swap(left, right);
                right_value = left_value;
            }
            if center_value > right_value {
                self.input.swap(center, right);
                center_value = right_value;
            }
            let pivot = center_value;
            if right <= left + 2 {
                break;
            }
            let pivot_index = self.partition_pass(center, (left, right), dimension, pivot);
            match target.cmp(&pivot_index) {
                std::cmp::Ordering::Less => right = pivot_index - 1,
                std::cmp::Ordering::Equal => break,
                std::cmp::Ordering::Greater => left = pivot_index + 1,
            }
        }
    }

    fn partition_pass(
        &mut self,
        center: usize,
        range: (usize, usize),
        dimension: usize,
        pivot: f32,
    ) -> usize {
        let (left, right) = range;
        let mut first = left;
        let mut last = right - 1;
        self.input.swap(center, last);
        loop {
            first += 1;
            while self.points[self.input[first]].axis(dimension) < pivot {
                first += 1;
            }
            last -= 1;
            while self.points[self.input[last]].axis(dimension) > pivot && first < last {
                last -= 1;
            }
            if first >= last {
                break;
            }
            self.input.swap(first, last);
        }
        self.input.swap(first, right - 1);
        first
    }
}
