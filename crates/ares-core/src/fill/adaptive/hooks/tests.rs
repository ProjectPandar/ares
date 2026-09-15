use super::*;

fn line(a: (i64, i64), b: (i64, i64)) -> Polyline {
    Polyline::new(vec![Point::new(a.0, a.1), Point::new(b.0, b.1)])
}

fn test_boundary() -> ExPolygon {
    let contour = crate::geometry::Polygon::new(vec![
        Point::new(-10000, -10000),
        Point::new(10000, -10000),
        Point::new(10000, 10000),
        Point::new(-10000, 10000),
    ]);
    ExPolygon::new(contour, Vec::new())
}

#[test]
fn passes_short_sets_through() {
    let single = vec![line((0, 0), (1000, 0))];
    let out = connect_lines_using_hooks(single, &test_boundary(), 100.0, 500.0, 500.0);
    assert_eq!(out.len(), 1);
    assert!(
        connect_lines_using_hooks(Vec::new(), &test_boundary(), 100.0, 500.0, 500.0).is_empty()
    );
}

#[test]
fn zero_hook_length_is_open() {
    let lines = vec![line((0, 0), (1000, 0)), line((2000, 0), (3000, 0))];
    let out = connect_lines_using_hooks(lines, &test_boundary(), 100.0, 0.0, 0.0);
    assert_eq!(out.len(), 2);
}

#[test]
fn t_joint_gets_hook_point() {
    // A horizontal line and a vertical line whose end touches its middle.
    let lines = vec![line((0, 0), (4000, 0)), line((2000, 50), (2000, 4000))];
    let out = connect_lines_using_hooks(lines, &test_boundary(), 100.0, 500.0, 500.0);
    assert!(
        out.iter().any(|p| p.points().len() > 2),
        "the touching polyline grows by the hook point(s)"
    );
}
