use crate::{
    geometry::Polygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        prepare_infill::{
            surface_type_detection::PreparedSurfaceTypeObject,
            vertical_shell_projection::types::VerticalShellProjectionObject,
            vertical_shells::types::VerticalShellCacheObject,
        },
    },
};

#[derive(Debug)]
pub(in crate::project_slice) struct VerticalShellTrim {
    pub(in crate::project_slice) shell: Vec<Polygon>,
}

pub(in crate::project_slice) struct VerticalShellTrimObject {
    pub(in crate::project_slice) records: Vec<Option<VerticalShellTrim>>,
}

pub(in crate::project_slice) struct PreparedPostVerticalShellTrim {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
    pub(in crate::project_slice) caches: Vec<VerticalShellCacheObject>,
    pub(in crate::project_slice) projections: Vec<VerticalShellProjectionObject>,
    pub(in crate::project_slice) trims: Vec<VerticalShellTrimObject>,
}
