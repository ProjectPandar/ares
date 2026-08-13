use crate::{
    ProcessInternalBridgeFilter,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

use super::super::{
    candidates::{CandidateLayer, gather_candidates},
    types::BridgeCandidateObject,
};

#[test]
fn task22o43_lightning_detection_does_not_change_candidate_inventory() {
    let lower_fill = [rectangle(0, 0, 20_000, 20_000)];
    let lower_surfaces = [RegionSurface::new(
        RegionSurfaceKind::Internal,
        lower_fill[0].clone(),
    )];
    let current_surfaces = [
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            rectangle(1_000, 1_000, 5_000, 5_000),
        ),
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            rectangle(8_000, 8_000, 12_000, 12_000),
        ),
    ];
    let layers = [
        Some(layer(None, &lower_fill, &lower_surfaces)),
        Some(layer(Some(0), &[], &current_surfaces)),
    ];

    let without_lightning = gather_candidates(
        &layers,
        false,
        ProcessInternalBridgeFilter::NoFilter,
        CoordinateScale::Normal,
    )
    .unwrap();
    let with_lightning = gather_candidates(
        &layers,
        true,
        ProcessInternalBridgeFilter::NoFilter,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert!(!without_lightning.has_lightning_infill);
    assert!(with_lightning.has_lightning_infill);
    let without_inventory = inventory(&without_lightning);
    let with_inventory = inventory(&with_lightning);
    assert!(!without_inventory.is_empty());
    assert_eq!(with_inventory, without_inventory);
}

fn inventory(
    object: &BridgeCandidateObject,
) -> Vec<(usize, super::super::types::CandidateSource, &[Polygon], u64)> {
    object
        .surfaces_by_layer
        .iter()
        .flat_map(|(&layer_index, candidates)| {
            candidates.iter().map(move |candidate| {
                (
                    layer_index,
                    candidate.source,
                    candidate.new_polygons.as_slice(),
                    candidate.bridge_angle.to_bits(),
                )
            })
        })
        .collect()
}

fn layer<'a>(
    lower_layer_index: Option<usize>,
    fill_expolygons: &'a [ExPolygon],
    fill_surfaces: &'a [RegionSurface],
) -> CandidateLayer<'a> {
    CandidateLayer {
        lower_layer_index,
        region_index: 7,
        fill_expolygons,
        fill_surfaces,
        sparse_infill_density_percent: 15.0,
        solid_infill_spacing: 500,
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
