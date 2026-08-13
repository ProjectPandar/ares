use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_nofilter_retains_empty_candidate_over_absent_lower_record() {
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(1_000, 1_000, 2_000, 2_000),
    )];
    let layers = [
        None,
        Some(CandidateLayer {
            lower_layer_index: Some(0),
            region_index: 3,
            fill_expolygons: &[],
            fill_surfaces: &current_surfaces,
            sparse_infill_density_percent: 15.0,
            solid_infill_spacing: 100,
        }),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::NoFilter,
        CoordinateScale::Normal,
    )
    .unwrap();

    let candidates = &gathered.surfaces_by_layer[&1];
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source.region_index, 3);
    assert!(candidates[0].new_polygons.is_empty());
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min_x, min_y),
            Point::new(max_x, min_y),
            Point::new(max_x, max_y),
            Point::new(min_x, max_y),
        ]),
        Vec::new(),
    )
}
