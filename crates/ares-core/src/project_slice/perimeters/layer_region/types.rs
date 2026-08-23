use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
            traversal::PreparedPostClassicTraversal,
        },
        region_slices::RegionSurface,
    },
};

pub(in crate::project_slice) struct PreparedPostLayerRegionPerimeters {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedLayerRegionPerimeterObject>,
}

pub(in crate::project_slice) struct PreparedLayerRegionPerimeterObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedLayerRegionPerimeterRecord>>,
}

pub(in crate::project_slice) struct PreparedLayerRegionPerimeterRecord {
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
    pub(in crate::project_slice) thin_fills: Vec<GapFillEntity>,
    pub(in crate::project_slice) fill_surfaces: Vec<RegionSurface>,
    pub(in crate::project_slice) fill_expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) fill_no_overlap_expolygons: Vec<ExPolygon>,
}
