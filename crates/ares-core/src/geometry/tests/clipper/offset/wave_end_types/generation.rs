use super::{raw_closed, raw_open};
use crate::geometry::clipper::{ClipperOffset, JoinType};
use crate::geometry::tests::clipper::offset::helpers::{coordinates, polygon};

#[test]
fn task22o27_open_round_preserves_side_and_cap_order() {
    assert_eq!(
        raw_open(&[(0, 0), (100, 0)], 10.0),
        vec![vec![
            (100, -10),
            (104, -9),
            (108, -6),
            (110, -2),
            (110, 2),
            (108, 6),
            (104, 9),
            (100, 10),
            (0, 10),
            (-4, 9),
            (-8, 6),
            (-10, 2),
            (-10, -2),
            (-8, -6),
            (-4, -9),
            (0, -10),
        ]]
    );
    assert_eq!(
        raw_open(&[(100, 0), (0, 0)], 10.0),
        vec![vec![
            (0, 10),
            (-4, 9),
            (-8, 6),
            (-10, 2),
            (-10, -2),
            (-8, -6),
            (-4, -9),
            (0, -10),
            (100, -10),
            (104, -9),
            (108, -6),
            (110, -2),
            (110, 2),
            (108, 6),
            (104, 9),
            (100, 10),
        ]]
    );
    assert_eq!(
        raw_open(&[(0, 0), (100, 0), (100, 100)], 10.0),
        vec![vec![
            (100, -10),
            (104, -9),
            (108, -6),
            (110, -2),
            (110, 0),
            (110, 100),
            (109, 104),
            (106, 108),
            (102, 110),
            (98, 110),
            (94, 108),
            (91, 104),
            (90, 100),
            (90, 0),
            (100, 0),
            (100, 10),
            (0, 10),
            (-4, 9),
            (-8, 6),
            (-10, 2),
            (-10, -2),
            (-8, -6),
            (-4, -9),
            (0, -10),
        ]]
    );
}

#[test]
fn task22o27_one_point_join_and_arc_tolerance_are_literal() {
    let round = vec![vec![(7, 7), (6, 9), (4, 9), (3, 7), (4, 5), (6, 5)]];
    assert_eq!(raw_open(&[(5, 7)], 2.0), round);
    assert_eq!(raw_closed(&[(5, 7)], 2.0), round);

    let mut custom = ClipperOffset::default();
    custom.set_arc_tolerance(0.5);
    custom.add_open_round_path(&polygon(&[(5, 7)]), JoinType::Round);
    assert_eq!(
        custom
            .generate_raw(2.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![vec![(7, 7), (5, 9), (3, 7), (4, 5)]]
    );

    let mut square = ClipperOffset::default();
    square.add_open_round_path(&polygon(&[(5, 7)]), JoinType::Square);
    assert_eq!(
        square
            .generate_raw(2.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![vec![(3, 5), (7, 5), (7, 9), (3, 9)]]
    );
}

#[test]
fn task22o27_closed_line_emits_both_raw_sides_and_fixes_orientation() {
    assert_eq!(
        raw_closed(&[(0, 0), (100, 0), (0, 0)], 10.0),
        vec![
            vec![
                (0, 10),
                (-4, 9),
                (-8, 6),
                (-10, 2),
                (-10, -2),
                (-8, -6),
                (-4, -9),
                (0, -10),
                (100, -10),
                (104, -9),
                (108, -6),
                (110, -2),
                (110, 2),
                (108, 6),
                (104, 9),
                (100, 10),
            ],
            vec![
                (100, -10),
                (104, -9),
                (108, -6),
                (110, -2),
                (110, 2),
                (108, 6),
                (104, 9),
                (100, 10),
                (0, 10),
                (-4, 9),
                (-8, 6),
                (-10, 2),
                (-10, -2),
                (-8, -6),
                (-4, -9),
                (0, -10),
            ],
        ]
    );

    let mut mixed = ClipperOffset::default();
    mixed.add_closed_path(
        &polygon(&[(0, 0), (0, 100), (100, 100), (100, 0)]),
        JoinType::Round,
    );
    mixed.add_closed_line(
        &polygon(&[(200, 0), (300, 0), (300, 100), (200, 100)]),
        JoinType::Round,
    );
    let generated = mixed.generate_raw(10.0);
    assert_eq!(generated.len(), 3);
    assert!(generated[1].area() < 0.0);
}
