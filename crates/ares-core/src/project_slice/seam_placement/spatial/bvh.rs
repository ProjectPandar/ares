use super::super::mesh::{TriangleMesh, Vec3};

#[derive(Clone, Copy)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    fn from_triangle(mesh: &TriangleMesh, triangle: usize) -> Self {
        let (min, max) = mesh.triangles[triangle].bounds();
        Self { min, max }
    }

    fn union(self, rhs: Self) -> Self {
        Self {
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
        }
    }

    fn intersects_ray(self, origin: Vec3, inverse_direction: [f64; 3], limit: f64) -> bool {
        let mut min = [
            f64::from(self.min.x),
            f64::from(self.min.y),
            f64::from(self.min.z),
        ];
        let mut max = [
            f64::from(self.max.x),
            f64::from(self.max.y),
            f64::from(self.max.z),
        ];
        let origin = [
            f64::from(origin.x),
            f64::from(origin.y),
            f64::from(origin.z),
        ];
        for axis in 0..3 {
            if inverse_direction[axis] < 0.0 {
                std::mem::swap(&mut min[axis], &mut max[axis]);
            }
        }
        let mut near = (min[0] - origin[0]) * inverse_direction[0];
        let mut far = (max[0] - origin[0]) * inverse_direction[0];
        let y_near = (min[1] - origin[1]) * inverse_direction[1];
        let y_far = (max[1] - origin[1]) * inverse_direction[1];
        if near > y_far || y_near > far {
            return false;
        }
        if y_near > near {
            near = y_near;
        }
        if y_far < far {
            far = y_far;
        }
        let z_near = (min[2] - origin[2]) * inverse_direction[2];
        if z_near > far {
            return false;
        }
        let z_far = (max[2] - origin[2]) * inverse_direction[2];
        if near > z_far {
            return false;
        }
        if z_near > near {
            near = z_near;
        }
        if z_far < far {
            far = z_far;
        }
        near < limit && far > 0.0
    }
}

#[derive(Clone, Copy)]
struct Input {
    triangle: usize,
    bounds: Bounds,
    centroid: Vec3,
}

#[derive(Clone, Copy, Default)]
enum NodeKind {
    #[default]
    Empty,
    Branch,
    Leaf(usize),
}

#[derive(Clone, Copy, Default)]
struct Node {
    kind: NodeKind,
    bounds: Option<Bounds>,
}

pub(in crate::project_slice::seam_placement) struct TriangleBvh {
    nodes: Vec<Node>,
}

impl TriangleBvh {
    pub(in crate::project_slice::seam_placement) fn new(mesh: &TriangleMesh) -> Self {
        let mut input = mesh
            .triangles
            .iter()
            .enumerate()
            .map(|(triangle, value)| Input {
                triangle,
                bounds: Bounds::from_triangle(mesh, triangle),
                centroid: value.centroid(),
            })
            .collect::<Vec<_>>();
        let mut nodes = vec![Node::default(); input.len().next_power_of_two() * 2 - 1];
        if !input.is_empty() {
            let right = input.len() - 1;
            build_recursive(&mut nodes, &mut input, 0, 0, right);
        }
        Self { nodes }
    }

    pub(in crate::project_slice::seam_placement) fn first_hit(
        &self,
        mesh: &TriangleMesh,
        origin: Vec3,
        direction: Vec3,
    ) -> Option<usize> {
        if self.nodes.is_empty() {
            return None;
        }
        let ray = Ray {
            origin,
            direction,
            inverse_direction: [
                f64::from(direction.x).recip(),
                f64::from(direction.y).recip(),
                f64::from(direction.z).recip(),
            ],
        };
        first_hit_recursive(&self.nodes, mesh, 0, ray, f64::INFINITY).map(|hit| hit.triangle)
    }
}

fn build_recursive(
    nodes: &mut [Node],
    input: &mut [Input],
    node: usize,
    left: usize,
    right: usize,
) {
    if left == right {
        nodes[node] = Node {
            kind: NodeKind::Leaf(input[left].triangle),
            bounds: Some(input[left].bounds),
        };
        return;
    }
    let bounds = input[left + 1..=right]
        .iter()
        .fold(input[left].bounds, |bounds, value| {
            bounds.union(value.bounds)
        });
    let extent = bounds.max - bounds.min;
    let dimension = if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    };
    let center = (left + right) / 2;
    partition(input, dimension, left, right, center);
    nodes[node] = Node {
        kind: NodeKind::Branch,
        bounds: Some(bounds),
    };
    build_recursive(nodes, input, node * 2 + 1, left, center);
    build_recursive(nodes, input, node * 2 + 2, center + 1, right);
}

