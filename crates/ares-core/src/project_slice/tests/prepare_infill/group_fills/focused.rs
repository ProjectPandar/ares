pub(in crate::project_slice::tests::prepare_infill) mod fixture;

use crate::{
    ExtrusionRole, FloatOrPercent, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, OrcaString, Percent,
    ProcessInfillPattern, SliceError, Transform3d,
    geometry::ExPolygon,
    project_slice::{
        group_fills::{self, LockFlowParam, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use fixture::*;

const LAYER: usize = 1;

#[test]
fn task22o73_projection_uses_source_precision_roles_selectors_and_density_skips() {
    let mut graph = graph();
    let shape = rectangle(0, 0, 4_000_000, 4_000_000);

    let mut first_layer_bridge = surface(RegionSurfaceKind::BottomBridge, shape.clone(), 0);
    first_layer_bridge.set_bridge_angle(0.375);
    record_mut(&mut graph, 0).fill_surfaces = vec![first_layer_bridge];
    let grouped = group_fills::group_fills(external(&graph), 0, 0).unwrap();
    let fill = &grouped.surface_fills[0];
    assert_eq!(fill.representative.kind, RegionSurfaceKind::BottomBridge);
    assert!(!fill.params.bridge);
    assert!(!fill.params.flow.bridge);
    assert_eq!(fill.params.extrusion_role, ExtrusionRole::BottomSurface);
    assert_eq!(fill.params.role_speed.to_bits(), 0);

    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::Internal, shape.clone(), 0)];
    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_direction = OrcaFloat(1.792621887649045);
        options.align_infill_direction_to_model = OrcaBool(false);
    }
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(grouped.surface_fills[0].params.angle.to_bits(), 0x3d00_26f5);

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_direction = OrcaFloat(-0.0);
        options.align_infill_direction_to_model = OrcaBool(false);
    }
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        grouped.surface_fills[0].params.angle.to_bits(),
        (-0.0_f32).to_bits()
    );

    {
        let options = options_mut(&mut graph, LAYER);
        options.infill_direction = OrcaFloat(0.0);
        options.align_infill_direction_to_model = OrcaBool(true);
    }
    set_transform(
        &mut graph,
        Transform3d::parse_3mf(
            "0.5562550826544693 0.8310116022180855 0 -0.8310116022180855 0.5562550826544693 0 0 0 1 0 0 0",
        )
        .unwrap(),
    );
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(grouped.surface_fills[0].params.angle.to_bits(), 0x3f7b_1dd2);

    let mut positive_zero = surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 2_000_000, 2_000_000),
        0,
    );
    positive_zero.set_bridge_angle(0.0);
    let mut negative_zero = surface(
        RegionSurfaceKind::BottomBridge,
        rectangle(3_000_000, 0, 5_000_000, 2_000_000),
        0,
    );
    negative_zero.set_bridge_angle(-0.0);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![positive_zero, negative_zero];
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(grouped.surface_fills.len(), 1);
    assert_eq!(
        grouped.surface_fills[0]
            .representative
            .bridge_angle
            .to_bits(),
        0
    );
    assert_eq!(grouped.surface_fills[0].expolygons.len(), 2);

    set_nozzles(&mut graph, OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]));
    object_mut(&mut graph).line_width = FloatOrPercent::Float(0.0);
    {
        let options = options_mut(&mut graph, LAYER);
        options.align_infill_direction_to_model = OrcaBool(false);
        options.top_surface_filament_id = OrcaInt(2);
        options.top_surface_line_width = FloatOrPercent::Float(0.0);
        options.top_surface_density = Percent(100.0);
    }
    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::Top, shape.clone(), 0)];
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let top = &grouped.surface_fills[0];
    assert_eq!(top.params.extruder, 2);
    assert_eq!(top.params.extrusion_role, ExtrusionRole::TopSolidInfill);
    assert_eq!(top.params.flow.nozzle_diameter.to_bits(), 0.6_f32.to_bits());
    assert_eq!(top.params.flow.width.to_bits(), 0.6_f32.to_bits());

    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_density = Percent(0.0);
        options.top_surface_density = Percent(0.0);
    }
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, shape.clone(), 0),
        surface(RegionSurfaceKind::Top, shape, 0),
    ];
    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert!(grouped.surface_fills.is_empty());
    combine_infill::dispose(graph);
}

