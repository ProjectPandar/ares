use crate::{
    ExtrusionRole, FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, ProcessInfillPattern,
    project_slice::{
        group_fills, prepare_infill::combine_infill, region_slices::RegionSurfaceKind,
    },
};

use super::super::focused::fixture::*;

const LAYER: usize = 1;

#[test]
fn task22o73_all_configured_pattern_ranks_order_every_adjacent_pair() {
    let mut graph = graph();
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_density = crate::Percent(20.0);
        options.sparse_infill_filament_id = OrcaInt(1);
        options.internal_solid_filament_id = OrcaInt(1);
        options.infill_direction = OrcaFloat(17.0);
        options.solid_infill_direction = OrcaFloat(17.0);
        options.align_infill_direction_to_model = OrcaBool(false);
    }
    let sparse = rectangle(0, 0, 2_000_000, 2_000_000);
    let solid = rectangle(3_000_000, 0, 5_000_000, 2_000_000);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, sparse, 0),
        surface(RegionSurfaceKind::InternalSolid, solid, 0),
    ];
    let before = graph_snapshot(&graph);

    for pair in CONFIGURED_PATTERNS.windows(2) {
        let [lower, higher] = pair else {
            unreachable!()
        };
        {
            let options = options_mut(&mut graph, LAYER);
            options.internal_solid_infill_pattern = *lower;
            options.sparse_infill_pattern = *higher;
        }
        let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
        assert_eq!(grouped.surface_fills.len(), 2);
        assert_eq!(grouped.surface_fills[0].params.extruder, 1);
        assert_eq!(grouped.surface_fills[1].params.extruder, 1);
        assert_eq!(
            grouped.surface_fills[0].params.angle.to_bits(),
            grouped.surface_fills[1].params.angle.to_bits()
        );
        assert_eq!(
            grouped.surface_fills[0].params.pattern,
            group_fills::SurfaceFillPattern::Configured(*lower)
        );
        assert_eq!(
            grouped.surface_fills[1].params.pattern,
            group_fills::SurfaceFillPattern::Configured(*higher)
        );
        assert_eq!(
            grouped.surface_fills[0].representative.kind,
            RegionSurfaceKind::InternalSolid
        );
        assert_eq!(
            grouped.surface_fills[1].representative.kind,
            RegionSurfaceKind::Internal
        );
    }
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_reachable_role_ranks_and_output_selector_precedence_are_observable() {
    let mut graph = graph();
    set_nozzles(
        &mut graph,
        OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.5), OrcaFloat(0.6)]),
    );
    {
        let options = options_mut(&mut graph, LAYER);
        options.internal_solid_filament_id = OrcaInt(1);
        options.bottom_surface_filament_id = OrcaInt(1);
        options.top_surface_filament_id = OrcaInt(1);
        options.internal_solid_infill_pattern = ProcessInfillPattern::Monotonic;
        options.sparse_infill_pattern = ProcessInfillPattern::Monotonic;
        options.bottom_surface_pattern = ProcessInfillPattern::Monotonic;
        options.top_surface_pattern = ProcessInfillPattern::Monotonic;
        options.sparse_infill_density = crate::Percent(100.0);
        options.sparse_infill_line_width = FloatOrPercent::Float(0.45);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.45);
        options.top_surface_line_width = FloatOrPercent::Float(0.45);
        options.infill_anchor = FloatOrPercent::Float(1_000.0);
        options.infill_anchor_max = FloatOrPercent::Float(1_000.0);
        options.infill_direction = OrcaFloat(17.0);
        options.solid_infill_direction = OrcaFloat(17.0);
        options.sparse_infill_speed = OrcaFloat(0.0);
        options.internal_solid_infill_speed = OrcaFloat(0.0);
        options.top_surface_speed = OrcaFloat(0.0);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(
            RegionSurfaceKind::Bottom,
            rectangle(0, 0, 1_000_000, 1_000_000),
            0,
        ),
        surface(
            RegionSurfaceKind::Top,
            rectangle(2_000_000, 0, 3_000_000, 1_000_000),
            0,
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            rectangle(4_000_000, 0, 5_000_000, 1_000_000),
            0,
        ),
        surface(
            RegionSurfaceKind::Internal,
            rectangle(6_000_000, 0, 7_000_000, 1_000_000),
            0,
        ),
    ];
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.extrusion_role)
            .collect::<Vec<_>>(),
        [
            ExtrusionRole::InternalInfill,
            ExtrusionRole::SolidInfill,
            ExtrusionRole::TopSolidInfill,
            ExtrusionRole::BottomSurface
        ]
    );

    {
        let options = options_mut(&mut graph, LAYER);
        options.bridge_speed = OrcaFloat(40.0);
        options.internal_bridge_speed = FloatOrPercent::Float(40.0);
        options.bridge_line_width = FloatOrPercent::Float(0.45);
        options.bridge_flow = OrcaFloat(1.0);
    }
    object_mut(&mut graph).thick_bridges = OrcaBool(false);
    object_mut(&mut graph).thick_internal_bridges = OrcaBool(false);
    let mut bridge = surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 2_000_000, 2_000_000),
        0,
    );
    bridge.set_bridge_angle(0.5);
    let mut internal_bridge = surface(
        RegionSurfaceKind::InternalBridge,
        rectangle(3_000_000, 0, 5_000_000, 2_000_000),
        0,
    );
    internal_bridge.set_bridge_angle(0.5);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![internal_bridge, bridge];
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.extrusion_role)
            .collect::<Vec<_>>(),
        [
            ExtrusionRole::BridgeInfill,
            ExtrusionRole::InternalBridgeInfill
        ]
    );

    {
        let options = options_mut(&mut graph, 0);
        options.internal_solid_filament_id = OrcaInt(3);
        options.bottom_surface_filament_id = OrcaInt(2);
    }
    record_mut(&mut graph, 0).fill_surfaces = vec![surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 4_000_000, 4_000_000),
        0,
    )];
    let grouped = group_fills::group_fills(external(&graph), 0, 0).unwrap();
    let fill = &grouped.surface_fills[0];
    assert_eq!(fill.params.extruder, 2);
    assert_eq!(
        fill.params.flow.nozzle_diameter.to_bits(),
        0.6_f32.to_bits()
    );
    assert_eq!(fill.params.extrusion_role, ExtrusionRole::BottomSurface);
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_extruder_precedes_pattern_and_drives_priority_order() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]));
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_filament_id = OrcaInt(2);
        options.internal_solid_filament_id = OrcaInt(1);
        options.sparse_infill_pattern = ProcessInfillPattern::Monotonic;
        options.internal_solid_infill_pattern = ProcessInfillPattern::OctagramSpiral;
        options.sparse_infill_density = crate::Percent(100.0);
        options.sparse_infill_line_width = FloatOrPercent::Float(0.45);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.45);
        options.infill_direction = OrcaFloat(17.0);
        options.solid_infill_direction = OrcaFloat(17.0);
    }
    let expolygon = rectangle(0, 0, 2_000_000, 2_000_000);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, expolygon.clone(), 0),
        surface(RegionSurfaceKind::InternalSolid, expolygon, 0),
    ];
    let before = graph_snapshot(&graph);

    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.extruder)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        grouped.surface_fills[0].representative.kind,
        RegionSurfaceKind::InternalSolid
    );
    assert_eq!(
        grouped.surface_fills[1].representative.kind,
        RegionSurfaceKind::Internal
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_angle_density_and_multiline_each_drive_priority_order() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4)]));
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_filament_id = OrcaInt(1);
        options.internal_solid_filament_id = OrcaInt(1);
        options.top_surface_filament_id = OrcaInt(1);
        options.sparse_infill_pattern = ProcessInfillPattern::Monotonic;
        options.internal_solid_infill_pattern = ProcessInfillPattern::Monotonic;
        options.top_surface_pattern = ProcessInfillPattern::Monotonic;
        options.sparse_infill_line_width = FloatOrPercent::Float(0.45);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.45);
        options.top_surface_line_width = FloatOrPercent::Float(0.45);
        options.infill_anchor = FloatOrPercent::Float(1_000.0);
        options.infill_anchor_max = FloatOrPercent::Float(1_000.0);
        options.sparse_infill_density = crate::Percent(100.0);
        options.top_surface_density = crate::Percent(100.0);
        options.fill_multiline = OrcaInt(1);
    }
    let expolygon = rectangle(0, 0, 2_000_000, 2_000_000);

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_direction = OrcaFloat(11.0);
        options.solid_infill_direction = OrcaFloat(19.0);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::InternalSolid, expolygon.clone(), 0),
        surface(RegionSurfaceKind::Internal, expolygon.clone(), 0),
    ];
    let before = graph_snapshot(&graph);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.representative.kind)
            .collect::<Vec<_>>(),
        [
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::InternalSolid
        ]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_direction = OrcaFloat(17.0);
        options.solid_infill_direction = OrcaFloat(17.0);
        options.sparse_infill_density = crate::Percent(80.0);
        options.top_surface_density = crate::Percent(20.0);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, expolygon.clone(), 0),
        surface(RegionSurfaceKind::Top, expolygon.clone(), 0),
    ];
    let before = graph_snapshot(&graph);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.density.to_bits())
            .collect::<Vec<_>>(),
        [20.0_f32.to_bits(), 80.0_f32.to_bits()]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);

    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_density = crate::Percent(100.0);
        options.fill_multiline = OrcaInt(3);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, expolygon.clone(), 0),
        surface(RegionSurfaceKind::InternalSolid, expolygon, 0),
    ];
    let before = graph_snapshot(&graph);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.multiline)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

