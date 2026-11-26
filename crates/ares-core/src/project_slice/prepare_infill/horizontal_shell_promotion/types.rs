use crate::project_slice::{
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    prepare_infill::surface_type_detection::PreparedSurfaceTypeObject,
};

pub(in crate::project_slice) struct PreparedPostHorizontalShellPromotion {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
}
