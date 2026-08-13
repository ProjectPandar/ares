use crate::{
    OrcaBool, ProcessInfillPattern,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        group_fills::{self, GroupedFills, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
    },
};

use super::focused::fixture::{
    assert_snapshot_eq, external, graph, graph_snapshot, object_mut, options, options_mut,
    record_mut, surface,
};

const LAYER: usize = 1;
const NON_LINE_PATTERNS: [ProcessInfillPattern; 24] = [
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

#[test]
fn task22o74_every_non_line_pattern_uses_the_same_graph_native_split() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        partial_core_shape(),
        0,
    )];
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();

    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Grid;
    let grid = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    assert_eq!(grid.surface_fills.len(), 2);
    let expected_normal = grid.surface_fills[0].expolygons.clone();
    let expected_narrow = grid.surface_fills[1].expolygons.clone();

    for pattern in NON_LINE_PATTERNS {
        options_mut(&mut graph, LAYER).internal_solid_infill_pattern = pattern;
        let graph_before = graph_snapshot(&graph);
        let options_before = options(&graph, LAYER).clone();

        let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

        assert_eq!(grouped.surface_fills.len(), 2, "{pattern:?}");
        assert_eq!(
            grouped.surface_fills[0].params.pattern,
            SurfaceFillPattern::Configured(pattern),
            "{pattern:?}"
        );
        assert_eq!(
            grouped.surface_fills[1].params.pattern,
            SurfaceFillPattern::ConcentricInternal,
            "{pattern:?}"
        );
        assert_eq!(
            grouped.surface_fills[0].expolygons, expected_normal,
            "{pattern:?}"
        );
        assert_eq!(
            grouped.surface_fills[1].expolygons, expected_narrow,
            "{pattern:?}"
        );
        assert_snapshot_eq(graph_snapshot(&graph), graph_before);
        assert_eq!(options(&graph, LAYER), &options_before, "{pattern:?}");
    }

    combine_infill::dispose(graph);
}

#[test]
fn task22o74_grid_preserves_exact_multi_expolygon_and_ordered_hole_topology() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Grid;
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::InternalSolid, leading_holed_strip(), 0),
        surface(
            RegionSurfaceKind::InternalSolid,
            input_rectangle(10_000_000, 0, 14_000_000, 4_000_000),
            0,
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            input_rectangle(20_000_000, 0, 20_200_000, 4_000_000),
            0,
        ),
    ];
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();
    let graph_before = graph_snapshot(&graph);
    let options_before = options(&graph, LAYER).clone();
    let object_before = object_mut(&mut graph).clone();

    let first = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();
    let repeated = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_exact_grid_topology(&first);
    assert_exact_grid_topology(&repeated);
    assert_ordered_repeat(&repeated, &first);
    assert_snapshot_eq(graph_snapshot(&graph), graph_before);
    assert_eq!(options(&graph, LAYER), &options_before);
    assert_eq!(object_mut(&mut graph).clone(), object_before);
    combine_infill::dispose(graph);
}

#[test]
fn task22o74_grid_opening_flattens_holes_before_classifying_a_thin_frame() {
    let mut graph = graph();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(true);
    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Grid;
    let frame = ExPolygon::new(
        polygon(&[
            (0, 0),
            (8_000_000, 0),
            (8_000_000, 8_000_000),
            (0, 8_000_000),
        ]),
        vec![polygon(&[
            (500_000, 500_000),
            (500_000, 7_500_000),
            (7_500_000, 7_500_000),
            (7_500_000, 500_000),
        ])],
    );
    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, frame.clone(), 0)];
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();
    let before = graph_snapshot(&graph);

    let grouped = group_fills::group_fills(external(&graph), 0, LAYER).unwrap();

    assert_eq!(grouped.surface_fills.len(), 1);
    assert_eq!(
        grouped.surface_fills[0].params.pattern,
        SurfaceFillPattern::ConcentricInternal
    );
    assert_eq!(grouped.surface_fills[0].expolygons, [frame]);
    assert_snapshot_eq(graph_snapshot(&graph), before);
    combine_infill::dispose(graph);
}

fn assert_exact_grid_topology(grouped: &GroupedFills) {
    assert_eq!(grouped.surface_fills.len(), 2);
    let normal = &grouped.surface_fills[0];
    let narrow = &grouped.surface_fills[1];
    assert_eq!(
        normal.params.pattern,
        SurfaceFillPattern::Configured(ProcessInfillPattern::Grid)
    );
    assert_eq!(
        normal.expolygons,
        [output_rectangle(9_999_990, -10, 14_000_010, 4_000_010)]
    );
    assert_eq!(
        narrow.params.pattern,
        SurfaceFillPattern::ConcentricInternal
    );
    assert_eq!(
        narrow.expolygons,
        [
            output_rectangle(19_999_990, -10, 20_200_010, 4_000_010),
            priority_holed_strip(),
        ]
    );
    assert_eq!(narrow.expolygons[1].holes().len(), 2);
    assert_eq!(narrow.params.idx, normal.params.idx);
    assert!(normal.no_overlap_expolygons.is_empty());
    assert!(narrow.no_overlap_expolygons.is_empty());
}

fn assert_ordered_repeat(actual: &GroupedFills, expected: &GroupedFills) {
    assert_eq!(actual.surface_fills.len(), expected.surface_fills.len());
    for (actual, expected) in actual.surface_fills.iter().zip(&expected.surface_fills) {
        assert_eq!(actual.params.pattern, expected.params.pattern);
        assert_eq!(actual.params.idx, expected.params.idx);
        assert_eq!(actual.expolygons, expected.expolygons);
        assert_eq!(actual.no_overlap_expolygons, expected.no_overlap_expolygons);
    }
}

fn partial_core_shape() -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (0, 0),
            (4_000_000, 0),
            (4_000_000, 1_900_000),
            (6_000_000, 1_900_000),
            (6_000_000, 0),
            (10_000_000, 0),
            (10_000_000, 4_000_000),
            (6_000_000, 4_000_000),
            (6_000_000, 2_100_000),
            (4_000_000, 2_100_000),
            (4_000_000, 4_000_000),
            (0, 4_000_000),
        ]),
        Vec::new(),
    )
}

fn leading_holed_strip() -> ExPolygon {
    ExPolygon::new(
        polygon(&[(0, 0), (200_000, 0), (200_000, 4_000_000), (0, 4_000_000)]),
        vec![
            polygon(&[
                (120_000, 1_000_000),
                (120_000, 3_000_000),
                (180_000, 3_000_000),
                (180_000, 1_000_000),
            ]),
            polygon(&[
                (20_000, 1_000_000),
                (20_000, 3_000_000),
                (80_000, 3_000_000),
                (80_000, 1_000_000),
            ]),
        ],
    )
}

fn priority_holed_strip() -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (200_010, 4_000_010),
            (-10, 4_000_010),
            (-10, -10),
            (200_010, -10),
        ]),
        vec![
            polygon(&[
                (20_010, 1_000_010),
                (20_010, 2_999_990),
                (79_990, 2_999_990),
                (79_990, 1_000_010),
            ]),
            polygon(&[
                (120_010, 1_000_010),
                (120_010, 2_999_990),
                (179_990, 2_999_990),
                (179_990, 1_000_010),
            ]),
        ],
    )
}

fn input_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn output_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (max_x, max_y),
            (min_x, max_y),
            (min_x, min_y),
            (max_x, min_y),
        ]),
        Vec::new(),
    )
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
