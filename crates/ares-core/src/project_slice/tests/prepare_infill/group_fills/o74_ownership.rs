use crate::{
    FloatOrPercent, OrcaBool, ProcessInfillPattern,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        group_fills::{self, GroupedFills, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::focused::fixture::{
    assert_snapshot_eq, external, graph, graph_snapshot, object_mut, options, options_mut,
    record_mut, rectangle, surface,
};

const LAYER: usize = 1;

#[test]
fn task22o74_partial_groups_append_after_the_original_prefix_with_copied_source_indices() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    let mut first = RegionSurface::internal_with_metadata(dog_bone(0), 0.2, 2, 0.5, 7);
    first.retag(RegionSurfaceKind::InternalSolid);
    let mut second = RegionSurface::internal_with_metadata(dog_bone(20_000_000), 0.3, 3, 0.75, 9);
    second.retag(RegionSurfaceKind::InternalSolid);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![first, second];
    let before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();

    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(grouped.surface_fills.len(), 4);
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.idx)
            .collect::<Vec<_>>(),
        [0, 1, 0, 1]
    );
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.pattern)
            .collect::<Vec<_>>(),
        [
            configured(ProcessInfillPattern::Monotonic),
            configured(ProcessInfillPattern::Monotonic),
            SurfaceFillPattern::ConcentricInternal,
            SurfaceFillPattern::ConcentricInternal,
        ]
    );
    assert_eq!(grouped.surface_fills[0].representative.thickness_layers, 3);
    assert_eq!(grouped.surface_fills[1].representative.thickness_layers, 2);
    assert_eq!(grouped.surface_fills[2].representative.thickness_layers, 1);
    assert_eq!(grouped.surface_fills[3].representative.thickness_layers, 1);
    assert!(
        grouped
            .surface_fills
            .iter()
            .all(|fill| !fill.expolygons.is_empty())
    );
    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(options(&graph, LAYER), &options_before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_active_narrow_tail_preserves_all_lockedzag_sidecars_and_non_solid_groups() {
    let mut graph = graph();
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_pattern = ProcessInfillPattern::LockedZag;
        options.internal_solid_infill_pattern = ProcessInfillPattern::Monotonic;
        options.skin_infill_line_width = FloatOrPercent::Float(0.5);
        options.skeleton_infill_line_width = FloatOrPercent::Float(0.55);
    }
    let sparse = rectangle(20_000_000, 0, 20_200_000, 4_000_000);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, sparse, 0),
        surface(RegionSurfaceKind::InternalSolid, dog_bone(0), 0),
    ];
    let disabled = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    let before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();
    let object_before = object_mut(&mut graph).clone();
    let enabled = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_lock_sidecars_equal(&enabled, &disabled);
    let enabled_sparse = enabled
        .surface_fills
        .iter()
        .find(|fill| fill.representative.kind == RegionSurfaceKind::Internal)
        .unwrap();
    let disabled_sparse = disabled
        .surface_fills
        .iter()
        .find(|fill| fill.representative.kind == RegionSurfaceKind::Internal)
        .unwrap();
    assert_eq!(
        enabled_sparse.params.pattern,
        disabled_sparse.params.pattern
    );
    assert_eq!(enabled_sparse.expolygons, disabled_sparse.expolygons);
    assert_eq!(enabled_sparse.expolygons.len(), 1);
    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(options(&graph, LAYER), &options_before);
    assert_eq!(object_mut(&mut graph).clone(), object_before);
    combine_infill::dispose(graph);
}

fn assert_lock_sidecars_equal(actual: &GroupedFills, expected: &GroupedFills) {
    let actual = &actual.lock_region_param;
    let expected = &expected.lock_region_param;
    assert_eq!(
        actual.skin_density_params.len(),
        expected.skin_density_params.len()
    );
    assert_eq!(
        actual.skeleton_density_params.len(),
        expected.skeleton_density_params.len()
    );
    assert_eq!(
        actual.skin_flow_params.len(),
        expected.skin_flow_params.len()
    );
    assert_eq!(
        actual.skeleton_flow_params.len(),
        expected.skeleton_flow_params.len()
    );
    assert!(
        actual
            .skin_density_params
            .iter()
            .zip(&expected.skin_density_params)
            .all(
                |(left, right)| left.density.to_bits() == right.density.to_bits()
                    && left.expolygons == right.expolygons
            )
    );
    assert!(
        actual
            .skeleton_density_params
            .iter()
            .zip(&expected.skeleton_density_params)
            .all(
                |(left, right)| left.density.to_bits() == right.density.to_bits()
                    && left.expolygons == right.expolygons
            )
    );
    assert!(
        actual
            .skin_flow_params
            .iter()
            .zip(&expected.skin_flow_params)
            .all(|(left, right)| flow_bits_equal(left.flow, right.flow)
                && left.expolygons == right.expolygons)
    );
    assert!(
        actual
            .skeleton_flow_params
            .iter()
            .zip(&expected.skeleton_flow_params)
            .all(|(left, right)| flow_bits_equal(left.flow, right.flow)
                && left.expolygons == right.expolygons)
    );
}

fn flow_bits_equal(
    left: crate::project_slice::perimeters::types::Flow,
    right: crate::project_slice::perimeters::types::Flow,
) -> bool {
    left.width.to_bits() == right.width.to_bits()
        && left.height.to_bits() == right.height.to_bits()
        && left.spacing.to_bits() == right.spacing.to_bits()
        && left.nozzle_diameter.to_bits() == right.nozzle_diameter.to_bits()
        && left.bridge == right.bridge
        && left.mm3_per_mm.to_bits() == right.mm3_per_mm.to_bits()
}

fn configured(pattern: ProcessInfillPattern) -> SurfaceFillPattern {
    SurfaceFillPattern::Configured(pattern)
}

fn dog_bone(offset: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(offset, 0),
            Point::new(offset + 4_000_000, 0),
            Point::new(offset + 4_000_000, 1_900_000),
            Point::new(offset + 6_000_000, 1_900_000),
            Point::new(offset + 6_000_000, 0),
            Point::new(offset + 10_000_000, 0),
            Point::new(offset + 10_000_000, 4_000_000),
            Point::new(offset + 6_000_000, 4_000_000),
            Point::new(offset + 6_000_000, 2_100_000),
            Point::new(offset + 4_000_000, 2_100_000),
            Point::new(offset + 4_000_000, 4_000_000),
            Point::new(offset, 4_000_000),
        ]),
        Vec::new(),
    )
}
