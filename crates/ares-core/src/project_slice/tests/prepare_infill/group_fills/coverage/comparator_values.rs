use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, ProcessInfillPattern,
    project_slice::{
        group_fills, prepare_infill::combine_infill, region_slices::RegionSurfaceKind,
    },
};

use super::super::focused::fixture::*;

const LAYER: usize = 1;

#[test]
fn task22o73_spacing_and_both_anchor_fields_each_drive_priority_order() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4)]));
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_filament_id = OrcaInt(1);
        options.internal_solid_filament_id = OrcaInt(1);
        options.sparse_infill_pattern = ProcessInfillPattern::Monotonic;
        options.internal_solid_infill_pattern = ProcessInfillPattern::Monotonic;
        options.sparse_infill_density = crate::Percent(100.0);
        options.infill_direction = OrcaFloat(17.0);
        options.solid_infill_direction = OrcaFloat(17.0);
        options.align_infill_direction_to_model = OrcaBool(false);
        options.fill_multiline = OrcaInt(1);
        options.infill_anchor = FloatOrPercent::Float(1_000.0);
        options.infill_anchor_max = FloatOrPercent::Float(1_000.0);
        options.sparse_infill_line_width = FloatOrPercent::Float(0.4);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.6);
    }
    let expolygon = rectangle(0, 0, 2_000_000, 2_000_000);
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
    assert!(grouped.surface_fills[0].params.spacing < grouped.surface_fills[1].params.spacing);
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);

    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_line_width = FloatOrPercent::Float(0.45);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.45);
        options.infill_anchor = FloatOrPercent::Float(500.0);
        options.infill_anchor_max = FloatOrPercent::Float(1_000.0);
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
            .map(|fill| fill.params.anchor_length.to_bits())
            .collect::<Vec<_>>(),
        [500.0_f32.to_bits(), 1_000.0_f32.to_bits()]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_anchor = FloatOrPercent::Float(1_000.0);
        options.infill_anchor_max = FloatOrPercent::Float(2_000.0);
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
            .map(|fill| fill.params.anchor_length_max.to_bits())
            .collect::<Vec<_>>(),
        [1_000.0_f32.to_bits(), 2_000.0_f32.to_bits()]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_flow_width_and_height_each_drive_priority_order() {
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
        options.sparse_infill_density = crate::Percent(100.0);
        options.top_surface_density = crate::Percent(100.0);
        options.infill_direction = OrcaFloat(17.0);
        options.solid_infill_direction = OrcaFloat(17.0);
        options.align_infill_direction_to_model = OrcaBool(false);
        options.fill_multiline = OrcaInt(1);
        options.infill_anchor = FloatOrPercent::Float(1_000.0);
        options.infill_anchor_max = FloatOrPercent::Float(1_000.0);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.45);
        options.top_surface_line_width = FloatOrPercent::Float(0.4);
    }
    let expolygon = rectangle(0, 0, 2_000_000, 2_000_000);
    let mut solid =
        surface_with_height(expolygon.clone(), f64::from(f32::from_bits(1_054_716_112)));
    solid.retag(RegionSurfaceKind::InternalSolid);
    let mut top = surface_with_height(expolygon.clone(), 0.2);
    top.retag(RegionSurfaceKind::Top);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![solid, top];
    let before = graph_snapshot(&graph);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped.surface_fills[0].params.spacing.to_bits(),
        grouped.surface_fills[1].params.spacing.to_bits()
    );
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.flow.width.to_bits())
            .collect::<Vec<_>>(),
        [0.4_f32.to_bits(), 0.45_f32.to_bits()]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);

    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_line_width = FloatOrPercent::Float(0.45);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface_with_height(expolygon.clone(), 0.3),
        surface_with_height(expolygon, 0.2),
    ];
    let before = graph_snapshot(&graph);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped.surface_fills[0].params.spacing.to_bits(),
        grouped.surface_fills[1].params.spacing.to_bits()
    );
    assert_eq!(
        grouped.surface_fills[0].params.flow.width.to_bits(),
        grouped.surface_fills[1].params.flow.width.to_bits()
    );
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.flow.height.to_bits())
            .collect::<Vec<_>>(),
        [0.2_f32.to_bits(), 0.3_f32.to_bits()]
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_flow_nozzle_precedes_extrusion_role_and_drives_priority_order() {
    let mut graph = graph();
    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]));
    {
        let options = options_mut(&mut graph, LAYER);
        options.internal_solid_filament_id = OrcaInt(1);
        options.top_surface_filament_id = OrcaInt(2);
        options.bottom_surface_filament_id = OrcaInt(2);
        options.top_surface_pattern = ProcessInfillPattern::Monotonic;
        options.bottom_surface_pattern = ProcessInfillPattern::Monotonic;
        options.top_surface_density = crate::Percent(100.0);
        options.bottom_surface_density = crate::Percent(100.0);
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.45);
        options.top_surface_line_width = FloatOrPercent::Float(0.45);
        options.solid_infill_direction = OrcaFloat(17.0);
        options.align_infill_direction_to_model = OrcaBool(false);
    }
    let expolygon = rectangle(0, 0, 2_000_000, 2_000_000);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Top, expolygon.clone(), 0),
        surface(RegionSurfaceKind::Bottom, expolygon, 0),
    ];
    let before = graph_snapshot(&graph);
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.extruder)
            .collect::<Vec<_>>(),
        [2, 2]
    );
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .map(|fill| fill.params.flow.nozzle_diameter.to_bits())
            .collect::<Vec<_>>(),
        [0.4_f32.to_bits(), 0.6_f32.to_bits()]
    );
    assert_eq!(
        grouped.surface_fills[0].representative.kind,
        RegionSurfaceKind::Bottom
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 1);
    assert!(grouped.surface_fills[1].expolygons.is_empty());
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}
