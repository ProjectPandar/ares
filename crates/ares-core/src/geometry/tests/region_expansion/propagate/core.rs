use super::{seed, square_boundary};
use crate::geometry::propagate_waves;
use crate::geometry::tests::region_expansion::helpers::{params, snapshots};

#[test]
fn task22o27_empty_seeds_do_not_access_boundaries() {
    assert_eq!(
        propagate_waves(&[], &[], &params((100.0, 100.0, 0, 110.0, 25.0, 0.5))),
        Ok(Vec::new())
    );
}

#[test]
fn task22o27_open_closed_and_staged_waves_match_source_order() {
    let boundary = [square_boundary()];
    let one = propagate_waves(
        &[seed(7, 0, &[(0, 400), (0, 600)])],
        &boundary,
        &params((100.0, 100.0, 0, 110.0, 25.0, 0.5)),
    )
    .unwrap();
    assert_eq!(
        snapshots(&one),
        vec![(
            7,
            0,
            vec![(100, 400), (100, 600), (12, 699), (0, 688), (0, 312)]
        )]
    );

    let staged = propagate_waves(
        &[seed(8, 0, &[(0, 400), (0, 600)])],
        &boundary,
        &params((100.0, 100.0, 2, 330.0, 25.0, 0.5)),
    )
    .unwrap();
    assert_eq!(
        snapshots(&staged),
        vec![(
            8,
            0,
            vec![
                (95, 141),
                (132, 162),
                (232, 250),
                (257, 284),
                (291, 359),
                (300, 400),
                (300, 600),
                (294, 635),
                (269, 701),
                (250, 732),
                (162, 831),
                (93, 865),
                (6, 870),
                (0, 864),
                (0, 143),
                (29, 121),
            ]
        )]
    );

    let closed = propagate_waves(
        &[seed(
            9,
            0,
            &[(100, 400), (300, 400), (300, 600), (100, 600), (100, 400)],
        )],
        &boundary,
        &params((100.0, 100.0, 0, 110.0, 25.0, 0.5)),
    )
    .unwrap();
    assert_eq!(
        snapshots(&closed),
        vec![(
            9,
            0,
            vec![
                (400, 400),
                (400, 600),
                (300, 700),
                (100, 700),
                (0, 600),
                (0, 400),
                (100, 300),
                (300, 300),
            ]
        )]
    );
}
