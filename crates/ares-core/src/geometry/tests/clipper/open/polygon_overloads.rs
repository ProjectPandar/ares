use super::{coordinates, polygon, square};
use crate::geometry::clipper::{diff_pl, intersection_pl};

#[test]
fn task22o6_polygon_intersection_appends_closure_and_recombines_loop_portion() {
    let subject = [square()];
    let clip = [polygon(&[(5, -5), (15, -5), (15, 15), (5, 15)])];

    assert_eq!(
        coordinates(&intersection_pl(&subject, &clip).expect("fixed coordinates are valid")),
        vec![vec![(5, 0), (10, 0), (10, 10), (5, 10)]]
    );
}

#[test]
fn task22o6_polygon_difference_recombines_fragments_split_at_first_point() {
    let subject = [square()];
    let clip = [polygon(&[(5, -5), (15, -5), (15, 15), (5, 15)])];

    assert_eq!(
        coordinates(&diff_pl(&subject, &clip).expect("fixed coordinates are valid")),
        vec![vec![(5, 10), (0, 10), (0, 0), (5, 0)]]
    );
}
