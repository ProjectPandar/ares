use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_filter_policies_apply_exact_multiplier_and_bypass_rules() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::Internal,
        lower_fill[0].clone(),
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(1_500, 4_000, 2_500, 6_000),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &lower_surfaces, 15.0, 1_000)),
        Some(layer(Some(0), &[], &current_surfaces, 15.0, 1_000)),
    ];

    let retained = [
        ProcessInternalBridgeFilter::Disabled,
        ProcessInternalBridgeFilter::Limited,
        ProcessInternalBridgeFilter::NoFilter,
    ]
    .map(|filter| {
        gather_candidates(&layers, false, filter, CoordinateScale::Normal)
            .unwrap()
            .surfaces_by_layer
            .get(&1)
            .is_some_and(|candidates| !candidates[0].new_polygons.is_empty())
    });

    assert_eq!(retained, [false, true, true]);
}

#[test]
fn task22o43_lower_internal_surface_is_solid_only_at_exact_100_percent() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::Internal,
        lower_fill[0].clone(),
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(4_000, 4_000, 6_000, 6_000),
    )];

    let retained = [99.999, 100.0].map(|density| {
        let layers = [
            Some(layer(None, &lower_fill, &lower_surfaces, density, 500)),
            Some(layer(Some(0), &[], &current_surfaces, 15.0, 500)),
        ];
        gather_candidates(
            &layers,
            false,
            ProcessInternalBridgeFilter::Disabled,
            CoordinateScale::Normal,
        )
        .unwrap()
        .surfaces_by_layer
        .contains_key(&1)
    });

    assert_eq!(retained, [true, false]);
}

#[test]
fn task22o43_partial_candidate_area_must_strictly_exceed_nine_spacing_squared() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::Internal,
        lower_fill[0].clone(),
    )];

    let retained = [400, 401].map(|max_x| {
        let current_surfaces = [RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            rectangle(0, 1_000, max_x, 1_300),
        )];
        let layers = [
            Some(layer(None, &lower_fill, &lower_surfaces, 15.0, 100)),
            Some(layer(Some(0), &[], &current_surfaces, 15.0, 100)),
        ];
        gather_candidates(
            &layers,
            false,
            ProcessInternalBridgeFilter::Limited,
            CoordinateScale::Normal,
        )
        .unwrap()
        .surfaces_by_layer
        .contains_key(&1)
    });

    assert_eq!(retained, [false, true]);
}

#[test]
fn task22o43_nofilter_retains_an_empty_fully_supported_candidate() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        lower_fill[0].clone(),
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(4_000, 4_000, 6_000, 6_000),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &lower_surfaces, 15.0, 500)),
        Some(layer(Some(0), &[], &current_surfaces, 15.0, 500)),
    ];

    let candidate_polygon_counts = [
        ProcessInternalBridgeFilter::Disabled,
        ProcessInternalBridgeFilter::Limited,
        ProcessInternalBridgeFilter::NoFilter,
    ]
    .map(|filter| {
        gather_candidates(&layers, false, filter, CoordinateScale::Normal)
            .unwrap()
            .surfaces_by_layer
            .get(&1)
            .map(|candidates| candidates[0].new_polygons.len())
    });

    assert_eq!(candidate_polygon_counts, [None, None, Some(0)]);
}

fn layer<'a>(
    lower_layer_index: Option<usize>,
    fill_expolygons: &'a [ExPolygon],
    fill_surfaces: &'a [RegionSurface],
    sparse_infill_density_percent: f64,
    solid_infill_spacing: i64,
) -> CandidateLayer<'a> {
    CandidateLayer {
        lower_layer_index,
        region_index: 0,
        fill_expolygons,
        fill_surfaces,
        sparse_infill_density_percent,
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
