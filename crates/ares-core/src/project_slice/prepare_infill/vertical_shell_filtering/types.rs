use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        prepare_infill::{
            surface_type_detection::PreparedSurfaceTypeObject,
            vertical_shell_projection::types::VerticalShellProjectionObject,
            vertical_shell_regularization::types::VerticalShellRegularizationObject,
            vertical_shell_trimming::types::VerticalShellTrimObject,
            vertical_shells::types::VerticalShellCacheObject,
        },
    },
};

#[derive(Debug)]
pub(in crate::project_slice) struct VerticalShellTinyFilter {
    pub(in crate::project_slice) filtered_shell: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct VerticalShellTinyFilterObject {
    pub(in crate::project_slice) records: Vec<Option<VerticalShellTinyFilter>>,
}

pub(in crate::project_slice) struct PreparedPostVerticalShellFiltering {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
    pub(in crate::project_slice) caches: Vec<VerticalShellCacheObject>,
    pub(in crate::project_slice) projections: Vec<VerticalShellProjectionObject>,
    pub(in crate::project_slice) trims: Vec<VerticalShellTrimObject>,
    pub(in crate::project_slice) regularizations: Vec<VerticalShellRegularizationObject>,
    pub(in crate::project_slice) filters: Vec<VerticalShellTinyFilterObject>,
}