#[test]
fn task22o73_lockedzag_retains_sticky_params_and_materializes_four_sorted_sidecars() {
    let mut graph = graph();
    let low = rectangle(0, 0, 2_000_000, 2_000_000);
    let high = rectangle(3_000_000, 0, 5_000_000, 2_000_000);
    let solid = rectangle(6_000_000, 0, 8_000_000, 2_000_000);
    let no_overlap = rectangle(-1_000_000, -1_000_000, 9_000_000, 3_000_000);
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_pattern = ProcessInfillPattern::LockedZag;
        options.sparse_infill_density = Percent(37.0);
        options.infill_lock_depth = OrcaFloat(1.25);
        options.skin_infill_depth = OrcaFloat(2.5);
        options.symmetric_infill_y_axis = OrcaBool(true);
        options.skin_infill_density = Percent(35.0);
        options.skeleton_infill_density = Percent(15.0);
        options.skin_infill_line_width = FloatOrPercent::Float(0.42);
        options.skeleton_infill_line_width = FloatOrPercent::Float(0.58);
    }
    let record = record_mut(&mut graph, LAYER);
    record.fill_surfaces = vec![
        surface_with_height(high.clone(), 0.3),
        surface_with_height(low.clone(), 0.2),
        surface(RegionSurfaceKind::InternalSolid, solid, 0),
    ];
    record.fill_no_overlap_expolygons = vec![no_overlap.clone()];
    let before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();

    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(options(&graph, LAYER), &options_before);
    combine_infill::dispose(graph);
    assert_eq!(grouped.surface_fills.len(), 3);
    for fill in &grouped.surface_fills {
        assert_eq!(fill.params.infill_lock_depth, 1_250_000.0);
        assert_eq!(fill.params.skin_infill_depth, 2_500_000.0);
        assert!(fill.params.symmetric_infill_y_axis);
        assert_eq!(
            fill.no_overlap_expolygons.as_slice(),
            std::slice::from_ref(&no_overlap)
        );
    }
    let solid = grouped
        .surface_fills
        .iter()
        .find(|fill| fill.representative.kind == RegionSurfaceKind::InternalSolid)
        .unwrap();
    assert_eq!(
        solid.params.pattern,
        SurfaceFillPattern::Configured(ProcessInfillPattern::Monotonic)
    );
    assert_eq!(
        grouped
            .surface_fills
            .iter()
            .filter(|fill| fill.params.pattern
                == SurfaceFillPattern::Configured(ProcessInfillPattern::LockedZag))
            .count(),
        2
    );

    let lock = &grouped.lock_region_param;
    assert_eq!(lock.skin_density_params.len(), 1);
    assert_eq!(lock.skeleton_density_params.len(), 1);
    assert_eq!(
        lock.skin_density_params[0].density.to_bits(),
        0.35_f32.to_bits()
    );
    assert_eq!(
        lock.skeleton_density_params[0].density.to_bits(),
        0.15_f32.to_bits()
    );
    assert_eq!(
        lock.skin_density_params[0].expolygons,
        [high.clone(), low.clone()]
    );
    assert_eq!(
        lock.skeleton_density_params[0].expolygons,
        [high.clone(), low.clone()]
    );
    assert!(
        lock.skin_flow_params
            .iter()
            .all(|entry| entry.flow.width.to_bits() == 0.42_f32.to_bits())
    );
    assert!(
        lock.skeleton_flow_params
            .iter()
            .all(|entry| entry.flow.width.to_bits() == 0.58_f32.to_bits())
    );
    assert_eq!(
        lock.skin_flow_params
            .iter()
            .map(|entry| entry.flow.mm3_per_mm.to_bits())
            .collect::<Vec<_>>(),
        [0x3fb3_4e75_4000_0000, 0x3fbb_4fc3_4000_0000]
    );
    assert_eq!(
        lock.skeleton_flow_params
            .iter()
            .map(|entry| entry.flow.mm3_per_mm.to_bits())
            .collect::<Vec<_>>(),
        [0x3fbb_7f9c_2000_0000, 0x3fc3_ccbe_e000_0000]
    );
    assert_sorted_flow_sidecar(&lock.skin_flow_params, &[low.clone(), high.clone()]);
    assert_sorted_flow_sidecar(&lock.skeleton_flow_params, &[low, high]);
}

