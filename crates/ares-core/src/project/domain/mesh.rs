use super::Point3d;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MeshGeometry {
    pub(super) vertices: Vec<Point3d>,
    pub(super) triangles: Vec<[u32; 3]>,
    pub(super) face_properties: Vec<Box<str>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectMesh {
    pub(super) geometry: Arc<MeshGeometry>,
    pub(crate) supported_facets: crate::project::facets_annotation::FacetsAnnotation,
    pub(crate) mmu_segmentation_facets: crate::project::facets_annotation::FacetsAnnotation,
    pub(crate) fuzzy_skin_facets: crate::project::facets_annotation::FacetsAnnotation,
}

impl ProjectMesh {
    /// Loader mesh-cache identity (`bbs_3mf.cpp` mesh reuse): volumes
    /// cloned from the same cached mesh compare equal for
    /// `shares_print_toolpaths` (`Print.cpp:2211` `mesh_ptr()`).
    pub(super) fn geometry_identity(&self) -> usize {
        std::ptr::from_ref(self.geometry.as_ref()) as usize
    }
    pub(crate) fn new(
        vertices: Vec<Point3d>,
        triangles: Vec<[u32; 3]>,
        face_properties: Vec<Box<str>>,
    ) -> Self {
        Self {
            geometry: Arc::new(MeshGeometry {
                vertices,
                triangles,
                face_properties,
            }),
            supported_facets: crate::project::facets_annotation::FacetsAnnotation::default(),
            mmu_segmentation_facets: crate::project::facets_annotation::FacetsAnnotation::default(),
            fuzzy_skin_facets: crate::project::facets_annotation::FacetsAnnotation::default(),
        }
    }

    pub fn vertices(&self) -> &[Point3d] {
        &self.geometry.vertices
    }
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.geometry.triangles
    }
    /// Opaque BBS triangle properties, in source-triangle order.
    pub fn face_properties(&self) -> &[Box<str>] {
        &self.geometry.face_properties
    }
}
