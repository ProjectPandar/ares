use super::helpers::{expolygon, params, polygon};
use crate::geometry::{ClipperError, ExPolygon, RegionExpansionEx, WaveSeed, propagate_waves_ex};

type ExSnapshot = (u32, u32, Vec<(i64, i64)>, Vec<Vec<(i64, i64)>>);

fn seed(src: u32, boundary: u32, points: &[(i64, i64)]) -> WaveSeed {
    WaveSeed {
        src,
        boundary,
        path: polygon(points),
    }
}

fn square_boundary() -> ExPolygon {
    expolygon(&[(0, 0), (1000, 0), (1000, 1000), (0, 1000)], vec![])
}

fn snapshots(expansions: &[RegionExpansionEx]) -> Vec<ExSnapshot> {
    expansions
        .iter()
        .map(|expansion| {
            let points = |path: &crate::geometry::Polygon| {
                path.points()
                    .iter()
                    .map(|point| (point.x(), point.y()))
                    .collect::<Vec<_>>()
            };
            (
                expansion.src_id,
                expansion.boundary_id,
                points(expansion.expolygon.contour()),
                expansion.expolygon.holes().iter().map(points).collect(),
            )
        })
        .collect()
}

#[test]
fn task22o30_empty_and_singleton_keep_complete_direct_contour() {
    let parameters = params((100.0, 100.0, 0, 110.0, 25.0, 0.5));
    assert_eq!(propagate_waves_ex(&[], &[], &parameters), Ok(Vec::new()));

    let output = propagate_waves_ex(
        &[seed(7, 0, &[(0, 400), (0, 600)])],
        &[square_boundary()],
        &parameters,
    )
    .unwrap();
    assert_eq!(
        snapshots(&output),
        vec![(
            7,
            0,
            vec![(100, 400), (100, 600), (12, 699), (0, 688), (0, 312)],
            vec![],
        )]
    );
}

#[test]
fn task22o30_equal_key_multi_output_keeps_nonzero_hole_topology() {
    let boundary = [square_boundary()];
    let seeds = [seed(
        9,
        0,
        &[(200, 200), (800, 200), (800, 800), (200, 800), (200, 200)],
    )];
    let parameters = params((100.0, 100.0, 0, 110.0, 25.0, 0.5));
    assert_eq!(
        snapshots(&propagate_waves_ex(&seeds, &boundary, &parameters).unwrap()),
        vec![(
            9,
            0,
            vec![
                (900, 200),
                (900, 800),
                (800, 900),
                (200, 900),
                (100, 800),
                (100, 200),
                (200, 100),
                (800, 100),
            ],
            vec![vec![(300, 300), (300, 700), (700, 700), (700, 300)]],
        )]
    );
}

#[test]
fn task22o30_adjacent_groups_preserve_islands_ids_and_boundary_first_order() {
    let boundaries = [square_boundary(), square_boundary(), square_boundary()];
    let seeds = [
        seed(3, 0, &[(0, 200), (0, 300)]),
        seed(3, 0, &[(0, 700), (0, 800)]),
        seed(8, 0, &[(0, 400), (0, 600)]),
        seed(8, 1, &[(0, 400), (0, 600)]),
        seed(1, 2, &[(0, 400), (0, 600)]),
    ];
    let parameters = params((100.0, 100.0, 1, 220.0, 25.0, 0.5));
    assert_eq!(
        snapshots(&propagate_waves_ex(&seeds, &boundaries, &parameters).unwrap()),
        vec![
            (
                3,
                0,
                vec![
                    (66, 537),
                    (166, 625),
                    (200, 700),
                    (200, 800),
                    (175, 866),
                    (87, 965),
                    (0, 970),
                    (0, 517),
                ],
                vec![],
            ),
            (
                3,
                0,
                vec![
                    (66, 37),
                    (166, 125),
                    (200, 200),
                    (200, 300),
                    (175, 366),
                    (87, 465),
                    (0, 470),
                    (0, 17),
                ],
                vec![],
            ),
            (
                8,
                0,
                vec![
                    (66, 237),
                    (166, 325),
                    (200, 400),
                    (200, 600),
                    (175, 666),
                    (87, 765),
                    (0, 770),
                    (0, 217),
                ],
                vec![],
            ),
            (
                8,
                1,
                vec![
                    (66, 237),
                    (166, 325),
                    (200, 400),
                    (200, 600),
                    (175, 666),
                    (87, 765),
                    (0, 770),
                    (0, 217),
                ],
                vec![],
            ),
            (
                1,
                2,
                vec![
                    (66, 237),
                    (166, 325),
                    (200, 400),
                    (200, 600),
                    (175, 666),
                    (87, 765),
                    (0, 770),
                    (0, 217),
                ],
                vec![],
            ),
        ]
    );
}

#[test]
fn task22o30_zero_propagation_emits_no_placeholder() {
    let boundary = [expolygon(
        &[(0, 0), (0, 1000), (1000, 1000), (1000, 0)],
        vec![],
    )];
    assert_eq!(
        propagate_waves_ex(
            &[seed(13, 0, &[(0, 400), (0, 600)])],
            &boundary,
            &params((100.0, 100.0, 0, 110.0, 25.0, 0.5)),
        ),
        Ok(Vec::new())
    );
}

#[test]
fn task22o30_unsorted_success_is_debug_only_and_release_keeps_adjacent_groups() {
    let boundary = [square_boundary()];
    let seeds = [
        seed(1, 0, &[(0, 200), (0, 300)]),
        seed(2, 0, &[(0, 400), (0, 600)]),
        seed(1, 0, &[(0, 700), (0, 800)]),
    ];
    let parameters = params((100.0, 100.0, 0, 110.0, 25.0, 0.5));

    #[cfg(debug_assertions)]
    assert!(
        std::panic::catch_unwind(|| propagate_waves_ex(&seeds, &boundary, &parameters)).is_err()
    );

    #[cfg(not(debug_assertions))]
    assert_eq!(
        snapshots(&propagate_waves_ex(&seeds, &boundary, &parameters).unwrap()),
        vec![
            (
                1,
                0,
                vec![(100, 200), (100, 300), (12, 399), (0, 388), (0, 112)],
                vec![],
            ),
            (
                2,
                0,
                vec![(100, 400), (100, 600), (12, 699), (0, 688), (0, 312)],
                vec![],
            ),
            (
                1,
                0,
                vec![(100, 700), (100, 800), (12, 899), (0, 888), (0, 612)],
                vec![],
            ),
        ]
    );
}

#[test]
fn task22o30_propagation_error_precedes_unsorted_debug_assertion() {
    const HIGH: i64 = 0x3fff_ffff_ffff_ffff;
    let boundary = [expolygon(
        &[
            (HIGH - 5000, -1000),
            (HIGH, -1000),
            (HIGH, 1000),
            (HIGH - 5000, 1000),
        ],
        vec![],
    )];
    let parameters = params((100.0, 100.0, 0, 500.0, 25.0, 0.1));
    let unsorted = [
        seed(2, 0, &[(HIGH - 10, -100), (HIGH - 10, 100)]),
        seed(1, 0, &[(HIGH - 1000, -100), (HIGH - 1000, 100)]),
    ];
    let result = std::panic::catch_unwind(|| propagate_waves_ex(&unsorted, &boundary, &parameters))
        .expect("propagation error must escape before the sorted debug assertion");
    assert_eq!(result, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(
        propagate_waves_ex(&unsorted[..1], &boundary, &parameters),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