fn partition(
    input: &mut [Input],
    dimension: usize,
    mut left: usize,
    mut right: usize,
    target: usize,
) {
    while left < right {
        let center = (left + right) / 2;
        let mut left_value = input[left].centroid.axis(dimension);
        let mut center_value = input[center].centroid.axis(dimension);
        let mut right_value = input[right].centroid.axis(dimension);
        if left_value > center_value {
            input.swap(left, center);
            std::mem::swap(&mut left_value, &mut center_value);
        }
        if left_value > right_value {
            input.swap(left, right);
            right_value = left_value;
        }
        if center_value > right_value {
            input.swap(center, right);
            center_value = right_value;
        }
        let pivot = center_value;
        if right <= left + 2 {
            break;
        }
        let mut first = left;
        let mut last = right - 1;
        input.swap(center, last);
        loop {
            first += 1;
            while input[first].centroid.axis(dimension) < pivot {
                first += 1;
            }
            last -= 1;
            while input[last].centroid.axis(dimension) > pivot && first < last {
                last -= 1;
            }
            if first >= last {
                break;
            }
            input.swap(first, last);
        }
        input.swap(first, right - 1);
        match target.cmp(&first) {
            std::cmp::Ordering::Less => right = first - 1,
            std::cmp::Ordering::Equal => break,
            std::cmp::Ordering::Greater => left = first + 1,
        }
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
    inverse_direction: [f64; 3],
}

#[derive(Clone, Copy)]
struct Hit {
    triangle: usize,
    distance: f32,
}

fn first_hit_recursive(
    nodes: &[Node],
    mesh: &TriangleMesh,
    node_index: usize,
    ray: Ray,
    limit: f64,
) -> Option<Hit> {
    let node = nodes[node_index];
    if !node
        .bounds
        .expect("a visited BVH node is initialized")
        .intersects_ray(ray.origin, ray.inverse_direction, limit)
    {
        return None;
    }
    match node.kind {
        NodeKind::Empty => unreachable!("a visited BVH node is initialized"),
        NodeKind::Leaf(triangle) => triangle_hit(mesh, triangle, ray.origin, ray.direction),
        NodeKind::Branch => {
            let left = first_hit_recursive(nodes, mesh, node_index * 2 + 1, ray, limit);
            let next_limit = left.map_or(limit, |hit| f64::from(hit.distance));
            let right = first_hit_recursive(nodes, mesh, node_index * 2 + 2, ray, next_limit);
            right.or(left)
        }
    }
}

fn triangle_hit(
    mesh: &TriangleMesh,
    triangle: usize,
    origin: Vec3,
    direction: Vec3,
) -> Option<Hit> {
    let value = mesh.triangles[triangle];
    let a = to_f64(value.vertices[0]);
    let edge_one = sub(to_f64(value.vertices[1]), a);
    let edge_two = sub(to_f64(value.vertices[2]), a);
    let direction = to_f64(direction);
    let p = cross(direction, edge_two);
    let determinant = dot(edge_one, p);
    let offset = sub(to_f64(origin), a);
    let u = dot(offset, p);
    let q = cross(offset, edge_one);
    let v = dot(direction, q);
    if determinant > 0.000_001 {
        if u < 0.0 || u > determinant || v < 0.0 || u + v > determinant {
            return None;
        }
    } else if determinant < -0.000_001 {
        if u > 0.0 || u < determinant || v > 0.0 || u + v < determinant {
            return None;
        }
    } else {
        return None;
    }
    let distance = dot(edge_two, q) / determinant;
    (distance > 0.0).then_some(Hit {
        triangle,
        distance: distance as f32,
    })
}

fn to_f64(value: Vec3) -> [f64; 3] {
    [f64::from(value.x), f64::from(value.y), f64::from(value.z)]
}

fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}
