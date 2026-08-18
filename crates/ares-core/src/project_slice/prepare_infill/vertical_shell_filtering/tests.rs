mod constants;
mod predicate;
mod protection;
mod thresholds;
mod topology;
mod transaction;

use crate::geometry::{ExPolygon, Point, Polygon};
use crate::project_slice::prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord;

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn expolygon(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(rectangle(min_x, min_y, max_x, max_y), Vec::new())
}

fn empty_record() -> PreparedSurfaceTypeRecord {
    PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        perimeter_source_indices: Vec::new(),
        thin_fill_source_indices: Vec::new(),
        slices: Vec::new(),
        fill_surfaces: Vec::new(),
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    }
}
