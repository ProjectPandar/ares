use super::{coordinates, polyline};
use crate::geometry::clipper::recombine_polylines;

#[test]
fn task22o6_recombine_back_to_front_branch() {
    let mut paths = vec![polyline(&[(0, 0), (1, 0)]), polyline(&[(1, 0), (2, 0)])];
    recombine_polylines(&mut paths);
    assert_eq!(coordinates(&paths), vec![vec![(0, 0), (1, 0), (2, 0)]]);
}

#[test]
fn task22o6_recombine_front_to_back_branch() {
    let mut paths = vec![polyline(&[(1, 0), (2, 0)]), polyline(&[(0, 0), (1, 0)])];
    recombine_polylines(&mut paths);
    assert_eq!(coordinates(&paths), vec![vec![(0, 0), (1, 0), (2, 0)]]);
}

#[test]
fn task22o6_recombine_front_to_front_reverses_joining_path() {
    let mut paths = vec![polyline(&[(1, 0), (2, 0)]), polyline(&[(1, 0), (0, 0)])];
    recombine_polylines(&mut paths);
    assert_eq!(coordinates(&paths), vec![vec![(0, 0), (1, 0), (2, 0)]]);
}

#[test]
fn task22o6_recombine_back_to_back_reverses_joining_path() {
    let mut paths = vec![polyline(&[(0, 0), (1, 0)]), polyline(&[(2, 0), (1, 0)])];
    recombine_polylines(&mut paths);
    assert_eq!(coordinates(&paths), vec![vec![(0, 0), (1, 0), (2, 0)]]);
}

#[test]
fn task22o6_recombine_retries_erased_slot_and_preserves_nested_order() {
    let mut paths = vec![
        polyline(&[(0, 0), (1, 0)]),
        polyline(&[(1, 0), (2, 0)]),
        polyline(&[(2, 0), (3, 0)]),
        polyline(&[(10, 0), (11, 0)]),
    ];
    recombine_polylines(&mut paths);
    assert_eq!(
        coordinates(&paths),
        vec![vec![(0, 0), (1, 0), (2, 0), (3, 0)], vec![(10, 0), (11, 0)]]
    );
}

#[test]
fn task22o6_recombine_uses_first_branch_when_multiple_endpoint_tests_match() {
    let mut paths = vec![polyline(&[(0, 0), (1, 0)]), polyline(&[(1, 0), (0, 0)])];
    recombine_polylines(&mut paths);
    assert_eq!(coordinates(&paths), vec![vec![(0, 0), (1, 0), (0, 0)]]);
}
