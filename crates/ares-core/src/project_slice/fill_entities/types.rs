use crate::{
    ExtrusionRole,
    geometry::Polyline,
    project_slice::perimeters::classic::{
        entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
        materialize::FittedMove,
    },
};

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct FillExtrusionPath {
    pub(in crate::project_slice) polyline: Polyline,
    pub(in crate::project_slice) fitting: Vec<FittedMove>,
    pub(in crate::project_slice) role: ExtrusionRole,
    pub(in crate::project_slice) mm3_per_mm: f64,
    pub(in crate::project_slice) width: f32,
    pub(in crate::project_slice) height: f32,
}

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) struct FillExtrusionCollection {
    pub(in crate::project_slice) paths: Vec<FillExtrusionPath>,
    pub(in crate::project_slice) no_sort: bool,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct LayerFillEntities {
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
    pub(in crate::project_slice) collections: Vec<FillExtrusionCollection>,
    pub(in crate::project_slice) perimeter_source_indices: Vec<usize>,
    pub(in crate::project_slice) thin_fill_source_indices: Vec<Option<usize>>,
    pub(in crate::project_slice) thin_fills: Vec<GapFillEntity>,
}
