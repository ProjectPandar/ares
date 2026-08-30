use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, ProcessInfillPattern, SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        group_fills, prepare_infill::combine_infill, region_slices::RegionSurfaceKind,
    },
};

use super::{
    focused::fixture::{
        assert_snapshot_eq, external, graph, graph_snapshot, object_mut, options_mut,
        planned_layer_mut, record_mut, surface,
    },
    oracle::authoritative_geometry,
};

const FIRST: usize = 1;
const SECOND: usize = 2;
const THIRD: usize = 3;

#[test]
fn task22o74_all_line_patterns_and_layer_alternation_cross_the_full_seam() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    let shape = asymmetric_neck();
    record_mut(&mut graph, FIRST).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, shape.clone(), 0)];
    record_mut(&mut graph, FIRST)
        .fill_no_overlap_expolygons
        .clear();
    record_mut(&mut graph, SECOND).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, shape, 0)];
    record_mut(&mut graph, SECOND)
        .fill_no_overlap_expolygons
        .clear();
    let before = graph_snapshot(&graph);

    let mut geometry = Vec::new();
    for pattern in [
        ProcessInfillPattern::Rectilinear,
        ProcessInfillPattern::Monotonic,
        ProcessInfillPattern::MonotonicLine,
    ] {
        options_mut(&mut graph, FIRST).internal_solid_infill_pattern = pattern;
        options_mut(&mut graph, SECOND).internal_solid_infill_pattern = pattern;
        let first = group_fills::group_fills(external(&graph), 0, FIRST).unwrap();
        let second = group_fills::group_fills(external(&graph), 0, SECOND).unwrap();
        geometry.push((
            authoritative_geometry(&first),
            authoritative_geometry(&second),
        ));
    }
    assert!(geometry.iter().all(|pair| pair == &geometry[0]));
    assert_ne!(geometry[0].0, geometry[0].1);

    options_mut(&mut graph, FIRST).internal_solid_infill_pattern =
        ProcessInfillPattern::AlignedRectilinear;
    options_mut(&mut graph, SECOND).internal_solid_infill_pattern =
        ProcessInfillPattern::AlignedRectilinear;
    let aligned_first = group_fills::group_fills(external(&graph), 0, FIRST).unwrap();
    let aligned_second = group_fills::group_fills(external(&graph), 0, SECOND).unwrap();
    let aligned_first = authoritative_geometry(&aligned_first);
    let aligned_second = authoritative_geometry(&aligned_second);
    assert_eq!(aligned_first, aligned_second);
    assert_eq!(aligned_first, geometry[0].1);
    assert_eq!(
        options_mut(&mut graph, FIRST).internal_solid_infill_pattern,
        ProcessInfillPattern::AlignedRectilinear
    );
    assert_eq!(
        options_mut(&mut graph, SECOND).internal_solid_infill_pattern,
        ProcessInfillPattern::AlignedRectilinear
    );
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_line_alternation_divides_stored_layer_id_by_surface_thickness_layers() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    let mut rich = crate::project_slice::region_slices::RegionSurface::internal_with_metadata(
        asymmetric_neck(),
        0.4,
        2,
        -1.0,
        0,
    );
    rich.retag(RegionSurfaceKind::InternalSolid);
    for (slot, id) in [(FIRST, 2), (SECOND, 3), (THIRD, 4)] {
        record_mut(&mut graph, slot).fill_surfaces = vec![rich.clone()];
        record_mut(&mut graph, slot)
            .fill_no_overlap_expolygons
            .clear();
        planned_layer_mut(&mut graph, slot).id = id;
        options_mut(&mut graph, slot).internal_solid_infill_pattern =
            ProcessInfillPattern::Monotonic;
    }
    let before = graph_snapshot(&graph);

    let geometry = [FIRST, SECOND, THIRD].map(|slot| {
        let grouped = group_fills::group_fills(external(&graph), 0, slot).unwrap();
        authoritative_geometry(&grouped)
    });

    assert_eq!(geometry[0], geometry[1]);
    assert_ne!(geometry[1], geometry[2]);
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_exact_four_millimeter_sections_remain_long() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    {
        let options = options_mut(&mut graph, FIRST);
        options.internal_solid_infill_pattern = ProcessInfillPattern::AlignedRectilinear;
        options.internal_solid_infill_line_width = FloatOrPercent::Float(0.42);
        options.solid_infill_direction = OrcaFloat(-90.0);
    }
    let rectangle = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(3_770_790, 0),
            Point::new(3_770_790, 4_377_079),
            Point::new(0, 4_377_079),
        ]),
        Vec::new(),
    );
    record_mut(&mut graph, FIRST).fill_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        rectangle.clone(),
        0,
    )];
    record_mut(&mut graph, FIRST)
        .fill_no_overlap_expolygons
        .clear();

    let grouped = group_fills::group_fills(external(&graph), 0, FIRST).unwrap();

    assert_eq!(grouped.surface_fills.len(), 1);
    assert_eq!(
        grouped.surface_fills[0].params.pattern,
        crate::project_slice::group_fills::SurfaceFillPattern::Configured(
            ProcessInfillPattern::AlignedRectilinear
        )
    );
    assert_eq!(grouped.surface_fills[0].expolygons, [rectangle]);
    assert!(grouped.surface_fills[0].params.fixed_angle);
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_narrow_rotation_range_failure_is_atomic_after_base_grouping() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    let origin = 3_400_000_000_000_000_000_i64;
    let shape = ExPolygon::new(
        Polygon::new(vec![
            Point::new(origin, -origin),
            Point::new(origin + 1_000_000, -origin),
            Point::new(origin + 1_000_000, -origin + 1_000_000),
            Point::new(origin, -origin + 1_000_000),
        ]),
        Vec::new(),
    );
    record_mut(&mut graph, FIRST).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, shape, 0)];
    let before = graph_snapshot(&graph);

    let error = match group_fills::group_fills(external(&graph), 0, FIRST) {
        Err(error) => error,
        Ok(_) => panic!("narrow rotation outside the Clipper range must fail"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned()
        )
    );
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

fn asymmetric_neck() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(5_000_000, 0),
            Point::new(5_000_000, 1_700_000),
            Point::new(8_000_000, 1_700_000),
            Point::new(8_000_000, 500_000),
            Point::new(12_000_000, 500_000),
            Point::new(12_000_000, 5_000_000),
            Point::new(8_000_000, 5_000_000),
            Point::new(8_000_000, 2_100_000),
            Point::new(5_000_000, 2_100_000),
            Point::new(5_000_000, 4_000_000),
            Point::new(0, 4_000_000),
        ]),
        Vec::new(),
    )
}
