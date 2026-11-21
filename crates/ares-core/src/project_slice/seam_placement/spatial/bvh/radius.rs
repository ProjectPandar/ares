use super::{Node, NodeKind, TriangleBvh, TriangleMesh, Vec3};
use crate::project_slice::seam_placement::mesh::Triangle;

impl TriangleBvh {
    pub(in crate::project_slice::seam_placement) fn any_in_radius(
        &self,
        mesh: &TriangleMesh,
        point: Vec3,
        radius: f32,
    ) -> bool {
        in_radius(&self.nodes, mesh, 0, point, radius * radius)
    }
}

fn in_radius(nodes: &[Node], mesh: &TriangleMesh, node: usize, point: Vec3, limit: f32) -> bool {
    let Some(bounds) = nodes[node].bounds else {
        return false;
    };
    let offset = (bounds.min - point)
        .max(point - bounds.max)
        .max(Vec3::default());
    if offset.norm_squared() > limit {
        return false;
    }
    match nodes[node].kind {
        NodeKind::Empty => false,
        NodeKind::Leaf(index) => distance_squared(mesh.triangles[index], point) < limit,
        NodeKind::Branch => {
            in_radius(nodes, mesh, node * 2 + 1, point, limit)
                || in_radius(nodes, mesh, node * 2 + 2, point, limit)
        }
    }
}

fn distance_squared(triangle: Triangle, p: Vec3) -> f32 {
    let [a, b, c] = triangle.vertices;
    let n = (b - a).cross(c - a);
    let squared = n.norm_squared();
    let edges = [(a, b), (b, c), (c, a)];
    // The plane projection is interior iff it is on the interior side of every edge.
    if squared > 0.0
        && edges
            .iter()
            .all(|&(u, v)| (v - u).cross(p - u).dot(n) >= 0.0)
    {
        let height = (p - a).dot(n);
        return height * height / squared;
    }
    edges
        .into_iter()
        .map(|(u, v)| {
            let edge = v - u;
            let length = edge.norm_squared();
            let t = if length > 0.0 {
                ((p - u).dot(edge) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (p - u - edge * t).norm_squared()
        })
        .fold(f32::INFINITY, f32::min)
}