const CONFIGURED_PATTERNS: [ProcessInfillPattern; 28] = [
    ProcessInfillPattern::Monotonic,
    ProcessInfillPattern::MonotonicLine,
    ProcessInfillPattern::Rectilinear,
    ProcessInfillPattern::AlignedRectilinear,
    ProcessInfillPattern::ZigZag,
    ProcessInfillPattern::CrossZag,
    ProcessInfillPattern::LockedZag,
    ProcessInfillPattern::Line,
    ProcessInfillPattern::Grid,
    ProcessInfillPattern::Triangles,
    ProcessInfillPattern::TriHexagon,
    ProcessInfillPattern::Cubic,
    ProcessInfillPattern::AdaptiveCubic,
    ProcessInfillPattern::QuarterCubic,
    ProcessInfillPattern::SupportCubic,
    ProcessInfillPattern::Lightning,
    ProcessInfillPattern::Honeycomb,
    ProcessInfillPattern::ThreeDHoneycomb,
    ProcessInfillPattern::LateralHoneycomb,
    ProcessInfillPattern::LateralLattice,
    ProcessInfillPattern::CrossHatch,
    ProcessInfillPattern::TpmsD,
    ProcessInfillPattern::TpmsFk,
    ProcessInfillPattern::Gyroid,
    ProcessInfillPattern::Concentric,
    ProcessInfillPattern::HilbertCurve,
    ProcessInfillPattern::ArchimedeanChords,
    ProcessInfillPattern::OctagramSpiral,
];
