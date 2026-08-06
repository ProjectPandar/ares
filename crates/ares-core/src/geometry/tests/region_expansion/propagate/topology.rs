use super::{seed, square_boundary};
use crate::geometry::propagate_waves;
use crate::geometry::tests::region_expansion::helpers::{expolygon, params, polygon, snapshots};

#[test]
fn task22o27_multiple_paths_share_only_their_contiguous_group() {
    let boundaries = [square_boundary(), square_boundary()];
    let together = propagate_waves(
        &[
            seed(3, 0, &[(0, 200), (0, 300)]),
            seed(3, 0, &[(0, 700), (0, 800)]),
        ],
        &boundaries,
        &params((100.0, 100.0, 1, 220.0, 25.0, 0.5)),
    )
    .unwrap();
    assert_eq!(
        snapshots(&together),
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
                ]
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
                ]
            ),
        ]
    );

    let separated = propagate_waves(
        &[
            seed(4, 0, &[(0, 400), (0, 600)]),
            seed(5, 1, &[(0, 400), (0, 600)]),
            seed(4, 0, &[(0, 400), (0, 600)]),
        ],
        &boundaries,
        &params((100.0, 100.0, 0, 110.0, 25.0, 0.5)),
    )
    .unwrap();
    assert_eq!(
        snapshots(&separated)
            .iter()
            .map(|(src, boundary, _)| (*src, *boundary))
            .collect::<Vec<_>>(),
        vec![(4, 0), (5, 1), (4, 0)]
    );
    assert_eq!(separated[0].polygon, separated[2].polygon);
}

#[test]
fn task22o27_positive_hole_clipping_matches_source_topology() {
    let boundary = [expolygon(
        &[(0, 0), (1000, 0), (1000, 1000), (0, 1000)],
        vec![polygon(&[(200, 300), (200, 700), (600, 700), (600, 300)])],
    )];
    let output = propagate_waves(
        &[seed(11, 0, &[(0, 450), (0, 550)])],
        &boundary,
        &params((150.0, 150.0, 2, 500.0, 25.0, 0.75)),
    )
    .unwrap();
    assert_eq!(
        snapshots(&output),
        vec![(
            11,
            0,
            vec![
                (56, 29),
                (161, 49),
                (227, 79),
                (337, 168),
                (382, 229),
                (388, 244),
                (376, 300),
                (200, 300),
                (200, 700),
                (363, 700),
                (355, 783),
                (308, 854),
                (241, 909),
                (88, 973),
                (1, 982),
                (0, 982),
                (0, 28),
                (12, 27),
            ]
        )]
    );
}

#[test]
fn task22o27_wave_clipping_uses_positive_not_nonzero_fill() {
    let clockwise_boundary = [expolygon(
        &[(0, 0), (0, 1000), (1000, 1000), (1000, 0)],
        vec![],
    )];
    assert!(
        propagate_waves(
            &[seed(13, 0, &[(0, 400), (0, 600)])],
            &clockwise_boundary,
            &params((100.0, 100.0, 0, 110.0, 25.0, 0.5)),
        )
        .unwrap()
        .is_empty()
    );
}
