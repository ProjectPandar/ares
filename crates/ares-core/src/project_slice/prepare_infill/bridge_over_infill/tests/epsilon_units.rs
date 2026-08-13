use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_partial_support_comparison_uses_unscaled_epsilon() {
    let lower_fill = [rectangle(-1, -1, 10, 2)];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(0, 0, 10, 1),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &[], 1)),
        Some(layer(Some(0), &[], &current_surfaces, 1)),
    ];

    let candidates = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Limited,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(!candidates.surfaces_by_layer.contains_key(&1));
}

fn layer<'a>(
    lower_layer_index: Option<usize>,
    fill_expolygons: &'a [ExPolygon],
    fill_surfaces: &'a [RegionSurface],
    solid_infill_spacing: i64,
) -> CandidateLayer<'a> {
    CandidateLayer {
        lower_layer_index,
        region_index: 0,
        fill_expolygons,
        fill_surfaces,
        sparse_infill_density_percent: 15.0,
        solid_infill_spacing,
    }
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
