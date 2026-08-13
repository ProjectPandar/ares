use crate::{
    ProcessInternalBridgeFilter,
    geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

const OUTSIDE_CLIPPER_RANGE: i64 = 0x4000_0000_0000_0000;

#[test]
fn task22o43_first_layer_and_missing_records_do_not_create_candidates() {
    let first_surfaces = [surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(1_000, 1_000, 9_000, 9_000),
    )];
    let no_candidate_surfaces = [surface(
        RegionSurfaceKind::Top,
        rectangle(2_000, 2_000, 8_000, 8_000),
    )];
    let layers = [
        Some(layer(None, 4, &[], &first_surfaces)),
        None,
        Some(layer(Some(0), 9, &[], &no_candidate_surfaces)),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Disabled,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(!gathered.has_lightning_infill);
    assert!(gathered.surfaces_by_layer.is_empty());
}

#[test]
fn task22o43_preserves_layer_region_and_surface_identity_in_source_order() {
    let lower_fill = [rectangle(0, 0, 20_000, 20_000)];
    let lower_surfaces = [surface(RegionSurfaceKind::Internal, lower_fill[0].clone())];
    let layer_one_surfaces = [
        surface(
            RegionSurfaceKind::InternalSolid,
            rectangle(1_000, 1_000, 4_000, 4_000),
        ),
        surface(
            RegionSurfaceKind::Bottom,
            rectangle(5_000, 1_000, 8_000, 4_000),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            rectangle(9_000, 1_000, 12_000, 4_000),
        ),
    ];
    let layer_two_surfaces = [
        surface(
            RegionSurfaceKind::Internal,
            rectangle(1_000, 6_000, 4_000, 9_000),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            rectangle(5_000, 6_000, 8_000, 9_000),
        ),
    ];
    let layers = [
        Some(layer(None, 0, &lower_fill, &lower_surfaces)),
        Some(layer(Some(0), 7, &[], &layer_one_surfaces)),
        Some(layer(Some(0), 3, &[], &layer_two_surfaces)),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::NoFilter,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        gathered
            .surfaces_by_layer
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        gathered.surfaces_by_layer[&1]
            .iter()
            .map(|candidate| {
                (
                    candidate.source.layer_index,
                    candidate.source.region_index,
                    candidate.source.surface_index,
                )
            })
            .collect::<Vec<_>>(),
        vec![(1, 7, 0), (1, 7, 2)]
    );
    assert_eq!(
        gathered.surfaces_by_layer[&2]
            .iter()
            .map(|candidate| {
                (
                    candidate.source.layer_index,
                    candidate.source.region_index,
                    candidate.source.surface_index,
                )
            })
            .collect::<Vec<_>>(),
        vec![(2, 3, 1)]
    );
}

#[test]
fn task22o43_scaled_epsilon_closes_a_gap_only_at_normal_scale() {
    let lower_fill = [
        rectangle(0, 0, 1_000, 1_000),
        rectangle(1_050, 0, 2_050, 1_000),
    ];
    let current_surfaces = [surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(1_010, 100, 1_040, 900),
    )];
    let layers = [
        Some(layer(None, 0, &lower_fill, &[])),
        Some(layer(Some(0), 0, &[], &current_surfaces)),
    ];

    let normal = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::NoFilter,
        CoordinateScale::Normal,
    )
    .unwrap();
    let large_bed = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::NoFilter,
        CoordinateScale::LargeBed,
    )
    .unwrap();

    assert!(!normal.surfaces_by_layer[&1][0].new_polygons.is_empty());
    assert!(large_bed.surfaces_by_layer[&1][0].new_polygons.is_empty());
}

#[test]
fn task22o43_flattened_lower_paths_preserve_a_hole() {
    let lower_fill = [ExPolygon::new(
        rectangle_polygon(0, 0, 10_000, 10_000),
        vec![clockwise_rectangle_polygon(3_000, 3_000, 7_000, 7_000)],
    )];
    let current_surfaces = [surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(4_000, 4_000, 6_000, 6_000),
    )];
    let layers = [
        Some(layer(None, 0, &lower_fill, &[])),
        Some(layer(Some(0), 0, &[], &current_surfaces)),
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
    assert!(candidates[0].new_polygons.is_empty());
}

#[test]
fn task22o43_propagates_an_error_from_initial_unsupported_closing() {
    let invalid_lower_fill = [outside_expolygon()];
    let current_surfaces = [surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(1_000, 1_000, 2_000, 2_000),
    )];
    let layers = [
        Some(layer(None, 0, &invalid_lower_fill, &[])),
        Some(layer(Some(0), 0, &[], &current_surfaces)),
    ];

    assert!(matches!(
        gather_candidates(
            &layers,
            false,
            ProcessInternalBridgeFilter::Disabled,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    ));
}

#[test]
fn task22o43_propagates_an_error_from_later_source_intersection() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let current_surfaces = [surface(
        RegionSurfaceKind::InternalSolid,
        outside_expolygon(),
    )];
    let layers = [
        Some(layer(None, 0, &lower_fill, &[])),
        Some(layer(Some(0), 0, &[], &current_surfaces)),
    ];

    assert!(matches!(
        gather_candidates(
            &layers,
            false,
            ProcessInternalBridgeFilter::Disabled,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    ));
}

fn layer<'a>(
    lower_layer_index: Option<usize>,
    region_index: usize,
    fill_expolygons: &'a [ExPolygon],
    fill_surfaces: &'a [RegionSurface],
) -> CandidateLayer<'a> {
    CandidateLayer {
        lower_layer_index,
        region_index,
        fill_expolygons,
        fill_surfaces,
        sparse_infill_density_percent: 15.0,
        solid_infill_spacing: 1,
    }
}

fn surface(kind: RegionSurfaceKind, expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::new(kind, expolygon)
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(rectangle_polygon(min_x, min_y, max_x, max_y), Vec::new())
}

fn rectangle_polygon(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (max_x, min_y),
        (max_x, max_y),
        (min_x, max_y),
    ])
}

fn clockwise_rectangle_polygon(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    polygon(&[
        (min_x, min_y),
        (min_x, max_y),
        (max_x, max_y),
        (max_x, min_y),
    ])
}

fn outside_expolygon() -> ExPolygon {
    rectangle(
        OUTSIDE_CLIPPER_RANGE,
        0,
        OUTSIDE_CLIPPER_RANGE + 1_000,
        1_000,
    )
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
