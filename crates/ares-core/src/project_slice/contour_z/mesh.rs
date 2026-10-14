//! SLA/IndexedMesh.cpp:17-43,139-171 / PrintObject.cpp:768-782.
use super::{
    ray::{self, Query},
    tree::Tree,
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Hit {
    pub(super) face: Option<usize>,
    pub(super) distance: f64,
    pub(super) normal: [f64; 3],
}
pub(super) struct IndexedMesh {
    vertices: Vec<[f32; 3]>,
    faces: Vec<[usize; 3]>,
    pub(super) tree: Tree,
    pub(super) ground_level: f64,
}
impl IndexedMesh {
    pub(super) fn new(vertices: Vec<[f32; 3]>, faces: Vec<[usize; 3]>, ground_level: f64) -> Self {
        let tree = Tree::build(&vertices, &faces);
        Self {
            vertices,
            faces,
            tree,
            ground_level,
        }
    }
    pub(super) fn from_object(
        object: &crate::ProjectObject,
        group: &crate::project::effective_config::types::ResolvedPrintObjectConfig,
        scale: crate::geometry::CoordinateScale,
    ) -> Self {
        let center = crate::project_slice::centering::center(object, group, scale)
            .expect("resolved model has finite bounds");
        let transform = group
            .transform
            .without_translation()
            .pretranslated(crate::Point3d::new(
                -scale.unscale(center.x()),
                -scale.unscale(center.y()),
                0.,
            ));
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut model_min_z = f64::INFINITY;
        for volume in object
            .volumes()
            .iter()
            .filter(|v| v.volume_type() == crate::ProjectVolumeType::ModelPart)
        {
            let offset = vertices.len();
            let model_transform = group.transform.then(volume.transform());
            vertices.extend(volume.mesh().vertices().iter().map(|&p| {
                let raw = p;
                let raw = crate::Point3d::new(
                    f64::from(raw.x as f32),
                    f64::from(raw.y as f32),
                    f64::from(raw.z as f32),
                );
                model_min_z = model_min_z.min(model_transform.transform_point(raw).z);
                let [x, y, z] = volume.transform().transform_point_f32(p);
                transform.transform_point_f32(crate::Point3d::new(
                    f64::from(x),
                    f64::from(y),
                    f64::from(z),
                ))
            }));
            faces.extend(
                volume
                    .mesh()
                    .triangles()
                    .iter()
                    .map(|face| face.map(|i| i as usize + offset)),
            );
        }
        // IndexedMesh::ground_level = mesh minimum Z plus -model min_z.
        let min_z = vertices
            .iter()
            .map(|v| f64::from(v[2]))
            .fold(f64::INFINITY, f64::min);
        Self::new(vertices, faces, min_z - model_min_z)
    }
    pub(super) fn query(&self, origin: [f64; 3]) -> Hit {
        let query = Query {
            vertices: &self.vertices,
            faces: &self.faces,
            tree: &self.tree,
            origin,
        };
        let hit = (!self.tree.nodes.is_empty())
            .then(|| query.first_hit(0, f64::INFINITY))
            .flatten();
        let Some(mut hit) = hit else {
            return Hit {
                face: None,
                distance: f64::INFINITY,
                normal: [0.; 3],
            };
        };
        let [a, b, c] = self.faces[hit.face.unwrap()].map(|i| self.vertices[i]);
        let edge1 = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
        let edge2 = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
        let normal = [
            edge1[1] * edge2[2] - edge1[2] * edge2[1],
            edge1[2] * edge2[0] - edge1[0] * edge2[2],
            edge1[0] * edge2[1] - edge1[1] * edge2[0],
        ]
        .map(f64::from);
        let length = ray::dot(normal, normal).sqrt();
        hit.normal = normal.map(|v| v / length);
        hit
    }
}
