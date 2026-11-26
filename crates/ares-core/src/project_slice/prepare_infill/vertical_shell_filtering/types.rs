use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        prepare_infill::surface_type_detection::PreparedSurfaceTypeObject,
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
    pub(in crate::project_slice) filters: Vec<VerticalShellTinyFilterObject>,
}
