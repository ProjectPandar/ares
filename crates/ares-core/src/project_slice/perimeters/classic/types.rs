use crate::{
    Project,
    geometry::{Coord, CoordinateScale, ExPolygon, Polygon},
    project::effective_config::types::BoundedResolvedProjectConfig,
    project_slice::region_slices::RegionSurfaceKind,
};

use super::super::types::{Flow, PostPerimeterInputPrintObject};

pub(in crate::project_slice) struct PreparedPostClassicPrelude {
    pub(in crate::project_slice) project: Project,
    pub(in crate::project_slice) resolved: Box<BoundedResolvedProjectConfig>,
    pub(in crate::project_slice) config_block: Option<Vec<u8>>,
    pub(in crate::project_slice) scale: CoordinateScale,
    pub(in crate::project_slice) objects: Vec<PostClassicPreludePrintObject>,
}

pub(in crate::project_slice) struct PostClassicPreludePrintObject {
    pub(in crate::project_slice) object: PostPerimeterInputPrintObject,
    pub(in crate::project_slice) records: Vec<Option<ClassicPreludeRecord>>,
}

#[derive(Debug)]
pub(in crate::project_slice) struct ClassicPreludeRecord {
    pub(in crate::project_slice) perimeter_width: Coord,
    pub(in crate::project_slice) perimeter_spacing: Coord,
    pub(in crate::project_slice) external_width: Coord,
    pub(in crate::project_slice) external_spacing: Coord,
    pub(in crate::project_slice) external_to_internal_spacing: Coord,
    pub(in crate::project_slice) solid_infill_spacing: Coord,
    pub(in crate::project_slice) minimum_spacing: Coord,
    pub(in crate::project_slice) external_minimum_spacing: Coord,
    pub(in crate::project_slice) smaller_external_minimum_spacing: Coord,
    pub(in crate::project_slice) has_gap_fill: bool,
    pub(in crate::project_slice) smaller_external_flow: Flow,
    pub(in crate::project_slice) lower_slices_polygons: Vec<Polygon>,
    pub(in crate::project_slice) lower_polygons_series: Vec<Vec<Polygon>>,
    pub(in crate::project_slice) external_lower_polygons_series: Vec<Vec<Polygon>>,
    pub(in crate::project_slice) smaller_external_lower_polygons_series: Vec<Vec<Polygon>>,
    pub(in crate::project_slice) surface_simplify_resolution: f64,
    pub(in crate::project_slice) surfaces: Vec<PreparedClassicSurface>,
}

#[derive(Debug)]
pub(in crate::project_slice) struct PreparedClassicSurface {
    pub(in crate::project_slice) source_index: usize,
    pub(in crate::project_slice) kind: RegionSurfaceKind,
    pub(in crate::project_slice) thickness: f64,
    pub(in crate::project_slice) thickness_layers: u16,
    pub(in crate::project_slice) bridge_angle: f64,
    pub(in crate::project_slice) extra_perimeters: u16,
    pub(in crate::project_slice) loop_number: i32,
    pub(in crate::project_slice) polygons: Vec<ExPolygon>,
}

impl PostClassicPreludePrintObject {
    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (
        PostPerimeterInputPrintObject,
        Vec<Option<ClassicPreludeRecord>>,
    ) {
        (self.object, self.records)
    }
}
