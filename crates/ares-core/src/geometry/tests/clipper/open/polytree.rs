use super::{point, polyline, square};
use crate::geometry::clipper::{ClipOperation, Clipper, ClipperOptions, FillRule, PathRole};

#[test]
fn task22o6_mixed_open_and_closed_polytree_records_keep_open_at_root() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_open_path(&polyline(&[(-5, 20), (15, 20)]), PathRole::Subject)
        .expect("fixed open coordinates are valid");
    clipper
        .add_closed_path(&square(), PathRole::Subject)
        .expect("fixed closed coordinates are valid");

    let tree = clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    let roots = tree.children().collect::<Vec<_>>();

    assert_eq!(roots.len(), 2);
    assert!(roots[0].is_open());
    assert_eq!(
        roots[0].polyline().points(),
        &[point(15, 20), point(-5, 20)]
    );
    assert!(roots[0].children().next().is_none());
    assert!(!roots[1].is_open());
    assert!(!roots[1].is_hole());
}

#[test]
fn task22o6_into_expolygons_explicitly_ignores_open_root_records() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_open_path(&polyline(&[(-5, 20), (15, 20)]), PathRole::Subject)
        .expect("fixed open coordinates are valid");
    clipper
        .add_closed_path(&square(), PathRole::Subject)
        .expect("fixed closed coordinates are valid");

    let output = clipper
        .execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
        .into_expolygons();
    assert_eq!(output.len(), 1);
    assert_eq!(
        output[0].contour().points(),
        &[point(10, 10), point(0, 10), point(0, 0), point(10, 0)]
    );
}
