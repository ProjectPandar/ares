use crate::{
    geometry::Polygon, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::super::surface_type_detection::PreparedSurfaceTypeObject;

#[derive(Debug)]
pub(in crate::project_slice) struct VerticalShellCache {
    pub(in crate::project_slice) top_surfaces: Vec<Polygon>,
    pub(in crate::project_slice) bottom_surfaces: Vec<Polygon>,
    pub(in crate::project_slice) holes: Vec<Polygon>,
}

pub(in crate::project_slice) struct VerticalShellCacheObject {
    pub(in crate::project_slice) records: Vec<Option<VerticalShellCache>>,
}

pub(in crate::project_slice) struct PreparedPostVerticalShellCache {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
    pub(in crate::project_slice) caches: Vec<VerticalShellCacheObject>,
}
