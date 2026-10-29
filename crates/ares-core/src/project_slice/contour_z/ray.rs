//! AABBTreeIndirect.hpp:269-365,396-440,723-746: first hit with float t.
use super::{
    mesh::Hit,
    tree::{INNER, Tree},
};
pub(super) struct Query<'a> {
    pub(super) vertices: &'a [[f32; 3]],
    pub(super) faces: &'a [[usize; 3]],
    pub(super) tree: &'a Tree,
    pub(super) origin: [f64; 3],
}
impl Query<'_> {
    pub(super) fn first_hit(&self, node_index: usize, mut min_t: f64) -> Option<Hit> {
        let node = &self.tree.nodes[node_index];
        if !box_hit(self.origin, node.bounds, min_t) {
            return None;
        }
        if node.index != INNER {
            let [a, b, c] = self.faces[node.index].map(|i| self.vertices[i].map(f64::from));
            return triangle(self.origin, a, b, c)
                .filter(|&t| t > 0.)
                .map(|t| Hit {
                    face: Some(node.index),
                    distance: f64::from(t as f32),
                    normal: [0.; 3],
                });
        }
        let left = self
            .first_hit(node_index * 2 + 1, min_t)
            .filter(|hit| hit.distance < min_t);
        if let Some(hit) = &left {
            min_t = hit.distance;
        }
        let right = self
            .first_hit(node_index * 2 + 2, min_t)
            .filter(|hit| hit.distance < min_t);
        right.or(left)
    }
}
fn box_hit(origin: [f64; 3], bounds: [[f32; 3]; 2], limit: f64) -> bool {
    // The only ContourZ query direction is (0,0,1); preserve source inf/NaN slab comparisons.
    let bounds = bounds.map(|v| v.map(f64::from));
    let mut tmin = (bounds[0][0] - origin[0]) * f64::INFINITY;
    let tymax = (bounds[1][1] - origin[1]) * f64::INFINITY;
    if tmin > tymax {
        return false;
    }
    let mut tmax = (bounds[1][0] - origin[0]) * f64::INFINITY;
    let tymin = (bounds[0][1] - origin[1]) * f64::INFINITY;
    if tymin > tmax {
        return false;
    }
    if tymin > tmin {
        tmin = tymin;
    }
    if tymax < tmax {
        tmax = tymax;
    }
    let tzmin = bounds[0][2] - origin[2];
    if tzmin > tmax {
        return false;
    }
    let tzmax = bounds[1][2] - origin[2];
    if tmin > tzmax {
        return false;
    }
    if tzmin > tmin {
        tmin = tzmin;
    }
    if tzmax < tmax {
        tmax = tzmax;
    }
    tmin < limit && tmax > 0.
}
fn triangle(origin: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    let edge1 = subtract(b, a);
    let edge2 = subtract(c, a);
    let direction = [0., 0., 1.];
    let pvec = cross(direction, edge2);
    let det = dot(edge1, pvec);
    let tvec = subtract(origin, a);
    let u = dot(tvec, pvec);
    let qvec = cross(tvec, edge1);
    let v = dot(direction, qvec);
    if det > 1e-6 {
        if u < 0. || u > det || v < 0. || u + v > det {
            return None;
        }
    } else if det < -1e-6 {
        if u > 0. || u < det || v > 0. || u + v < det {
            return None;
        }
    } else {
        return None;
    }
    Some(dot(edge2, qvec) * (1. / det))
}
pub(super) fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(super) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(super) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
