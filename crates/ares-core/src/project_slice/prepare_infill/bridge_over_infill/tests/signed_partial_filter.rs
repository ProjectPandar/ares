use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_partial_filter_uses_signed_area_for_a_donut() {
    let lower_fill = [ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(720, 0),
            Point::new(720, 720),
            Point::new(0, 720),
        ]),
        vec![Polygon::new(vec![
            Point::new(210, 210),
            Point::new(210, 510),
            Point::new(510, 510),
            Point::new(510, 210),
        ])],
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(720, 0),
                Point::new(720, 720),
                Point::new(0, 720),
            ]),
            Vec::new(),
        ),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &[], 100)),
        Some(layer(Some(0), &[], &current_surfaces, 100)),
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
