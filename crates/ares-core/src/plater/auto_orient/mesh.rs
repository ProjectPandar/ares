use crate::{
    ProjectMesh, SliceError,
    geometry::{
        mesh_normals::{face_normal, facet_area},
        try_convex_hull_3d,
    },
};

pub(super) struct Facet {
    pub vertices: [[f32; 3]; 3],
    pub normal: [f32; 3],
    pub area: f32,
    pub appearance: bool,
}
pub(super) struct Mesh {
    pub facets: Vec<Facet>,
    pub hull: Vec<Facet>,
}
impl Mesh {
    pub fn new(mesh: &ProjectMesh) -> Result<Self, SliceError> {
        let points: Vec<_> = mesh
            .vertices()
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32])
            .collect();
        let hull = try_convex_hull_3d(&points)
            .map_err(|err| SliceError::InvalidInput(format!("automatic orientation: {err}")))?
            .ok_or_else(|| {
                SliceError::InvalidInput(
                    "automatic orientation requires a three-dimensional mesh".into(),
                )
            })?;
        let facets = mesh
            .triangles()
            .iter()
            .enumerate()
            .map(|(i, face)| {
                let vertices = face.map(|index| points[index as usize]);
                let [a, b, c] = vertices;
                let property = mesh.face_properties().get(i).map_or("", AsRef::as_ref);
                // FaceProperty::from_string: valid enum 3 is exterior appearance;
                // its stored area does not contribute to geometric scoring.
                let appearance = property
                    .split_ascii_whitespace()
                    .next()
                    .and_then(|s| s.parse::<i32>().ok())
                    == Some(3);
                Facet {
                    vertices,
                    normal: face_normal(a, b, c),
                    area: facet_area(a, b, c),
                    appearance,
                }
            })
            .collect();
        let hull = hull
            .facets
            .iter()
            .map(|face| {
                let vertices = face.map(|index| hull.vertices[index as usize]);
                let [a, b, c] = vertices;
                Facet {
                    vertices,
                    // Orient.cpp recomputes face normals on the triangulated hull.
                    normal: face_normal(a, b, c),
                    area: facet_area(a, b, c),
                    appearance: false,
                }
            })
            .collect();
        Ok(Self { facets, hull })
    }
}
