use super::{point, polyline};
use crate::geometry::clipper::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole,
};

#[test]
fn task22o6_open_input_accepts_two_points_and_preserves_collinear_vertices() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    let path = polyline(&[(0, 0), (5, 5), (10, 10)]);

    assert_eq!(clipper.add_open_path(&path, PathRole::Subject), Ok(true));
    let snapshot = clipper.input_snapshot();
    assert_eq!(snapshot.edges.len(), 3);
    assert!(snapshot.edges.iter().all(|edge| !edge.removed));
    assert_eq!(snapshot.edges[2].wind_delta, Some(0));
}

#[test]
fn task22o6_open_input_rejects_fewer_than_two_points() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    assert_eq!(
        clipper.add_open_path(&polyline(&[]), PathRole::Subject),
        Ok(false)
    );
    assert_eq!(
        clipper.add_open_path(&polyline(&[(1, 2)]), PathRole::Subject),
        Ok(false)
    );
    assert_eq!(
        clipper.add_open_path(&polyline(&[(1, 2), (3, 4)]), PathRole::Subject),
        Ok(true)
    );
}

#[test]
fn task22o6_open_input_rejects_clip_role_at_public_boundary() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    assert_eq!(
        clipper.add_open_path(&polyline(&[(0, 0), (10, 0)]), PathRole::Clip),
        Err(ClipperError::OpenPathMustBeSubject)
    );
    assert!(clipper.input_snapshot().edges.is_empty());
}

#[test]
fn task22o6_open_input_retains_matching_start_and_end() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    let path = polyline(&[(0, 0), (10, 0), (10, 10), (0, 10), (0, 0)]);

    assert_eq!(clipper.add_open_path(&path, PathRole::Subject), Ok(true));
    let snapshot = clipper.input_snapshot();
    assert_eq!(snapshot.edges.len(), 5);
    assert_eq!(snapshot.edges[0].current, Some(point(0, 0)));
    assert_eq!(snapshot.edges[4].current, Some(point(0, 0)));
    assert!(snapshot.edges.iter().all(|edge| !edge.removed));
}

#[test]
fn task22o6_flat_execution_rejects_open_input_without_dropping_it() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_open_path(&polyline(&[(0, 0), (10, 0)]), PathRole::Subject)
        .expect("fixed coordinates are valid");

    assert_eq!(
        clipper.execute_paths(
            ClipOperation::Difference,
            FillRule::NonZero,
            FillRule::NonZero,
        ),
        Err(ClipperError::OpenPathsRequirePolyTree)
    );
}
