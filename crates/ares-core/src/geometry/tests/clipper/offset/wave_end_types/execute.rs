use crate::geometry::clipper::{ClipperError, ClipperOffset, JoinType};
use crate::geometry::tests::clipper::offset::helpers::{coordinates, polygon};

#[test]
fn task22o27_wave_end_cleanup_and_range_failure_match_clipper() {
    let mut offset = ClipperOffset::default();
    offset.add_open_round_path(&polygon(&[(0, 0), (100, 0)]), JoinType::Round);
    assert_eq!(
        offset
            .execute_paths(10.0)
            .unwrap()
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![vec![
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
            (100, -10),
        ]]
    );

    let mut raw_closed_as_open = ClipperOffset::default();
    raw_closed_as_open.add_open_round_path(
        &polygon(&[(0, 0), (100, 0), (100, 100), (0, 100), (0, 0)]),
        JoinType::Round,
    );
    assert_eq!(
        raw_closed_as_open
            .execute_paths(10.0)
            .unwrap()
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![
            vec![
                (-8, -6),
                (-6, -8),
                (-2, -10),
                (100, -10),
                (104, -9),
                (108, -6),
                (110, -2),
                (110, 100),
                (109, 104),
                (106, 108),
                (102, 110),
                (0, 110),
                (-4, 109),
                (-8, 106),
                (-10, 102),
                (-10, -2),
            ],
            vec![(10, 10), (10, 90), (90, 90), (90, 10)],
        ]
    );

    let mut outside = ClipperOffset::default();
    outside.add_closed_line(
        &polygon(&[(0x4000_0000_0000_0000, 0), (0x4000_0000_0000_1000, 0)]),
        JoinType::Round,
    );
    assert_eq!(
        outside.execute_paths(1_024.0),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
