use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

const SPACING: i64 = 100;

#[test]
fn task22o43_lower_solid_thinner_than_two_spacings_disappears_before_reexpansion() {
    let geometry = candidate_geometry(
        rectangle(2_000, 0, 2_199, 5_000),
        rectangle(1_900, 2_000, 2_300, 2_400),
        ProcessInternalBridgeFilter::Limited,
    );

    assert_eq!(
        geometry,
        Some(vec![vec![
            (1_900, 2_000),
            (1_900, 2_400),
            (2_300, 2_000),
            (2_300, 2_400),
        ]])
    );
}

#[test]
fn task22o43_lower_solid_reexpands_by_one_plus_filter_multiplier_spacings() {
    let geometry = candidate_geometry(
        rectangle(2_000, 0, 3_000, 5_000),
        rectangle(1_920, 2_000, 1_980, 2_400),
        ProcessInternalBridgeFilter::Limited,
    );

    assert_eq!(geometry, None);
}

#[test]
fn task22o43_disabled_multiplier_reaches_two_spacings_farther_than_limited() {
    let support = rectangle(2_000, 0, 3_000, 5_000);
    let current = rectangle(1_750, 2_000, 1_800, 2_400);
    let geometry = [
        ProcessInternalBridgeFilter::Disabled,
        ProcessInternalBridgeFilter::Limited,
        ProcessInternalBridgeFilter::NoFilter,
    ]
    .map(|filter| candidate_geometry(support.clone(), current.clone(), filter));

    let limited_candidate = Some(vec![vec![
        (1_750, 2_000),
        (1_750, 2_400),
        (1_800, 2_000),
        (1_800, 2_400),
    ]]);
    let nofilter_candidate = Some(vec![vec![
        (1_350, 1_600),
        (1_350, 2_800),
        (2_200, 1_600),
        (2_200, 2_800),
    ]]);
    assert_eq!(geometry, [None, limited_candidate, nofilter_candidate]);
}

fn candidate_geometry(
    lower_solid: ExPolygon,
    current_solid: ExPolygon,
    filter: ProcessInternalBridgeFilter,
) -> Option<Vec<Vec<(i64, i64)>>> {
    let lower_fill = [rectangle(0, 0, 5_000, 5_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        lower_solid,
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        current_solid,
    )];
    let layers = [
        Some(layer(None, &lower_fill, &lower_surfaces)),
        Some(layer(Some(0), &[], &current_surfaces)),
    ];

    gather_candidates(&layers, false, filter, CoordinateScale::Normal)
        .unwrap()
        .surfaces_by_layer
        .get(&1)
        .map(|candidates| {
            assert_eq!(candidates.len(), 1);
            candidates[0]
                .new_polygons
                .iter()
                .map(|polygon| {
                    let mut vertices = polygon
                        .points()
                        .iter()
                        .map(|point| (point.x(), point.y()))
                        .collect::<Vec<_>>();
                    vertices.sort_unstable();
                    vertices
                })
                .collect()
        })
}

fn layer<'a>(
    lower_layer_index: Option<usize>,
    fill_expolygons: &'a [ExPolygon],
    fill_surfaces: &'a [RegionSurface],
) -> CandidateLayer<'a> {
    CandidateLayer {
        lower_layer_index,
        region_index: 0,
        fill_expolygons,
        fill_surfaces,
        sparse_infill_density_percent: 15.0,
        solid_infill_spacing: SPACING,
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