#[test]
fn task22o73_priority_coalesces_first_multi_retains_empty_and_reports_range_atomically() {
    let mut graph = graph();
    let first = rectangle(0, 0, 4_000_000, 4_000_000);
    let second = rectangle(6_000_000, 0, 10_000_000, 4_000_000);
    let covered = rectangle(1_000_000, 1_000_000, 2_000_000, 2_000_000);
    let mut first_surface = RegionSurface::internal_with_metadata(first.clone(), 0.3, 3, -1.0, 7);
    first_surface.retag(RegionSurfaceKind::InternalSolid);
    let mut second_surface = RegionSurface::internal_with_metadata(second.clone(), 0.3, 5, -1.0, 9);
    second_surface.retag(RegionSurfaceKind::InternalSolid);
    let record = record_mut(&mut graph, LAYER);
    record.fill_surfaces = vec![
        first_surface,
        second_surface,
        surface(RegionSurfaceKind::Internal, covered, 0),
    ];
    record.fill_no_overlap_expolygons.clear();
    let before = graph_snapshot(&graph);

    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(grouped.surface_fills.len(), 2);
    assert_eq!(
        grouped.surface_fills[0].representative.kind,
        RegionSurfaceKind::InternalSolid
    );
    assert_eq!(grouped.surface_fills[0].representative.extra_perimeters, 7);
    assert_eq!(
        grouped.surface_fills[0].representative.thickness.to_bits(),
        0.3_f64.to_bits()
    );
    assert_eq!(grouped.surface_fills[0].representative.thickness_layers, 3);
    let mut expanded_bounds = grouped.surface_fills[0]
        .expolygons
        .iter()
        .map(bounds)
        .collect::<Vec<_>>();
    expanded_bounds.sort_unstable();
    assert_eq!(
        expanded_bounds,
        [
            (-10, -10, 4_000_010, 4_000_010),
            (5_999_990, -10, 10_000_010, 4_000_010),
        ]
    );
    assert_eq!(
        grouped.surface_fills[1].representative.kind,
        RegionSurfaceKind::Internal
    );
    assert!(grouped.surface_fills[1].expolygons.is_empty());

    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::InternalSolid, first, 0),
        surface(RegionSurfaceKind::Internal, outside_clipper_range(), 0),
    ];
    let before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();
    let error = match group_fills::group_fills(external(&graph), 0, LAYER) {
        Err(error) => error,
        Ok(_) => panic!("out-of-range fill grouping must fail atomically"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned()
        )
    );
    assert_snapshot_eq(graph_snapshot(&graph), before);
    assert_eq!(options(&graph, LAYER), &options_before);
    combine_infill::dispose(graph);
}

#[test]
fn simple_rotation_template_cycles_angles_and_marks_them_fixed() {
    let mut graph = graph();
    let shape = rectangle(0, 0, 4_000_000, 4_000_000);
    {
        let options = options_mut(&mut graph, LAYER);
        options.sparse_infill_rotate_template = OrcaString("0,90".to_owned());
        options.solid_infill_rotate_template = OrcaString("30,60".to_owned());
    }
    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::Internal, shape.clone(), 0)];
    let sparse = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        sparse.surface_fills[0].params.angle,
        std::f32::consts::FRAC_PI_2
    );
    assert!(sparse.surface_fills[0].params.fixed_angle);

    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, shape, 0)];
    let solid = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(
        solid.surface_fills[0].params.angle,
        std::f32::consts::FRAC_PI_3
    );
    assert!(solid.surface_fills[0].params.fixed_angle);
    combine_infill::dispose(graph);
}

#[test]
fn rotation_template_metalanguage_remains_explicitly_deferred() {
    let mut graph = graph();
    options_mut(&mut graph, LAYER).solid_infill_rotate_template = OrcaString("+30N2".to_owned());
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(0, 0, 4_000_000, 4_000_000),
        0,
    )];

    let error = match group_fills::group_fills(external(&graph), 0, LAYER) {
        Err(error) => error,
        Ok(_) => panic!("rotation-template metalanguage must remain gated"),
    };
    assert_eq!(
        error,
        SliceError::UnsupportedProjectFeature("solid_infill_rotate_template".to_owned())
    );
    combine_infill::dispose(graph);
}

fn assert_sorted_flow_sidecar(entries: &[LockFlowParam], expected: &[ExPolygon]) {
    assert_eq!(entries.len(), expected.len());
    assert!(
        entries
            .windows(2)
            .all(|pair| pair[0].flow.mm3_per_mm < pair[1].flow.mm3_per_mm)
    );
    assert!(entries.iter().all(|entry| entry.expolygons.len() == 1));
    assert_eq!(
        entries
            .iter()
            .map(|entry| &entry.expolygons[0])
            .collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
}

fn bounds(expolygon: &ExPolygon) -> (i64, i64, i64, i64) {
    expolygon.contour().points().iter().fold(
        (i64::MAX, i64::MAX, i64::MIN, i64::MIN),
        |(min_x, min_y, max_x, max_y), point| {
            (
                min_x.min(point.x()),
                min_y.min(point.y()),
                max_x.max(point.x()),
                max_y.max(point.y()),
            )
        },
    )
}
