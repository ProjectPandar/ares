use super::operations::execute;
use super::{coordinates, polyline, square};
use crate::geometry::clipper::ClipOperation;

#[test]
fn task22o6_horizontal_subject_preserves_legal_collinear_output_vertices() {
    let subject = [polyline(&[(-5, 5), (5, 5), (5, 5), (15, 5)])];

    assert_eq!(
        coordinates(&execute(ClipOperation::Intersection, &subject, &[square()])),
        vec![vec![(10, 5), (5, 5), (0, 5)]]
    );
}

#[test]
fn task22o6_endpoint_touch_does_not_materialize_a_one_point_polyline() {
    let subject = [polyline(&[(-5, 5), (0, 5)])];

    assert!(execute(ClipOperation::Intersection, &subject, &[square()]).is_empty());
    assert_eq!(
        coordinates(&execute(ClipOperation::Difference, &subject, &[square()])),
        vec![vec![(0, 5), (-5, 5)]]
    );
}

#[test]
fn task22o6_coincident_horizontal_clip_boundary_uses_scanline_semantics() {
    let subject = [polyline(&[(-5, 0), (15, 0)])];

    assert!(execute(ClipOperation::Intersection, &subject, &[square()]).is_empty());
    assert_eq!(
        coordinates(&execute(ClipOperation::Difference, &subject, &[square()])),
        vec![vec![(15, 0), (-5, 0)]]
    );
}

#[test]
fn task22o6_tangent_vertex_does_not_split_an_outside_path() {
    let subject = [polyline(&[(-5, 15), (5, 10), (15, 15)])];

    assert!(execute(ClipOperation::Intersection, &subject, &[square()]).is_empty());
    assert_eq!(
        coordinates(&execute(ClipOperation::Difference, &subject, &[square()])),
        vec![vec![(-5, 15), (5, 10), (15, 15)]]
    );
}
