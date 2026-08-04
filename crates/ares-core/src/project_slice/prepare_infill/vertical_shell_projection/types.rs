use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{
    geometry::Polygon,
    project_slice::prepare_infill::{
        surface_type_detection::PreparedSurfaceTypeObject,
        vertical_shells::types::VerticalShellCacheObject,
    },
};

#[derive(Debug)]
pub(in crate::project_slice) struct VerticalShellProjection {
    pub(in crate::project_slice) shell: Vec<Polygon>,
    pub(in crate::project_slice) holes: Vec<Polygon>,
}

pub(in crate::project_slice) struct VerticalShellProjectionObject {
    pub(in crate::project_slice) records: Vec<Option<VerticalShellProjection>>,
}

pub(in crate::project_slice) struct PreparedPostVerticalShellProjection {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
    pub(in crate::project_slice) caches: Vec<VerticalShellCacheObject>,
    pub(in crate::project_slice) projections: Vec<VerticalShellProjectionObject>,
}
