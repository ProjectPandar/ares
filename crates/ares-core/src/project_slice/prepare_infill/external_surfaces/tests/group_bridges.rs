use super::{get_grouped_bridges, group_id};
use crate::geometry::{ClipperError, ExPolygon, Polygon, RegionExpansionEx};

use super::super::Bridge;
use super::helpers::{ExPolygonSnapshot, expolygon, snapshots, square};

type BridgeSnapshot = (ExPolygonSnapshot, u32, Option<f64>);

fn expansion(expolygon: ExPolygon, src_id: u32, boundary_id: u32) -> RegionExpansionEx {
    RegionExpansionEx {
        expolygon,
        src_id,
        boundary_id,
    }
}

fn bridge_snapshots(bridges: &[Bridge]) -> Vec<BridgeSnapshot> {
    bridges
        .iter()
        .map(|bridge| {
            (
                snapshots(std::slice::from_ref(&bridge.expolygon)).remove(0),
                bridge.group_id,
                bridge.angle,
            )
        })
        .collect()
}

fn groups(bridges: &[Bridge]) -> Vec<u32> {
    bridges.iter().map(|bridge| bridge.group_id).collect()
}

fn roots(bridges: &mut [Bridge]) -> Vec<u32> {
    (0..bridges.len())
        .map(|src_id| group_id(bridges, src_id as u32))
        .collect()
}

fn invalid_overlap() -> ExPolygon {
    expolygon(&[(0, 0), (i64::MAX, 0), (0, 10)], Vec::new())
}

#[test]
fn task22o37_initializes_empty_and_preserves_source_geometry() {
    assert!(get_grouped_bridges(Vec::new(), &[]).unwrap().is_empty());

    let source_with_hole = expolygon(
        &[(10, 10), (20, 10), (20, 20), (10, 20)],
        vec![super::helpers::polygon(&[
            (12, 12),
            (12, 18),
            (18, 18),
            (18, 12),
        ])],
    );
    let sources = vec![square(0, 4), source_with_hole];

    let bridges = get_grouped_bridges(sources, &[]).unwrap();

    assert_eq!(
        bridge_snapshots(&bridges),
        vec![
            (((vec![(0, 0), (4, 0), (4, 4), (0, 4)]), vec![]), 0, None),
            (
                (
                    vec![(10, 10), (20, 10), (20, 20), (10, 20)],
                    vec![vec![(12, 12), (12, 18), (18, 18), (18, 12)]],
                ),
                1,
                None,
            ),
        ]
    );
}

#[test]
fn task22o37_group_id_follows_parent_chain_without_full_compression() {
    let mut bridges = get_grouped_bridges(
        vec![square(0, 1), square(2, 3), square(4, 5), square(6, 7)],
        &[],
    )
    .unwrap();
    bridges[1].group_id = 0;
    bridges[2].group_id = 1;
    bridges[3].group_id = 2;

    assert_eq!(group_id(&mut bridges, 3), 0);
    assert_eq!(groups(&bridges), vec![0, 0, 1, 2]);
}

#[test]
fn task22o37_matches_pinned_multiple_boundary_oracle_in_source_order() {
    let sources = vec![
        square(100, 104),
        square(110, 114),
        square(120, 124),
        square(130, 134),
    ];
    let expansions = vec![
        expansion(square(0, 10), 0, 0),
        expansion(square(5, 15), 1, 0),
        expansion(square(30, 40), 2, 0),
        expansion(square(0, 10), 3, 1),
        expansion(square(5, 15), 2, 1),
    ];

    let bridges = get_grouped_bridges(sources, &expansions).unwrap();

    assert_eq!(
        bridge_snapshots(&bridges),
        vec![
            (
                (vec![(100, 100), (104, 100), (104, 104), (100, 104)], vec![]),
                0,
                None
            ),
            (
                (vec![(110, 110), (114, 110), (114, 114), (110, 114)], vec![]),
                0,
                None
            ),
            (
                (vec![(120, 120), (124, 120), (124, 124), (120, 124)], vec![]),
                2,
                None
            ),
            (
                (vec![(130, 130), (134, 130), (134, 134), (130, 134)], vec![]),
                2,
                None
            ),
        ]
    );
}

