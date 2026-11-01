//! TriangleMeshSlicer.cpp:169-240: geometric planes, not XY collinearity.
use super::MeshTopology;

#[derive(Clone, Copy)]
struct Plane {
    origin: [f64; 3],
    normal: [f64; 3],
    valid: bool,
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(crate) fn coplanar_edges(
    vertices: &[[f64; 3]],
    triangles: &[[u32; 3]],
    topology: &MeshTopology,
    tolerance: f64,
) -> Vec<bool> {
    let mut first = vec![None; topology.edge_count() as usize];
    let mut planes = vec![None; triangles.len()];
    let mut output = vec![false; first.len()];
    let mut plane = |face: usize| {
        *planes[face].get_or_insert_with(|| {
            let [a, b, c] = triangles[face].map(|i| vertices[i as usize]);
            let u = sub(b, a);
            let v = sub(c, a);
            let mut normal = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let length = dot(normal, normal).sqrt();
            if length > 0.0 {
                normal = normal.map(|v| v / length);
            }
            Plane {
                origin: a,
                normal,
                valid: length > 0.0,
            }
        })
    };
    for (face, edges) in topology.face_edge_ids().iter().enumerate() {
        for (local, &edge) in edges.iter().enumerate() {
            if let Some((other, other_local)) = first[edge as usize] {
                let a = plane(other);
                let b = plane(face);
                let opposite_a = vertices[triangles[other][(other_local + 2) % 3] as usize];
                let opposite_b = vertices[triangles[face][(local + 2) % 3] as usize];
                output[edge as usize] = a.valid
                    && b.valid
                    && dot(a.normal, b.normal) > 0.0
                    && dot(a.normal, sub(opposite_b, a.origin)).abs() <= tolerance
                    && dot(b.normal, sub(opposite_a, b.origin)).abs() <= tolerance;
            } else {
                first[edge as usize] = Some((face, local));
            }
        }
    }
    output
}
