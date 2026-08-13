use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_candidate_gather_uses_current_layer_solid_spacing() {
    let lower_fill = [rectangle(0, 0, 10_000, 10_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::Internal,
        lower_fill[0].clone(),
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(0, 1_000, 401, 1_300),
    )];

    let retained = [(1_000, 100), (100, 1_000)].map(|(lower_spacing, current_spacing)| {
        let layers = [
            Some(layer(
                None,
                &lower_fill,
                &lower_surfaces,
                15.0,
                lower_spacing,
            )),
            Some(layer(
                Some(0),
                &[],
                &current_surfaces,
                15.0,
                current_spacing,
            )),
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

    assert_eq!(retained, [true, false]);
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
