use crate::{
    OrcaBool, ProcessInfillPattern,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        group_fills::{self, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
    },
};

use super::focused::fixture::{
    assert_snapshot_eq, external, graph, graph_snapshot, object_mut, options_mut, record_mut,
    rectangle, surface,
};
use super::oracle::authoritative_geometry;
use crate::project_slice::tests::prepare_infill::bridge_over_infill::transaction::sha256;

const PARTIAL_LAYER: usize = 1;
const ALL_NARROW_LAYER: usize = 4;

#[test]
fn task22o74_ksr_partial_and_all_narrow_mutations_preserve_source_identity_and_order() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    let before = graph_snapshot(&graph);

    let partial = group_fills::group_fills(external(&graph), 0, PARTIAL_LAYER).unwrap();
    let partial_repeat = group_fills::group_fills(external(&graph), 0, PARTIAL_LAYER).unwrap();
    let all_narrow = group_fills::group_fills(external(&graph), 0, ALL_NARROW_LAYER).unwrap();
    assert_snapshot_eq(graph_snapshot(&graph), before);

    assert_eq!(partial.surface_fills.len(), 2);
    assert_eq!(partial.surface_fills[0].params.idx, 0);
    assert_eq!(partial.surface_fills[1].params.idx, 0);
    assert_eq!(
        partial.surface_fills[0].representative.kind,
        RegionSurfaceKind::InternalSolid
    );
    assert_eq!(
        partial.surface_fills[1].representative.kind,
        RegionSurfaceKind::InternalSolid
    );
    assert_eq!(
        partial.surface_fills[0].representative.thickness.to_bits(),
        partial.surface_fills[1].representative.thickness.to_bits()
    );
    assert_eq!(
        partial.surface_fills[0].params.pattern,
        configured(ProcessInfillPattern::Monotonic)
    );
    assert_eq!(
        partial.surface_fills[1].params.pattern,
        SurfaceFillPattern::ConcentricInternal
    );
    assert_eq!(partial.surface_fills[1].representative.thickness_layers, 1);
    assert_eq!(partial.surface_fills[1].representative.bridge_angle, -1.0);
    assert_eq!(partial.surface_fills[1].representative.extra_perimeters, 0);
    assert_eq!(
        partial.surface_fills[1].region_id,
        partial.surface_fills[0].region_id
    );
    assert_eq!(
        partial.surface_fills[1].region_id_group,
        partial.surface_fills[0].region_id_group
    );
    assert_eq!(
        partial.surface_fills[1].no_overlap_expolygons,
        partial.surface_fills[0].no_overlap_expolygons
    );
    assert_eq!(
        super::oracle::authoritative_geometry(&partial_repeat),
        super::oracle::authoritative_geometry(&partial)
    );

    let converted = all_narrow
        .surface_fills
        .iter()
        .find(|fill| fill.representative.kind == RegionSurfaceKind::InternalSolid)
        .unwrap();
    assert_eq!(
        converted.params.pattern,
        SurfaceFillPattern::ConcentricInternal
    );
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_partial_split_resets_only_synthetic_metadata_and_preserves_ordered_topology() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    options_mut(&mut graph, PARTIAL_LAYER).internal_solid_infill_line_width =
        crate::FloatOrPercent::Float(0.42);
    let source = dog_bone();
    let mut rich = crate::project_slice::region_slices::RegionSurface::internal_with_metadata(
        source, 0.3, 3, 0.75, 7,
    );
    rich.retag(RegionSurfaceKind::InternalSolid);
    let record = record_mut(&mut graph, PARTIAL_LAYER);
    record.fill_surfaces = vec![rich];
    record.fill_no_overlap_expolygons = vec![rectangle(20_000_000, 0, 21_000_000, 1_000_000)];

    let grouped = group_fills::group_fills(external(&graph), 0, PARTIAL_LAYER).unwrap();
    assert_eq!(grouped.surface_fills.len(), 2);
    let original = &grouped.surface_fills[0];
    let synthetic = &grouped.surface_fills[1];
    assert_eq!(
        original.representative.thickness.to_bits(),
        0.3_f64.to_bits()
    );
    assert_eq!(original.representative.thickness_layers, 3);
    assert_eq!(
        original.representative.bridge_angle.to_bits(),
        0.75_f64.to_bits()
    );
    assert_eq!(original.representative.extra_perimeters, 7);
    assert_eq!(
        synthetic.representative.thickness.to_bits(),
        0.3_f64.to_bits()
    );
    assert_eq!(synthetic.representative.thickness_layers, 1);
    assert_eq!(synthetic.representative.bridge_angle, -1.0);
    assert_eq!(synthetic.representative.extra_perimeters, 0);
    assert_eq!(synthetic.params.idx, original.params.idx);
    assert_eq!(
        original.params.flow.mm3_per_mm.to_bits(),
        0x3fbb_4fc3_4000_0000
    );
    assert_eq!(
        synthetic.params.flow.mm3_per_mm.to_bits(),
        0x3fbb_4fc3_4000_0000
    );
    assert_eq!(synthetic.region_id_group, original.region_id_group);
    assert_eq!(
        synthetic.no_overlap_expolygons,
        original.no_overlap_expolygons
    );
    assert!(!original.expolygons.is_empty());
    assert!(!synthetic.expolygons.is_empty());
    assert_eq!(
        sha256(&authoritative_geometry(&grouped)),
        "b9c5e3d97269820130c5fbf1387b7c3856ad7aa7567ccc7ec7547f933c463cb4"
    );
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_non_line_split_distinguishes_full_core_no_core_and_partial_core() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    options_mut(&mut graph, PARTIAL_LAYER).internal_solid_infill_pattern =
        ProcessInfillPattern::Grid;

    let wide = rectangle(0, 0, 4_000_000, 4_000_000);
    record_mut(&mut graph, PARTIAL_LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, wide.clone(), 0)];
    let full_core = group_fills::group_fills(external(&graph), 0, PARTIAL_LAYER).unwrap();
    assert_eq!(full_core.surface_fills.len(), 1);
    assert_eq!(
        full_core.surface_fills[0].params.pattern,
        configured(ProcessInfillPattern::Grid)
    );
    assert_eq!(full_core.surface_fills[0].expolygons, [wide]);

    let thin = rectangle(0, 0, 200_000, 4_000_000);
    record_mut(&mut graph, PARTIAL_LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, thin.clone(), 0)];
    let no_core = group_fills::group_fills(external(&graph), 0, PARTIAL_LAYER).unwrap();
    assert_eq!(no_core.surface_fills.len(), 1);
    assert_eq!(
        no_core.surface_fills[0].params.pattern,
        SurfaceFillPattern::ConcentricInternal
    );
    assert_eq!(no_core.surface_fills[0].expolygons, [thin]);

    record_mut(&mut graph, PARTIAL_LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, dog_bone(), 0)];
    let partial = group_fills::group_fills(external(&graph), 0, PARTIAL_LAYER).unwrap();
    assert_eq!(partial.surface_fills.len(), 2);
    assert_eq!(
        partial.surface_fills[0].params.pattern,
        configured(ProcessInfillPattern::Grid)
    );
    assert_eq!(
        partial.surface_fills[1].params.pattern,
        SurfaceFillPattern::ConcentricInternal
    );
    assert!(!partial.surface_fills[0].expolygons.is_empty());
    assert!(!partial.surface_fills[1].expolygons.is_empty());
    assert_eq!(
        sha256(&authoritative_geometry(&partial)),
        "0010b87782cb4a432c42f274825566880bb8d126b021529c76dea41e0f128b5e"
    );
    combine_infill::dispose(graph);
}

fn configured(pattern: ProcessInfillPattern) -> SurfaceFillPattern {
    SurfaceFillPattern::Configured(pattern)
}

fn dog_bone() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(4_000_000, 0),
            Point::new(4_000_000, 1_900_000),
            Point::new(6_000_000, 1_900_000),
            Point::new(6_000_000, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 4_000_000),
            Point::new(6_000_000, 4_000_000),
            Point::new(6_000_000, 2_100_000),
            Point::new(4_000_000, 2_100_000),
            Point::new(4_000_000, 4_000_000),
            Point::new(0, 4_000_000),
        ]),
        Vec::new(),
    )
}