#[test]
fn task22o37_preserves_raw_parent_forest_and_does_not_regroup_windows() {
    let expansions = vec![
        expansion(square(0, 20), 2, 8),
        expansion(square(1, 19), 1, 8),
        expansion(square(2, 18), 0, 8),
    ];
    let mut bridges = get_grouped_bridges(
        vec![square(300, 304), square(310, 314), square(320, 324)],
        &expansions,
    )
    .unwrap();
    assert_eq!(groups(&bridges), vec![0, 0, 1]);
    assert_eq!(roots(&mut bridges), vec![0, 0, 0]);
    assert_eq!(groups(&bridges), vec![0, 0, 1]);

    let separated = vec![
        expansion(square(0, 20), 0, 9),
        expansion(square(50, 60), 0, 10),
        expansion(square(1, 19), 1, 9),
    ];
    let bridges =
        get_grouped_bridges(vec![square(400, 404), square(410, 414)], &separated).unwrap();
    assert_eq!(groups(&bridges), vec![0, 1]);
}

#[test]
fn task22o37_same_source_disjoint_and_separate_windows_stay_roots() {
    let expansions = vec![
        expansion(square(0, 10), 0, 0),
        expansion(square(5, 15), 0, 0),
        expansion(square(30, 40), 1, 0),
        expansion(square(30, 40), 1, 1),
        expansion(square(35, 45), 2, 2),
    ];
    let bridges =
        get_grouped_bridges(vec![square(0, 1), square(2, 3), square(4, 5)], &expansions).unwrap();
    assert_eq!(groups(&bridges), vec![0, 1, 2]);
}

#[test]
fn task22o37_uses_expansion_contours_and_ignores_holes() {
    let expansion_with_hole = expolygon(
        &[(0, 0), (20, 0), (20, 20), (0, 20)],
        vec![super::helpers::polygon(&[
            (5, 5),
            (5, 15),
            (15, 15),
            (15, 5),
        ])],
    );
    let source_with_hole = expolygon(
        &[(200, 200), (220, 200), (220, 220), (200, 220)],
        vec![super::helpers::polygon(&[
            (204, 204),
            (204, 216),
            (216, 216),
            (216, 204),
        ])],
    );
    let expansions = vec![
        expansion(expansion_with_hole, 0, 4),
        expansion(square(6, 14), 1, 4),
    ];
    let bridges =
        get_grouped_bridges(vec![source_with_hole, square(230, 234)], &expansions).unwrap();

    assert_eq!(groups(&bridges), vec![0, 0]);
    assert_eq!(bridges[0].expolygon.holes().len(), 1);
    assert!(bridges[1].expolygon.holes().is_empty());
}

#[test]
fn task22o37_propagates_first_and_later_coordinate_failures_without_input_mutation() {
    let first = vec![
        expansion(invalid_overlap(), 0, 0),
        expansion(square(0, 10), 1, 0),
    ];
    let first_before = first.clone();
    assert!(matches!(
        get_grouped_bridges(vec![square(0, 1), square(2, 3)], &first),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(first, first_before);

    let later = vec![
        expansion(square(0, 10), 0, 0),
        expansion(square(5, 15), 1, 0),
        expansion(invalid_overlap(), 2, 0),
    ];
    let later_before = later.clone();
    assert!(matches!(
        get_grouped_bridges(vec![square(0, 1), square(2, 3), square(4, 5)], &later),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert_eq!(later, later_before);
}

#[test]
fn task22o37_invalid_paths_short_circuit_for_equal_source_and_disjoint_bbox() {
    let equal_source = vec![
        expansion(invalid_overlap(), 0, 0),
        expansion(square(0, 10), 0, 0),
    ];
    let bridges = get_grouped_bridges(vec![square(0, 1)], &equal_source).unwrap();
    assert_eq!(groups(&bridges), vec![0]);

    let disjoint = vec![
        expansion(square(0, 10), 0, 0),
        expansion(square(i64::MAX - 20, i64::MAX - 10), 1, 0),
    ];
    let bridges = get_grouped_bridges(vec![square(0, 1), square(2, 3)], &disjoint).unwrap();
    assert_eq!(groups(&bridges), vec![0, 1]);
}

#[test]
#[should_panic]
fn task22o37_trusts_source_ids_when_overlap_reaches_root_resolution() {
    let expansions = vec![
        expansion(square(0, 10), 0, 0),
        expansion(square(5, 15), 2, 0),
    ];
    let _ = get_grouped_bridges(vec![square(0, 1), square(2, 3)], &expansions);
}

#[test]
#[should_panic]
fn task22o37_trusts_nonempty_expansion_contours() {
    let empty = ExPolygon::new(Polygon::new(Vec::new()), Vec::new());
    let expansions = vec![expansion(empty, 0, 0)];
    let _ = get_grouped_bridges(vec![square(0, 1)], &expansions);
}
