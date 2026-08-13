use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_filter_policy_geometry_freezes_expansion_and_final_clip() {
    let lower_fill = [rectangle(100, 400, 600, 1_000)];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        notched_source(),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &[], 100)),
        Some(layer(Some(0), &[], &current_surfaces, 100)),
    ];

    let [no_filter, filtered] = [
        ProcessInternalBridgeFilter::NoFilter,
        ProcessInternalBridgeFilter::Limited,
    ]
    .map(|filter| {
        gather_candidates(&layers, false, filter, CoordinateScale::Normal)
            .unwrap()
            .surfaces_by_layer[&1][0]
            .new_polygons
            .clone()
    });

    assert_geometry(
        &no_filter,
        [(-200, 100), (900, 1_300)],
        1_320_000.0,
        &[(-200, 100), (-200, 1_300), (900, 100), (900, 1_300)],
    );
    assert_geometry(
        &filtered,
        [(0, 100), (900, 1_300)],
        1_070_000.0,
        &[
            (0, 100),
            (0, 700),
            (0, 800),
            (0, 1_300),
            (100, 700),
            (100, 800),
            (900, 100),
            (900, 1_300),
        ],
    );
}

fn assert_geometry(
    polygons: &[Polygon],
    expected_bounds: [(i64, i64); 2],
    expected_area: f64,
    expected_points: &[(i64, i64)],
) {
    assert_eq!(polygons.len(), 1);
    let polygon = &polygons[0];
    let mut points = polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect::<Vec<_>>();
    points.sort_unstable();
    let bounds = [points[0], points[points.len() - 1]];

    assert_eq!(bounds, expected_bounds);
    assert_eq!(polygon.area().abs(), expected_area);
    assert_eq!(points, expected_points);
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

fn notched_source() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000, 0),
            Point::new(1_000, 1_410),
            Point::new(0, 1_410),
            Point::new(0, 800),
            Point::new(100, 800),
            Point::new(100, 700),
            Point::new(0, 700),
        ]),
        Vec::new(),
    )
}
