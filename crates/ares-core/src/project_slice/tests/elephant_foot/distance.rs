use crate::geometry::{GridEdge, Point};

use super::{
    super::super::elephant_foot::distance::{
        filtered_closest_hits, filtered_contour_distances, inside_corner, left_of_segment,
        resample_polygon,
    },
    build_grid, resampled_contour, thresholds,
};

fn distance_bits(
    outer: &[(i64, i64)],
    holes: &[&[(i64, i64)]],
    contour_index: usize,
    compensation: f64,
    radius: f64,
) -> (
    Vec<u32>,
    Vec<Option<super::super::super::elephant_foot::distance::ClosestHit>>,
) {
    let grid = build_grid(outer, holes, radius);
    let (contour, parameters) = resampled_contour(&grid, contour_index);
    let thresholds = thresholds(compensation, radius);
    let distances =
        filtered_contour_distances(&grid, contour_index, &contour, &parameters, thresholds)
            .unwrap();
    let hits =
        filtered_closest_hits(&grid, contour_index, &contour, &parameters, thresholds).unwrap();
    (
        distances
            .iter()
            .map(|distance| distance.to_bits())
            .collect(),
        hits,
    )
}

#[test]
fn task22m_elephant_foot_rejects_near_same_contour_and_bulging_paths() {
    let near = [(0, 0), (5_000_000, 0), (5_000_000, 500_000), (0, 500_000)];
    let (distances, hits) = distance_bits(&near, &[], 0, 1_000_000.0, 2_000_000.0);
    assert_eq!(distances[1], 0x49f4_2400);
    assert!(hits[1].is_none());

    let bulge = [
        (0, 0),
        (10_000_000, 0),
        (10_000_000, 2_000_000),
        (0, 2_000_000),
    ];
    let (distances, hits) = distance_bits(&bulge, &[], 0, 50_000.0, 4_000_000.0);
    assert_eq!(distances[4], 0x4a74_2400);
    assert!(hits[4].is_none());
}

#[test]
fn task22m_elephant_foot_accepts_strictly_inward_interior_projection() {
    let outer = [(0, 0), (2_500_000, 0), (2_500_000, 500_000), (0, 500_000)];
    let (distances, hits) = distance_bits(&outer, &[], 0, 250_000.0, 1_000_000.0);
    assert_eq!(distances[7], 0x48f4_2400);
    assert_eq!(
        hits[7].unwrap().edge,
        GridEdge {
            contour_index: 0,
            segment_index: 0,
        }
    );

    let segment = [Point::new(0, 0), Point::new(10, 0)];
    assert!(left_of_segment(&segment, 0, Point::new(5, 1)));
    assert!(!left_of_segment(&segment, 0, Point::new(5, 0)));
    assert!(!left_of_segment(&segment, 0, Point::new(5, -1)));
}

#[test]
fn task22m_elephant_foot_rejects_clockwise_hole_behind_the_inward_direction() {
    let outer = [
        (0, 0),
        (3_750_000, 0),
        (3_750_000, 2_500_000),
        (1_750_000, 2_500_000),
        (1_750_000, 750_000),
        (750_000, 750_000),
        (750_000, 2_500_000),
        (0, 2_500_000),
    ];
    let hole = [
        (125_000, 1_000_000),
        (125_000, 1_500_000),
        (625_000, 1_500_000),
        (625_000, 1_000_000),
    ];
    let (distances, hits) = distance_bits(&outer, &[&hole], 0, 250_000.0, 1_500_000.0);
    assert_eq!(distances[23], 0x49b7_1b00);
    let hit = hits[23].unwrap();
    assert_eq!(hit.distance.to_bits(), 2_000_000.0f64.to_bits());
    assert_eq!(
        hit.edge,
        GridEdge {
            contour_index: 0,
            segment_index: 1,
        }
    );
    assert!(!hits[23].is_some_and(|hit| {
        hit.edge.contour_index == 1 && (hit.distance as f32).to_bits() == 0x498a_2c99
    }));
}

