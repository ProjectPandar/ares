use crate::{
    ExtrusionRole, FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, Percent,
    ProcessInfillPattern,
    project_slice::{
        group_fills::{self, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
    },
};

use super::super::focused::fixture::*;

const LAYER: usize = 1;

#[test]
fn task22o73_sparse_percent_anchors_clamp_after_f32_projection_and_repeat_exactly() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4)]));
    object_mut(&mut graph).layer_height = OrcaFloat(0.2);
    object_mut(&mut graph).set_other_flow_ratios = OrcaBool(true);
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_pattern = ProcessInfillPattern::Gyroid;
        options.sparse_infill_line_width = FloatOrPercent::Float(0.45);
        options.sparse_infill_speed = OrcaFloat(73.25);
        options.infill_anchor = FloatOrPercent::Float(0.2);
        options.infill_anchor_max = FloatOrPercent::Percent(Percent(33.333_333_333_333_3));
        options.fill_multiline = OrcaInt(4);
        options.gyroid_optimized = OrcaBool(true);
        options.sparse_infill_flow_ratio = OrcaFloat(1.1);
        options.internal_solid_infill_flow_ratio = OrcaFloat(1.2);
    }
    let shape = rectangle(0, 0, 4_000_000, 4_000_000);
    let solid_shape = rectangle(5_000_000, 0, 9_000_000, 4_000_000);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, shape.clone(), 0),
        surface(RegionSurfaceKind::InternalSolid, solid_shape.clone(), 0),
    ];
    let before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();

    let first = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let second = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(options(&graph, LAYER), &options_before);
    let first_sparse = find_kind(&first.surface_fills, RegionSurfaceKind::Internal);
    let second_sparse = find_kind(&second.surface_fills, RegionSurfaceKind::Internal);
    assert_eq!(
        first_sparse.params.pattern,
        SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid)
    );
    assert_eq!(
        first_sparse.params.extrusion_role,
        ExtrusionRole::InternalInfill
    );
    assert_eq!((first_sparse.params.spacing as f32).to_bits(), 0x3ed0_6cbe);
    assert_eq!(first_sparse.params.anchor_length.to_bits(), 0x3e0a_f329);
    assert_eq!(first_sparse.params.anchor_length_max.to_bits(), 0x3e0a_f329);
    assert_eq!(first_sparse.params.multiline, 4);
    assert!(first_sparse.params.gyroid_optimized);
    assert_eq!(first_sparse.params.flow_ratio, 1.1);
    assert_eq!(
        first_sparse.params.flow.mm3_per_mm.to_bits(),
        0x3fb4_d7ac_a000_0000
    );
    assert_eq!(
        first_sparse.params.role_speed.to_bits(),
        73.25_f32.to_bits()
    );
    assert_eq!(first_sparse.expolygons.len(), 1);
    assert_eq!(first_sparse.expolygons[0].contour().points().len(), 4);
    assert_eq!(
        second_sparse.params.spacing.to_bits(),
        first_sparse.params.spacing.to_bits()
    );
    assert_eq!(
        second_sparse.params.anchor_length.to_bits(),
        first_sparse.params.anchor_length.to_bits()
    );
    assert_eq!(second_sparse.expolygons, first_sparse.expolygons);
    let solid = find_kind(&second.surface_fills, RegionSurfaceKind::InternalSolid);
    assert_eq!(solid.params.multiline, 1);
    assert!(!solid.params.gyroid_optimized);
    assert_eq!(solid.params.flow_ratio, 1.2);
    assert_eq!(solid.expolygons, [solid_shape]);

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_anchor = FloatOrPercent::Percent(Percent(50.0));
        options.infill_anchor_max = FloatOrPercent::Float(1_000.0);
    }
    let percent = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let percent = find_kind(&percent.surface_fills, RegionSurfaceKind::Internal);
    assert_eq!(percent.params.anchor_length.to_bits(), 0x3e50_6cbe);
    assert_eq!(
        percent.params.anchor_length_max.to_bits(),
        1_000.0_f32.to_bits()
    );

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_anchor = FloatOrPercent::Float(-0.0);
        options.infill_anchor_max = FloatOrPercent::Float(0.0);
    }
    let signed_zero = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let signed_zero = find_kind(&signed_zero.surface_fills, RegionSurfaceKind::Internal);
    assert_eq!(
        signed_zero.params.anchor_length.to_bits(),
        (-0.0_f32).to_bits()
    );
    assert_eq!(
        signed_zero.params.anchor_length_max.to_bits(),
        0.0_f32.to_bits()
    );
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_bridge_flags_flows_and_sparse_custom_role_speeds_are_independent() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4)]));
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_speed = OrcaFloat(73.25);
        options.bridge_speed = OrcaFloat(40.0);
        options.internal_bridge_speed = FloatOrPercent::Percent(Percent(125.0));
        options.bridge_flow = OrcaFloat(1.0);
        options.internal_bridge_flow = OrcaFloat(0.95);
        options.bottom_solid_infill_flow_ratio = OrcaFloat(1.2);
        options.bridge_line_width = FloatOrPercent::Float(0.4);
    }
    let mut external_bridge = surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(3_000_000, 0, 5_000_000, 2_000_000),
        0,
    );
    external_bridge.set_bridge_angle(0.25);
    let mut internal_bridge = surface(
        RegionSurfaceKind::InternalBridge,
        rectangle(6_000_000, 0, 8_000_000, 2_000_000),
        0,
    );
    internal_bridge.set_bridge_angle(0.5);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(
            RegionSurfaceKind::Internal,
            rectangle(0, 0, 2_000_000, 2_000_000),
            0,
        ),
        external_bridge,
        internal_bridge,
        surface(
            RegionSurfaceKind::Bottom,
            rectangle(9_000_000, 0, 11_000_000, 2_000_000),
            0,
        ),
    ];
    object_mut(&mut graph).thick_bridges = OrcaBool(false);
    object_mut(&mut graph).thick_internal_bridges = OrcaBool(false);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let sparse = find_kind(&grouped.surface_fills, RegionSurfaceKind::Internal);
    let external_bridge = find_kind(&grouped.surface_fills, RegionSurfaceKind::BottomBridge);
    let internal_bridge = find_kind(&grouped.surface_fills, RegionSurfaceKind::InternalBridge);
    let bottom = find_kind(&grouped.surface_fills, RegionSurfaceKind::Bottom);
    assert_eq!(sparse.params.role_speed.to_bits(), 73.25_f32.to_bits());
    assert_eq!(sparse.params.extrusion_role, ExtrusionRole::InternalInfill);
    assert_eq!(
        external_bridge.params.role_speed.to_bits(),
        40.0_f32.to_bits()
    );
    assert_eq!(
        external_bridge.params.extrusion_role,
        ExtrusionRole::BridgeInfill
    );
    assert_eq!(internal_bridge.params.role_speed.to_bits(), 0x4248_0000);
    assert_eq!(
        internal_bridge.params.extrusion_role,
        ExtrusionRole::InternalBridgeInfill
    );
    assert_eq!(external_bridge.params.flow_ratio, 1.0);
    assert_eq!(internal_bridge.params.flow_ratio, 0.95);
    assert_eq!(bottom.params.flow_ratio, 1.2);
    for fill in [external_bridge, internal_bridge] {
        assert!(fill.params.bridge);
        assert!(!fill.params.flow.bridge);
        assert_eq!(fill.params.flow.width.to_bits(), 0x3ecc_cccd);
        assert_eq!(fill.params.flow.height.to_bits(), 0x3e4c_cccd);
        assert_eq!(fill.params.flow.spacing.to_bits(), 0x3eb6_d324);
        assert_eq!(fill.params.flow.mm3_per_mm.to_bits(), 0x3fb2_4850_8000_0000);
    }

    options_mut(&mut graph, LAYER).internal_bridge_speed = FloatOrPercent::Float(17.25);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        find_kind(&grouped.surface_fills, RegionSurfaceKind::InternalBridge)
            .params
            .role_speed
            .to_bits(),
        17.25_f32.to_bits()
    );

    object_mut(&mut graph).thick_bridges = OrcaBool(true);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert!(
        find_kind(&grouped.surface_fills, RegionSurfaceKind::BottomBridge)
            .params
            .flow
            .bridge
    );
    assert!(
        !find_kind(&grouped.surface_fills, RegionSurfaceKind::InternalBridge)
            .params
            .flow
            .bridge
    );

    object_mut(&mut graph).thick_bridges = OrcaBool(false);
    object_mut(&mut graph).thick_internal_bridges = OrcaBool(true);
    let before = graph_snapshot(&graph);
    let object_before = object_mut(&mut graph).clone();
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert!(
        !find_kind(&grouped.surface_fills, RegionSurfaceKind::BottomBridge)
            .params
            .flow
            .bridge
    );
    let thick_internal = find_kind(&grouped.surface_fills, RegionSurfaceKind::InternalBridge);
    assert!(thick_internal.params.bridge);
    assert!(thick_internal.params.flow.bridge);
    assert_eq!(thick_internal.params.flow.width.to_bits(), 0x3ecc_cccd);
    assert_eq!(thick_internal.params.flow.height.to_bits(), 0x3ecc_cccd);
    assert_eq!(thick_internal.params.flow.spacing.to_bits(), 0x3ee6_6667);
    assert_eq!(
        thick_internal.params.flow.mm3_per_mm.to_bits(),
        0x3fc0_15bf_a000_0000
    );
    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(object_mut(&mut graph).clone(), object_before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_lockedzag_flow_key_keeps_first_flow_and_source_geometry_order() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]));
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_pattern = ProcessInfillPattern::LockedZag;
        options.internal_solid_infill_pattern = ProcessInfillPattern::LockedZag;
        options.sparse_infill_filament_id = OrcaInt(1);
        options.internal_solid_filament_id = OrcaInt(2);
        options.skin_infill_line_width = FloatOrPercent::Float(0.5);
        options.skeleton_infill_line_width = FloatOrPercent::Float(0.55);
    }
    let sparse = rectangle(0, 0, 2_000_000, 2_000_000);
    let solid = rectangle(3_000_000, 0, 5_000_000, 2_000_000);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, sparse.clone(), 0),
        surface(RegionSurfaceKind::InternalSolid, solid.clone(), 0),
    ];
    let before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();

    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(options(&graph, LAYER), &options_before);
    assert_eq!(
        find_kind(&grouped.surface_fills, RegionSurfaceKind::Internal)
            .params
            .flow
            .nozzle_diameter
            .to_bits(),
        0.4_f32.to_bits()
    );
    assert_eq!(
        find_kind(&grouped.surface_fills, RegionSurfaceKind::InternalSolid)
            .params
            .flow
            .nozzle_diameter
            .to_bits(),
        0.6_f32.to_bits()
    );
    let lock = &grouped.lock_region_param;
    for entries in [&lock.skin_flow_params, &lock.skeleton_flow_params] {
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].flow.nozzle_diameter.to_bits(), 0.4_f32.to_bits());
        assert_eq!(entries[0].expolygons, [sparse.clone(), solid.clone()]);
    }
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_internal_void_is_an_exact_noop_beside_printable_groups() {
    let mut graph = graph();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(6_000_000, 0, 10_000_000, 4_000_000),
        0,
    )];
    let without_void = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    record_mut(&mut graph, LAYER).fill_surfaces.push(surface(
        RegionSurfaceKind::InternalVoid,
        rectangle(0, 0, 4_000_000, 4_000_000),
        0,
    ));
    let before = graph_snapshot(&graph);
    let with_void = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_snapshot_eq(graph_snapshot(&graph), before);
    let header = super::super::oracle::LayerHeader {
        id: LAYER,
        height: 0.0,
        print_z: 0.0,
    };
    assert_eq!(
        super::super::oracle::metadata(header, &with_void),
        super::super::oracle::metadata(header, &without_void)
    );
    assert_eq!(
        super::super::oracle::authoritative_geometry(&with_void),
        super::super::oracle::authoritative_geometry(&without_void)
    );
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_aligned_absent_layer_is_empty_repeatable_and_nonmutating() {
    let mut graph = graph();
    clear_aligned_layer(&mut graph, LAYER);
    let before = graph_snapshot(&graph);
    let first = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let second = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
    for grouped in [first, second] {
        assert!(grouped.surface_fills.is_empty());
        assert!(grouped.lock_region_param.skin_density_params.is_empty());
        assert!(grouped.lock_region_param.skeleton_density_params.is_empty());
        assert!(grouped.lock_region_param.skin_flow_params.is_empty());
        assert!(grouped.lock_region_param.skeleton_flow_params.is_empty());
    }
}

fn find_kind(
    fills: &[group_fills::SurfaceFill],
    kind: RegionSurfaceKind,
) -> &group_fills::SurfaceFill {
    fills
        .iter()
        .find(|fill| fill.representative.kind == kind)
        .unwrap()
}
