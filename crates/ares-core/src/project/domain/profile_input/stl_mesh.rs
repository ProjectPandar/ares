//! STL vertex indexing for project volume construction
//! (`Format/STL.cpp:17-40`, `Model.cpp:485-500`).

use std::collections::HashMap;

pub(in crate::project::domain) fn index_stl_vertices(
    model: &crate::Model,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let mut vertices = Vec::new();
    let mut index = HashMap::new();
    let mut triangles = Vec::with_capacity(model.triangles().len());
    for triangle in model.triangles() {
        let ids = triangle.vertices().map(|vertex| {
            let coordinates = [vertex.x, vertex.y, vertex.z];
            *index.entry(vertex_key(coordinates)).or_insert_with(|| {
                let id = vertices.len() as u32;
                vertices.push(coordinates);
                id
            })
        });
        triangles.push(ids);
    }
    (vertices, triangles)
}

fn vertex_key(coordinates: [f32; 3]) -> [u32; 3] {
    coordinates.map(|value| if value == 0.0 { 0 } else { value.to_bits() })
}
