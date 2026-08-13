use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::{
    candidates::{CandidateLayer, gather_candidates},
    types::CandidateSource,
};

#[test]
fn task22o43_top_donut_supports_its_ring_but_not_its_hole() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let lower_surfaces = [RegionSurface::new(RegionSurfaceKind::Top, donut())];
    let current_surfaces = [
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            rectangle(1_500, 1_500, 2_500, 2_500),
        ),
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            rectangle(4_000, 4_000, 6_000, 6_000),
        ),
    ];
    let layers = [
        Some(layer(None, 7, &lower_fill, &lower_surfaces)),
        Some(layer(Some(0), 7, &[], &current_surfaces)),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Limited,
        CoordinateScale::Normal,
    )
    .unwrap();

    let candidates = &gathered.surfaces_by_layer[&1];
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].source,
        CandidateSource {
            layer_index: 1,
            region_index: 7,
            surface_index: 1,
        }
    );
    assert_eq!(candidates[0].new_polygons.len(), 1);
    assert_eq!(candidates[0].new_polygons[0].area(), 4_000_000.0);
    let mut points = candidates[0].new_polygons[0]
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect::<Vec<_>>();
    points.sort_unstable();
    assert_eq!(
        points,
        [
            (4_000, 4_000),
            (4_000, 6_000),
            (6_000, 4_000),
            (6_000, 6_000),
        ]
    );
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
        solid_infill_spacing: 100,
    }
}

fn donut() -> ExPolygon {
    ExPolygon::new(
        rectangle_polygon(1_000, 1_000, 9_000, 9_000),
        vec![clockwise_rectangle_polygon(3_500, 3_500, 6_500, 6_500)],
    )
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

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
