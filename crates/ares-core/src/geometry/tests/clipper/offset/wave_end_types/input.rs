use super::{raw_closed, raw_open};
use crate::geometry::Polygon;
use crate::geometry::clipper::{ClipperOffset, JoinType};
use crate::geometry::tests::clipper::offset::helpers::{coordinates, polygon};

#[test]
fn task22o27_end_type_filtering_is_exact_and_strict() {
    let mut open = ClipperOffset::default();
    open.set_shortest_edge_length(5.0);
    open.add_open_round_path(
        &polygon(&[(0, 0), (0, 0), (3, 0), (5, 0), (11, 0)]),
        JoinType::Round,
    );
    assert_eq!(
        open.generate_raw(2.0)
            .iter()
            .map(coordinates)
            .collect::<Vec<_>>(),
        vec![vec![
            (5, -2),
            (11, -2),
            (13, -1),
            (13, 1),
            (11, 2),
            (5, 2),
            (0, 2),
            (-2, 1),
            (-2, -1),
            (0, -2),
        ]]
    );

    let mut closed = ClipperOffset::default();
    closed.set_shortest_edge_length(5.0);
    closed.add_closed_line(&polygon(&[(0, 0), (3, 0), (5, 0)]), JoinType::Round);
    assert_eq!(closed.generate_raw(2.0).len(), 2);
    let exact = raw_open(&[(0, 0), (1, 0), (100, 0)], 2.0);
    assert!(exact[0].contains(&(1, 2)));
    assert!(raw_open(&[], 2.0).is_empty());
    assert!(raw_closed(&[], 2.0).is_empty());
}

#[test]
fn task22o27_non_polygon_delta_thresholds_are_exact() {
    for add in [
        ClipperOffset::add_open_round_path as fn(&mut ClipperOffset, &Polygon, JoinType),
        ClipperOffset::add_closed_line,
    ] {
        let run = |delta| {
            let mut offset = ClipperOffset::default();
            add(&mut offset, &polygon(&[(0, 0), (100, 0)]), JoinType::Round);
            offset.generate_raw(delta).len()
        };
        assert_eq!(run(5.0e-21), 0);
        assert!(run(1.0e-20) > 0);
        assert_eq!(run(0.0), 0);
        assert_eq!(run(-10.0), 0);
    }
}
