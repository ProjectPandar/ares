use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::candidates::{CandidateLayer, gather_candidates};

#[test]
fn task22o43_gathers_a_wholly_unsupported_internal_solid() {
    let lower_fill = [rectangle(0, 0, 10_000_000, 10_000_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::Internal,
        lower_fill[0].clone(),
    )];
    let current_surfaces = [RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        rectangle(2_000_000, 2_000_000, 8_000_000, 8_000_000),
    )];
    let layers = [
        Some(layer(None, &lower_fill, &lower_surfaces, 100_000)),
        Some(layer(Some(0), &[], &current_surfaces, 100_000)),
    ];

    let gathered = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::Disabled,
        CoordinateScale::Normal,
    )
    .unwrap();

    let candidates = gathered.surfaces_by_layer.get(&1).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source.layer_index, 1);
    assert_eq!(candidates[0].source.region_index, 0);
    assert_eq!(candidates[0].source.surface_index, 0);
    assert_eq!(candidates[0].bridge_angle.to_bits(), 0.0_f64.to_bits());
    assert!(!candidates[0].new_polygons.is_empty());
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
