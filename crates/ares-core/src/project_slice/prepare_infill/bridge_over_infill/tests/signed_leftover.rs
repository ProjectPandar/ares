use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_signed_leftover_filter_appends_only_the_positive_outer_path() {
    let lower_fill = [rectangle(7_000, 7_000, 13_000, 13_000)];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(0, 0, 20_000, 20_000),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &[], 1_000)),
        Some(layer(Some(0), &[], &current_surfaces, 1_000)),
    ];

    let polygons = &gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Limited,
        CoordinateScale::Normal,
    )
    .unwrap()
    .surfaces_by_layer[&1][0]
        .new_polygons;

    assert_eq!(
        polygons,
        &[Polygon::new(vec![
            Point::new(20_000, 20_000),
            Point::new(0, 20_000),
            Point::new(0, 0),
            Point::new(20_000, 0),
        ])]
    );
    assert_eq!(polygons[0].area(), 400_000_000.0);
    assert!(polygons[0].area().is_sign_positive());
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
