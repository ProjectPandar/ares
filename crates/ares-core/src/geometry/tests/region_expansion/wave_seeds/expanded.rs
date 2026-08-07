use super::*;
use crate::geometry::ClipperError;
use crate::geometry::region_expansion::expanded_source_paths_for_test;

#[test]
fn positive_precondition_precedes_empty_shortcuts() {
    for expansion in [0.0, -1.0, f32::NAN] {
        assert!(
            std::panic::catch_unwind(|| {
                wave_seeds(&[], &[], expansion, false, CoordinateScale::Normal)
            })
            .is_err()
        );
    }
}

#[test]
fn either_empty_side_short_circuits() {
    assert_eq!(
        wave_seeds(&[], &[square(0, 100)], 1.0, false, CoordinateScale::Normal).unwrap(),
        Vec::new()
    );
    assert_eq!(
        wave_seeds(&[square(20, 30)], &[], 1.0, false, CoordinateScale::Normal).unwrap(),
        Vec::new()
    );
}

#[test]
fn boundary_range_is_validated_before_source_expansion() {
    let outside = 0x4000_0000_0000_0000_i64;
    let invalid_boundary = expolygon(
        &[(outside, 0), (outside, 10), (outside - 1, 10)],
        Vec::new(),
    );
    assert_eq!(
        wave_seeds(
            &[square(20, 30)],
            &[invalid_boundary],
            1.0,
            false,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn discovered_sorted_seed_feeds_unchanged_propagation() {
    let boundary = vec![square(0, 100)];
    let seeds = wave_seeds(
        &[square(20, 30)],
        &boundary,
        1.0,
        true,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!((seeds[0].src, seeds[0].boundary), (0, 0));
    let result =
        propagate_waves(&seeds, &boundary, &params((2.0, 2.0, 0, 4.0, 0.25, 0.0))).unwrap();
    let ordered = result
        .iter()
        .map(|item| {
            (
                item.src_id,
                item.boundary_id,
                item.polygon
                    .points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        vec![
            (
                0,
                0,
                vec![
                    (33, 20),
                    (33, 30),
                    (30, 33),
                    (20, 33),
                    (17, 30),
                    (17, 20),
                    (20, 17),
                    (30, 17),
                ],
            ),
            (0, 0, vec![(21, 21), (21, 29), (29, 29), (29, 21)]),
        ]
    );
}

#[test]
fn expanded_paths_use_outer_positive_hole_negative_and_shared_source_id() {
    let source = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![polygon(&[(40, 40), (60, 40), (60, 60), (40, 60)])],
    );
    let (paths, end) = expanded_source_paths_for_test(&[source], 5.0, 9).unwrap();
    assert_eq!(end, 10);
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().all(|path| {
        path.first().unwrap().full_eq(*path.last().unwrap())
            && path.iter().all(|point| point.z == 9)
    }));
    let outer_x = paths[0].iter().map(|point| point.x()).min().unwrap();
    let hole_x = paths[1].iter().map(|point| point.x()).min().unwrap();
    assert!(outer_x < 0);
    assert!(hole_x > 40);
}

#[test]
fn each_expolygon_increments_once_and_keeps_exact_xyz_endpoint_order() {
    let (paths, end) =
        expanded_source_paths_for_test(&[square(0, 10), square(30, 40)], 1.0, 4).unwrap();
    assert_eq!(end, 6);
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].first().unwrap().z, 4);
    assert_eq!(paths[1].first().unwrap().z, 5);
    for path in paths {
        assert!(path.first().unwrap().full_eq(*path.last().unwrap()));
        assert_eq!(
            path.iter().filter(|point| point.z == path[0].z).count(),
            path.len()
        );
    }
}

#[test]
fn empty_expansion_has_zero_paths_and_does_not_advance_source_id() {
    let (paths, end) = expanded_source_paths_for_test(&[], 1.0, 17).unwrap();
    assert!(paths.is_empty());
    assert_eq!(end, 17);
}

#[test]
fn degenerate_nonempty_expolygon_emits_zero_paths_but_advances_its_id() {
    let source = expolygon(&[(0, 0), (1, 0)], Vec::new());
    let (paths, end) = expanded_source_paths_for_test(&[source], 1.0, 17).unwrap();
    assert!(paths.is_empty());
    assert_eq!(end, 18);
}

#[test]
fn boundary_failure_precedes_out_of_range_source_geometry() {
    let outside = 0x4000_0000_0000_0000_i64;
    let invalid = expolygon(
        &[(outside, 0), (outside, 10), (outside - 1, 10)],
        Vec::new(),
    );
    assert_eq!(
        wave_seeds(
            std::slice::from_ref(&invalid),
            std::slice::from_ref(&invalid),
            1.0,
            false,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn source_expansion_shortest_edge_threshold_is_strict() {
    let source = expolygon(&[(0, 0), (2, 0), (3, 0), (3, 10), (0, 10)], Vec::new());
    let (paths, _) = expanded_source_paths_for_test(&[source], 400.0, 1).unwrap();
    assert_eq!(
        paths[0]
            .iter()
            .map(|point| (point.x(), point.y(), point.z))
            .collect::<Vec<_>>(),
        vec![
            (385, -193, 1),
            (419, 147, 1),
            (180, 410, 1),
            (-166, 410, 1),
            (-400, 176, 1),
            (-400, -166, 1),
            (-166, -400, 1),
            (156, -400, 1),
            (385, -193, 1),
        ]
    );
}

#[test]
fn one_concave_hole_can_expand_into_multiple_open_source_paths() {
    let dumbbell = polygon(&[
        (20, 20),
        (40, 20),
        (40, 29),
        (60, 29),
        (60, 20),
        (80, 20),
        (80, 40),
        (60, 40),
        (60, 31),
        (40, 31),
        (40, 40),
        (20, 40),
    ]);
    let source = expolygon(&[(0, 0), (100, 0), (100, 100), (0, 100)], vec![dumbbell]);
    let (paths, end) = expanded_source_paths_for_test(&[source], 2.0, 6).unwrap();
    assert_eq!(end, 7);
    assert!(paths.len() >= 3);
    assert!(
        paths
            .iter()
            .all(|path| path.iter().all(|point| point.z == 6))
    );
}
