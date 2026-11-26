use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        prepare_infill::{
            surface_type_detection::PreparedSurfaceTypeObject,
            vertical_shell_trimming::types::VerticalShellTrimObject,
        },
    },
};

#[derive(Debug)]
pub(in crate::project_slice) struct VerticalShellRegularization {
    pub(in crate::project_slice) regularized_shell: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct VerticalShellRegularizationObject {
    pub(in crate::project_slice) records: Vec<Option<VerticalShellRegularization>>,
}

pub(in crate::project_slice) struct PreparedPostVerticalShellRegularization {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
    pub(in crate::project_slice) trims: Vec<VerticalShellTrimObject>,
    pub(in crate::project_slice) regularizations: Vec<VerticalShellRegularizationObject>,
}
