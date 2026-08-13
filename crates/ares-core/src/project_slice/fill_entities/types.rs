use crate::{ExtrusionRole, geometry::Polyline};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct FillExtrusionPath {
    pub(in crate::project_slice) polyline: Polyline,
    pub(in crate::project_slice) role: ExtrusionRole,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct FillExtrusionCollection {
    pub(in crate::project_slice) paths: Vec<FillExtrusionPath>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::project_slice) struct LayerFillEntities {
    pub(in crate::project_slice) collections: Vec<FillExtrusionCollection>,
}
