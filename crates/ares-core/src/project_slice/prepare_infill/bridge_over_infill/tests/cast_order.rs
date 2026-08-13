use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_unsupported_shrink_multiplies_before_casting_to_f32() {
    let lower_fill = [rectangle(0, 0, 200_000_000, 200_000_000)];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(50_331_649, 60_000_000, 50_331_651, 70_000_000),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &[], 16_777_217)),
        Some(layer(Some(0), &[], &current_surfaces, 16_777_217)),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Disabled,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(!gathered.surfaces_by_layer.contains_key(&1));
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
