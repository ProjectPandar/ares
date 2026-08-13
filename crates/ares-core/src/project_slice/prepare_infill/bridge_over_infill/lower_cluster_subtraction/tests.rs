use super::{ClusterBridgeHistoryLayer, subtract_filled_lower_cluster_bridges};
use crate::{
    geometry::{ClipperError, Point, Polygon, difference_polygons_paths},
    project_slice::prepare_infill::bridge_over_infill::types::{CandidateSource, CandidateSurface},
};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

#[test]
fn task22o56_unconditionally_executes_difference_for_empty_inputs_and_clips() {
    assert!(subtract(&[], &[], 1.0, 0.2).unwrap().is_empty());
    let deep = [rectangle(0, 0, 100, 100)];
    assert_eq!(
        snapshot(&subtract(&deep, &[], 1.0, 0.2).unwrap()),
        vec![vec![(100, 100), (0, 100), (0, 0), (100, 0)]]
    );
    assert_eq!(
        subtract(&[outside_range()], &[], 1.0, 0.2),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let no_polygons = [candidate(0, Vec::new())];
    assert_eq!(
        snapshot(&subtract(&deep, &[view(0.9, &no_polygons)], 1.0, 0.2).unwrap()),
        vec![vec![(100, 100), (0, 100), (0, 0), (100, 0)]]
    );

    let invalid = [candidate(0, vec![outside_range()])];
    let history = [view(0.9, &invalid)];
    assert_eq!(
        subtract(&[], &history, 1.0, 0.2),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o56_includes_exact_bottom_z_and_above_but_excludes_one_ulp_below() {
    let deep = [rectangle(0, 0, 100, 100)];
    let target = f64::from_bits(0x3fc7_0a3d_6000_0000);
    let bottom = (1.0 - target) - 1.0e-4_f64;
    let clip = [candidate(0, vec![rectangle(0, 0, 10, 100)])];

    for print_z in [bottom, f64::from_bits(bottom.to_bits() + 1)] {
        let history = [view(print_z, &clip)];
        assert_eq!(
            total_area(&subtract(&deep, &history, 1.0, target).unwrap()),
            9_000.0
        );
    }
    let history = [view(f64::from_bits(bottom.to_bits() - 1), &clip)];
    assert_eq!(
        total_area(&subtract(&deep, &history, 1.0, target).unwrap()),
        10_000.0
    );
}

#[test]
fn task22o56_preserves_promoted_f32_product_and_epsilon_subtraction_order() {
    let deep = [rectangle(0, 0, 100, 100)];
    let target = f64::from_bits(0x4000_05e7_c000_0000);
    let current_print_z = 2.251_340_223_295_654_3;
    let source_bottom = (current_print_z - target) - 1.0e-4_f64;
    let reassociated_bottom = current_print_z - (target + 1.0e-4_f64);
    assert!(source_bottom > reassociated_bottom);
    let clip = [candidate(0, vec![rectangle(0, 0, 10, 100)])];
    let history = [view(0.248_356_788_999_999_9, &clip)];

    assert_eq!(
        total_area(&subtract(&deep, &history, current_print_z, target).unwrap()),
        10_000.0
    );
}

#[test]
fn task22o56_reverse_walk_breaks_at_first_below_layer() {
    let deep = [rectangle(0, 0, 100, 100)];
    let oldest = [candidate(0, vec![rectangle(0, 0, 20, 100)])];
    let below = [candidate(1, vec![rectangle(20, 0, 40, 100)])];
    let newest = [candidate(2, vec![rectangle(40, 0, 60, 100)])];
    let history = [view(0.60, &oldest), view(0.70, &below), view(0.90, &newest)];

    let output = subtract(&deep, &history, 1.0, 0.2001).unwrap();

    assert_eq!(total_area(&output), 8_000.0);
    assert_eq!(
        sorted_bounds(&output),
        vec![(0, 0, 40, 100), (60, 0, 100, 100)]
    );
}

#[test]
fn task22o56_flattens_reverse_layers_candidates_and_polygons_in_source_order() {
    let deep = [rectangle(0, 0, 100, 100)];
    let older = [
        candidate(0, vec![rectangle(40, 0, 60, 100)]),
        candidate(1, vec![rectangle(0, 40, 40, 60)]),
    ];
    let newer = [candidate(
        2,
        vec![rectangle(60, 40, 100, 60), rectangle(10, 10, 20, 20)],
    )];
    let history = [view(0.81, &older), view(0.90, &newer)];

    let output = subtract(&deep, &history, 1.0, 0.2).unwrap();
    let expected_clips = newer
        .iter()
        .chain(&older)
        .flat_map(|candidate| candidate.new_polygons.iter().cloned())
        .collect::<Vec<_>>();
    let expected = difference_polygons_paths(&deep, &expected_clips).unwrap();

    assert_eq!(snapshot(&output), snapshot(&expected));
    assert_eq!(output.len(), 5);
    assert_eq!(total_area(&output), 6_300.0);
    assert_eq!(output.iter().filter(|path| path.area() > 0.0).count(), 4);
    assert_eq!(output.iter().filter(|path| path.area() < 0.0).count(), 1);
    assert_eq!(
        sorted_bounds(&output),
        vec![
            (0, 0, 40, 40),
            (0, 60, 40, 100),
            (10, 10, 20, 20),
            (60, 0, 100, 40),
            (60, 60, 100, 100),
        ]
    );
}

#[test]
fn task22o56_preserves_flat_hole_and_component_topology() {
    let deep = [rectangle(0, 0, 100, 100), rectangle(120, 0, 160, 40)];
    let clips = [candidate(
        0,
        vec![rectangle(20, 20, 80, 80), rectangle(130, 0, 150, 40)],
    )];
    let history = [view(0.9, &clips)];

    let output = subtract(&deep, &history, 1.0, 0.2).unwrap();

    assert_eq!(
        snapshot(&output),
        vec![
            vec![(100, 100), (0, 100), (0, 0), (100, 0)],
            vec![(20, 20), (20, 80), (80, 80), (80, 20)],
            vec![(150, 0), (160, 0), (160, 40), (150, 40)],
            vec![(120, 40), (120, 0), (130, 0), (130, 40)],
        ]
    );
}

#[test]
fn task22o56_range_error_is_atomic_and_borrowed_allocations_are_unchanged() {
    let deep = vec![rectangle(0, 0, 100, 100)];
    let candidates = vec![candidate(0, vec![outside_range()])];
    let history = [view(0.9, &candidates)];
    let before = input_snapshot(&deep, &candidates, &[0.9]);

    assert_eq!(
        subtract(&deep, &history, 1.0, 0.2),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(input_snapshot(&deep, &candidates, &[0.9]), before);
}

#[test]
fn task22o56_independent_calls_are_repeatable_and_leave_inputs_unchanged() {
    let deep = vec![rectangle(0, 0, 100, 100)];
    let candidates = vec![candidate(0, vec![rectangle(25, 25, 75, 75)])];
    let history = [view(0.9, &candidates)];
    let before = input_snapshot(&deep, &candidates, &[0.9]);

    let first = subtract(&deep, &history, 1.0, 0.2).unwrap();
    let second = subtract(&deep, &history, 1.0, 0.2).unwrap();

    assert_eq!(snapshot(&first), snapshot(&second));
    assert_eq!(input_snapshot(&deep, &candidates, &[0.9]), before);
}

fn subtract(
    deep: &[Polygon],
    history: &[ClusterBridgeHistoryLayer<'_>],
    current_print_z: f64,
    target_flow_height: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    subtract_filled_lower_cluster_bridges(deep, history, current_print_z, target_flow_height)
}

fn view<'a>(print_z: f64, candidates: &'a [CandidateSurface]) -> ClusterBridgeHistoryLayer<'a> {
    ClusterBridgeHistoryLayer {
        print_z,
        candidates,
    }
}

fn candidate(id: usize, new_polygons: Vec<Polygon>) -> CandidateSurface {
    CandidateSurface {
        source: CandidateSource {
            layer_index: 1,
            region_index: 0,
            surface_index: id,
        },
        new_polygons,
        bridge_angle: id as f64,
    }
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
        Point::new(min_x, max_y),
    ])
}

fn outside_range() -> Polygon {
    rectangle(HI_RANGE + 1, 0, HI_RANGE + 11, 10)
}

fn total_area(polygons: &[Polygon]) -> f64 {
    polygons.iter().map(Polygon::area).sum::<f64>().abs()
}

fn sorted_bounds(polygons: &[Polygon]) -> Vec<(i64, i64, i64, i64)> {
    let mut bounds = polygons
        .iter()
        .map(|polygon| {
            polygon.points().iter().fold(
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
        })
        .collect::<Vec<_>>();
    bounds.sort_unstable();
    bounds
}

#[derive(Debug, Eq, PartialEq)]
struct InputSnapshot {
    history_z_bits: Vec<u64>,
    deep_allocation: (usize, usize, usize),
    deep: Vec<PolygonSnapshot>,
    candidate_allocation: (usize, usize, usize),
    candidates: Vec<CandidateSnapshot>,
}

#[derive(Debug, Eq, PartialEq)]
struct CandidateSnapshot {
    source: CandidateSource,
    angle_bits: u64,
    polygon_allocation: (usize, usize, usize),
    polygons: Vec<PolygonSnapshot>,
}

#[derive(Debug, Eq, PartialEq)]
struct PolygonSnapshot {
    allocation: (usize, usize),
    points: Vec<(i64, i64)>,
}

fn input_snapshot(
    deep: &Vec<Polygon>,
    candidates: &Vec<CandidateSurface>,
    history_z: &[f64],
) -> InputSnapshot {
    InputSnapshot {
        history_z_bits: history_z.iter().map(|value| value.to_bits()).collect(),
        deep_allocation: (deep.as_ptr() as usize, deep.len(), deep.capacity()),
        deep: deep.iter().map(polygon_snapshot).collect(),
        candidate_allocation: (
            candidates.as_ptr() as usize,
            candidates.len(),
            candidates.capacity(),
        ),
        candidates: candidates
            .iter()
            .map(|candidate| CandidateSnapshot {
                source: candidate.source,
                angle_bits: candidate.bridge_angle.to_bits(),
                polygon_allocation: (
                    candidate.new_polygons.as_ptr() as usize,
                    candidate.new_polygons.len(),
                    candidate.new_polygons.capacity(),
                ),
                polygons: candidate
                    .new_polygons
                    .iter()
                    .map(polygon_snapshot)
                    .collect(),
            })
            .collect(),
    }
}

fn polygon_snapshot(polygon: &Polygon) -> PolygonSnapshot {
    PolygonSnapshot {
        allocation: (polygon.points().as_ptr() as usize, polygon.points().len()),
        points: polygon
            .points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect(),
    }
}

#[test]
fn task22o56_candidate_and_polygon_order_freeze_flat_path_order() {
    let deep = [rectangle(0, 0, 100, 100)];
    let first = rectangle(0, 10, 25, 35);
    let second = rectangle(40, 10, 65, 35);
    let expected = vec![
        vec![(40, 10), (40, 35), (65, 35), (65, 10)],
        vec![
            (100, 0),
            (100, 100),
            (0, 100),
            (0, 35),
            (25, 35),
            (25, 10),
            (0, 10),
            (0, 0),
        ],
    ];

    let candidates = [
        candidate(0, vec![first.clone()]),
        candidate(1, vec![second.clone()]),
    ];
    assert_eq!(
        snapshot(&subtract(&deep, &[view(0.9, &candidates)], 1.0, 0.2).unwrap()),
        expected
    );
    let polygons = [candidate(0, vec![first, second])];
    assert_eq!(
        snapshot(&subtract(&deep, &[view(0.9, &polygons)], 1.0, 0.2).unwrap()),
        expected
    );
}

#[test]
fn task22o56_one_call_flattening_differs_from_repeated_per_layer_difference() {
    let deep = [rectangle(0, 0, 100, 100)];
    let clip = [candidate(0, vec![rectangle(0, 0, 25, 25)])];
    let history = [view(0.8, &clip), view(0.9, &clip)];
    assert_eq!(
        snapshot(&subtract(&deep, &history, 1.0, 0.3).unwrap()),
        vec![vec![
            (25, 0),
            (100, 0),
            (100, 100),
            (0, 100),
            (0, 25),
            (25, 25),
        ]]
    );
}

fn snapshot(polygons: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    polygons
        .iter()
        .map(|polygon| {
            polygon
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}
