use crate::project::{
    Point3d, ProjectMesh,
    model_xml::{Mesh, ModelUnit},
    transform::Transform3d,
};

pub(super) struct PreparedMesh {
    pub(super) mesh: ProjectMesh,
    pub(super) transform: Transform3d,
}

pub(super) fn prepare(
    source: &Mesh,
    unit: ModelUnit,
    component_transform: Transform3d,
) -> PreparedMesh {
    let factor = unit.millimeter_factor();
    let mut vertices = source
        .vertices
        .vertices
        .iter()
        .map(|vertex| [vertex.x * factor, vertex.y * factor, vertex.z * factor])
        .collect::<Vec<_>>();
    let mut triangles = source
        .triangles
        .triangles
        .iter()
        .map(|triangle| [triangle.v1, triangle.v2, triangle.v3])
        .collect::<Vec<_>>();

    if signed_volume(&vertices, &triangles) < 0.0 {
        for triangle in &mut triangles {
            triangle.swap(1, 2);
        }
    }

    let shift = center_shift(&vertices);
    if shift.x != 0.0 || shift.y != 0.0 || shift.z != 0.0 {
        for vertex in &mut vertices {
            vertex[0] += -(shift.x as f32);
            vertex[1] += -(shift.y as f32);
            vertex[2] += -(shift.z as f32);
        }
    }

    PreparedMesh {
        mesh: ProjectMesh::new(
            vertices
                .into_iter()
                .map(|vertex| {
                    Point3d::new(
                        f64::from(vertex[0]),
                        f64::from(vertex[1]),
                        f64::from(vertex[2]),
                    )
                })
                .collect(),
            triangles,
        ),
        transform: component_transform.translated(shift),
    }
}

fn signed_volume(vertices: &[[f32; 3]], triangles: &[[u32; 3]]) -> f32 {
    let origin = vertices[0];
    triangles.iter().fold(0.0, |volume, triangle| {
        let [a, b, c] = triangle.map(|index| vertices[index as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let norm = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        let normal = if norm == 0.0 {
            cross
        } else {
            cross.map(|component| component / norm)
        };
        let height = normal[0] * (a[0] - origin[0])
            + normal[1] * (a[1] - origin[1])
            + normal[2] * (a[2] - origin[2]);
        volume + 0.5 * norm * height / 3.0
    })
}

fn center_shift(vertices: &[[f32; 3]]) -> Point3d {
    let bounds = vertices.iter().fold(
        [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]],
        |mut bounds, vertex| {
            for axis in 0..3 {
                bounds[0][axis] = bounds[0][axis].min(vertex[axis]);
                bounds[1][axis] = bounds[1][axis].max(vertex[axis]);
            }
            bounds
        },
    );
    Point3d::new(
        (f64::from(bounds[0][0]) + f64::from(bounds[1][0])) * 0.5,
        (f64::from(bounds[0][1]) + f64::from(bounds[1][1])) * 0.5,
        (f64::from(bounds[0][2]) + f64::from(bounds[1][2])) * 0.5,
    )
}
