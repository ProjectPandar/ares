use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_leftover_area_must_strictly_exceed_spacing_squared() {
    let candidate_areas =
        [100, 101].map(|width| candidate_area_with_right_leftover(CoordinateScale::Normal, width));

    assert_eq!(candidate_areas, [140_000.0, 160_100.0]);
}

#[test]
fn task22o43_leftover_area_must_be_strictly_below_twelve_mm_strip() {
    let candidate_areas = [
        (CoordinateScale::Normal, 12_000_000),
        (CoordinateScale::Normal, 11_999_999),
        (CoordinateScale::LargeBed, 1_200_000),
        (CoordinateScale::LargeBed, 1_199_999),
    ]
    .map(|(scale, width)| candidate_area_with_right_leftover(scale, width));

    assert_eq!(
        candidate_areas,
        [140_000.0, 1_200_149_900.0, 140_000.0, 120_139_900.0]
    );
}

fn candidate_area_with_right_leftover(scale: CoordinateScale, leftover_width: i64) -> f64 {
    const SPACING: i64 = 100;
    let lower_fill = [rectangle(-SPACING, -SPACING, 1_100, 200)];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(0, 0, 1_500 + leftover_width, 100),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &[], 15.0, SPACING)),
        Some(layer(Some(0), &[], &current_surfaces, 15.0, SPACING)),
    ];

    gather_candidates(&layers, false, ProcessInternalBridgeFilter::Limited, scale)
        .unwrap()
        .surfaces_by_layer[&1][0]
        .new_polygons
        .iter()
        .map(Polygon::area)
        .sum()
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
