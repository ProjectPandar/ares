use super::Point3d;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub(super) struct MeshGeometry {
    pub(super) vertices: Vec<Point3d>,
    pub(super) triangles: Vec<[u32; 3]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectMesh {
    pub(super) geometry: Arc<MeshGeometry>,
    pub(crate) supported_facets: crate::project::facets_annotation::FacetsAnnotation,
}

impl ProjectMesh {
    pub(crate) fn new(vertices: Vec<Point3d>, triangles: Vec<[u32; 3]>) -> Self {
        Self {
            geometry: Arc::new(MeshGeometry {
                vertices,
                triangles,
            }),
            supported_facets: crate::project::facets_annotation::FacetsAnnotation::default(),
        }
    }

    pub fn vertices(&self) -> &[Point3d] {
        &self.geometry.vertices
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.geometry.triangles
    }
}