#[test]
fn task22m_elephant_foot_concave_corner_uses_or_while_convex_uses_and() {
    let outer = [
        (-1_500_000, -1_000_000),
        (750_000, -1_000_000),
        (500_000, 0),
        (1_500_000, 1_000_000),
        (-750_000, 1_750_000),
    ];
    let (distances, hits) = distance_bits(&outer, &[], 0, 25_000.0, 5_000_000.0);
    assert_eq!(distances[19], 0x49b6_15db);
    assert_eq!(hits[19].unwrap().edge.segment_index, 1);

    let convex = [Point::new(0, 10), Point::new(0, 0), Point::new(10, 0)];
    assert!(!inside_corner(&convex, 1, Point::new(5, -1)));
    let concave = [Point::new(10, 0), Point::new(0, 0), Point::new(0, 10)];
    assert!(inside_corner(&concave, 1, Point::new(1, -1)));
}

#[test]
fn task22m_elephant_foot_strict_search_epsilon_boundary_preserves_raw_hits() {
    let outer = [(0, 0), (2_500_000, 0), (2_500_000, 750_000), (0, 750_000)];
    for (radius, expected_bits, expected_hit) in [
        (749_900.0, 0x4937_14c0, true),
        (749_899.0, 0x4937_14b0, true),
        (749_901.0, 0x4937_14d0, false),
    ] {
        let (distances, hits) = distance_bits(&outer, &[], 0, 250_000.0, radius);
        assert_eq!(distances[8], expected_bits);
        assert_eq!(hits[8].is_some(), expected_hit);
        if let Some(hit) = hits[8] {
            assert_eq!(hit.distance.to_bits(), 750_000.0f64.to_bits());
        }
    }
}

#[test]
fn task22m_elephant_foot_clockwise_hole_freezes_distance_and_tie_order() {
    let outer = [
        (0, 0),
        (2_500_000, 0),
        (2_500_000, 2_500_000),
        (0, 2_500_000),
    ];
    let hole = [
        (500_000, 500_000),
        (500_000, 2_000_000),
        (2_000_000, 2_000_000),
        (2_000_000, 500_000),
    ];
    let (distances, _) = distance_bits(&outer, &[&hole], 1, 250_000.0, 1_000_000.0);
    assert_eq!(distances, vec![0x48f4_2400; 12]);
}

#[test]
fn task22m_elephant_foot_distance_vectors_subtract_before_f64_conversion() {
    let base = 1_i64 << 60;
    let contour = [Point::new(base, 0), Point::new(base + 101, 0)];
    assert!(left_of_segment(&contour, 0, Point::new(base, 101)));
}

#[test]
fn task22m_elephant_foot_resampling_casts_before_subtracting_coordinates() {
    let base = 1_i64 << 60;
    let (points, parameters) = resample_polygon(
        &[
            Point::new(base, 0),
            Point::new(base + 257, 0),
            Point::new(base + 257, 1),
        ],
        256.0,
    )
    .unwrap();
    assert_eq!(
        points,
        [
            Point::new(base, 0),
            Point::new(base, 0),
            Point::new(base + 257, 0),
            Point::new(base + 257, 1),
        ]
    );
    assert_eq!(
        parameters
            .iter()
            .map(|parameter| (
                parameter.source_index,
                parameter.interpolated,
                parameter.step_length.to_bits(),
                parameter.curve_parameter.to_bits(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, true, 0x4060_0007_fffe_0001, 0x4060_0007_fffe_0001),
            (0, false, 0x4060_0007_fffe_0001, 0x4070_0007_fffe_0001),
            (1, false, 0x4070_0000_0000_0000, 0x4080_0003_ffff_0000),
            (2, false, 0x3ff0_0000_0000_0000, 0x4080_0803_ffff_0000),
        ]
    );
}
