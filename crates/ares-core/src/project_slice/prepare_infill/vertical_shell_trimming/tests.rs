mod paths;
mod record;
mod transaction;

use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn square(min: i64, max: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ])
}

fn surface(kind: RegionSurfaceKind, min: i64, max: i64) -> RegionSurface {
    RegionSurface::new(kind, ExPolygon::new(square(min, max), Vec::new()))
}

fn record(fill_surfaces: Vec<RegionSurface>) -> PreparedSurfaceTypeRecord {
    PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices: Vec::new(),
        fill_surfaces,
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    }
}
