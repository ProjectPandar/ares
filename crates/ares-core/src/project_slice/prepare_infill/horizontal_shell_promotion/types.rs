use crate::project_slice::{
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    prepare_infill::{
        surface_type_detection::PreparedSurfaceTypeObject,
        vertical_shell_filtering::types::VerticalShellTinyFilterObject,
        vertical_shell_projection::types::VerticalShellProjectionObject,
        vertical_shell_regularization::types::VerticalShellRegularizationObject,
        vertical_shell_trimming::types::VerticalShellTrimObject,
        vertical_shells::types::VerticalShellCacheObject,
    },
};

pub(in crate::project_slice) struct PreparedPostHorizontalShellPromotion {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
    pub(in crate::project_slice) caches: Vec<VerticalShellCacheObject>,
    pub(in crate::project_slice) projections: Vec<VerticalShellProjectionObject>,
    pub(in crate::project_slice) trims: Vec<VerticalShellTrimObject>,
    pub(in crate::project_slice) regularizations: Vec<VerticalShellRegularizationObject>,
    pub(in crate::project_slice) filters: Vec<VerticalShellTinyFilterObject>,
}
