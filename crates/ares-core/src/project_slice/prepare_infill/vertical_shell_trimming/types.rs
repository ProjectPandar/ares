use crate::{
    geometry::Polygon,
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        prepare_infill::surface_type_detection::PreparedSurfaceTypeObject,
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
    pub(in crate::project_slice) trims: Vec<VerticalShellTrimObject>,
}
