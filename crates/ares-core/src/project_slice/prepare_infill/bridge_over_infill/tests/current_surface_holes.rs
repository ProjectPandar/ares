use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_filtered_current_donut_preserves_flat_contour_and_hole() {
    let lower_fill = [ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000, 0),
            Point::new(10_000, 10_000),
            Point::new(0, 10_000),
        ]),
        Vec::new(),
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(2_000, 2_000),
                Point::new(8_000, 2_000),
                Point::new(8_000, 8_000),
                Point::new(2_000, 8_000),
            ]),
            vec![Polygon::new(vec![
                Point::new(4_000, 4_000),
                Point::new(4_000, 6_000),
                Point::new(6_000, 6_000),
                Point::new(6_000, 4_000),
            ])],
        ),
    )];
    let layers = [
        Some(CandidateLayer {
            lower_layer_index: None,
            region_index: 0,
            fill_expolygons: &lower_fill,
            fill_surfaces: &[],
            sparse_infill_density_percent: 15.0,
            solid_infill_spacing: 100,
        }),
        Some(CandidateLayer {
            lower_layer_index: Some(0),
            region_index: 0,
            fill_expolygons: &[],
            fill_surfaces: &current_surfaces,
            sparse_infill_density_percent: 15.0,
            solid_infill_spacing: 100,
        }),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Limited,
        CoordinateScale::Normal,
    )
    .unwrap();
    let polygons = &gathered.surfaces_by_layer[&1][0].new_polygons;
    assert_eq!(
        polygons,
        &[
            Polygon::new(vec![
                Point::new(8_000, 8_000),
                Point::new(2_000, 8_000),
                Point::new(2_000, 2_000),
                Point::new(8_000, 2_000),
            ]),
            Polygon::new(vec![
                Point::new(4_000, 4_000),
                Point::new(4_000, 6_000),
                Point::new(6_000, 6_000),
                Point::new(6_000, 4_000),
            ]),
        ]
    );
    assert_eq!(
        polygons.iter().map(Polygon::area).collect::<Vec<_>>(),
        vec![36_000_000.0, -4_000_000.0]
    );

    let center = Point::new(5_000, 5_000);
    assert!(polygons.iter().all(|polygon| polygon.contains(&center)));
}
