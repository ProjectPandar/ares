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
fn task22o43_closes_unsupported_paths_before_shrinking_them() {
    let lower_fill = [
        rectangle(0, 0, 1_000, 1_000),
        rectangle(1_150, 0, 2_150, 1_000),
    ];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(1_050, 400, 1_100, 600),
    )];
    let layers = [
        Some(layer(None, 5, &lower_fill, &[])),
        Some(layer(Some(0), 7, &[], &current_surfaces)),
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
    assert_eq!(
        candidates[0].source,
        CandidateSource {
            layer_index: 1,
            region_index: 7,
            surface_index: 0,
        }
    );
    assert_eq!(
        candidates[0].new_polygons,
        [Polygon::new(vec![
            Point::new(1_260, 760),
            Point::new(890, 760),
            Point::new(890, 240),
            Point::new(1_260, 240),
        ])]
    );
    let polygon = &candidates[0].new_polygons[0];
    let bounds = polygon.points().iter().fold(
        [i64::MAX, i64::MAX, i64::MIN, i64::MIN],
        |[min_x, min_y, max_x, max_y], point| {
            [
                min_x.min(point.x()),
                min_y.min(point.y()),
                max_x.max(point.x()),
                max_y.max(point.y()),
            ]
        },
    );
    assert_eq!(bounds, [890, 240, 1_260, 760]);
    assert_eq!(polygon.area(), 192_400.0);
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
        solid_infill_spacing: 40,
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
