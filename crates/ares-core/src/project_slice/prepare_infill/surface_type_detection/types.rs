use crate::geometry::ExPolygon;
use crate::project_slice::{
    perimeters::{
        classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
            traversal::PreparedPostClassicTraversal,
        },
        layer_region::PreparedLayerRegionPerimeterRecord,
    },
    region_slices::RegionSurface,
};

pub(in crate::project_slice) struct PreparedPostSurfaceTypeDetection {
    pub(in crate::project_slice) predecessor: Box<PreparedPostClassicTraversal>,
    pub(in crate::project_slice) objects: Vec<PreparedSurfaceTypeObject>,
}

pub(in crate::project_slice) struct PreparedSurfaceTypeObject {
    pub(in crate::project_slice) records: Vec<Option<PreparedSurfaceTypeRecord>>,
}

pub(in crate::project_slice) struct PreparedSurfaceTypeRecord {
    pub(in crate::project_slice) perimeters: Vec<ExtrusionEntityCollection>,
    pub(in crate::project_slice) thin_fills: Vec<GapFillEntity>,
    pub(in crate::project_slice) perimeter_source_indices: Vec<usize>,
    pub(in crate::project_slice) thin_fill_source_indices: Vec<usize>,
    pub(in crate::project_slice) slices: Vec<RegionSurface>,
    pub(in crate::project_slice) fill_surfaces: Vec<RegionSurface>,
    pub(in crate::project_slice) fill_expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) fill_no_overlap_expolygons: Vec<ExPolygon>,
}

pub(super) struct StagedRecord {
    pub(super) slices: Vec<RegionSurface>,
    pub(super) fill_surfaces: Vec<RegionSurface>,
}

pub(super) fn materialize_record(
    source: PreparedLayerRegionPerimeterRecord,
    staged: StagedRecord,
) -> PreparedSurfaceTypeRecord {
    let PreparedLayerRegionPerimeterRecord {
        perimeters,
        thin_fills,
        perimeter_source_indices,
        thin_fill_source_indices,
        fill_surfaces: _,
        fill_expolygons,
        fill_no_overlap_expolygons,
    } = source;
    PreparedSurfaceTypeRecord {
        perimeters,
        thin_fills,
        perimeter_source_indices,
        thin_fill_source_indices,
        slices: staged.slices,
        fill_surfaces: staged.fill_surfaces,
        fill_expolygons,
        fill_no_overlap_expolygons,
    }
}
