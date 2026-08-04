use crate::{
    geometry::{Point, Polygon},
    project_slice::prepare_infill::vertical_shell_projection::{
        GeometryStep, gather, geometry_events, reset_geometry_hooks,
    },
};

use super::square;

#[test]
fn task22o20_combine_holes_clears_on_either_empty_side() {
    reset_geometry_hooks();
    let mut holes = vec![square(0, 10)];
    gather::combine_holes(&mut holes, &[]).unwrap();
    assert!(holes.is_empty());
    gather::combine_holes(&mut holes, &[square(0, 10)]).unwrap();
    assert!(holes.is_empty());
    assert!(geometry_events().is_empty());
}

#[test]
fn task22o20_combine_holes_uses_incremental_nonzero_intersection() {
    reset_geometry_hooks();
    let mut holes = vec![square(0, 20), square(5, 15)];
    gather::combine_holes(&mut holes, &[square(10, 30)]).unwrap();
    assert_eq!(
        holes,
        vec![crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(20, 20),
            crate::geometry::Point::new(10, 20),
            crate::geometry::Point::new(10, 10),
            crate::geometry::Point::new(20, 10),
        ])]
    );
    assert_eq!(geometry_events(), [GeometryStep::HoleIntersection]);
}

#[test]
fn task22o20_combine_shells_copies_then_appends_and_unions() {
    reset_geometry_hooks();
    let mut shell = Vec::new();
    gather::combine_shells(&mut shell, &[]).unwrap();
    gather::combine_shells(&mut shell, &[square(0, 20)]).unwrap();
    assert_eq!(shell, [square(0, 20)]);
    gather::combine_shells(&mut shell, &[]).unwrap();
    assert!(geometry_events().is_empty());
    gather::combine_shells(&mut shell, &[square(10, 30)]).unwrap();
    assert_eq!(
        shell,
        [crate::geometry::Polygon::new(vec![
            crate::geometry::Point::new(20, 10),
            crate::geometry::Point::new(30, 10),
            crate::geometry::Point::new(30, 30),
            crate::geometry::Point::new(10, 30),
            crate::geometry::Point::new(10, 20),
            crate::geometry::Point::new(0, 20),
            crate::geometry::Point::new(0, 0),
            crate::geometry::Point::new(20, 0),
        ])]
    );
    assert_eq!(geometry_events(), [GeometryStep::ShellUnion]);
}

#[test]
fn task22o20_both_combiners_preserve_holed_repeated_disjoint_paths_order() {
    let outer = path(&[(0, 0), (40, 0), (40, 40), (0, 40)]);
    let hole = path(&[(10, 10), (10, 30), (30, 30), (30, 10)]);
    let disjoint = path(&[(100, 0), (120, 0), (120, 20), (100, 20)]);
    let expected = vec![
        path(&[(40, 40), (0, 40), (0, 0), (40, 0)]),
        path(&[(10, 10), (10, 30), (30, 30), (30, 10)]),
        path(&[(120, 20), (100, 20), (100, 0), (120, 0)]),
    ];

    reset_geometry_hooks();
    let mut holes = vec![
        outer.clone(),
        hole.clone(),
        disjoint.clone(),
        disjoint.clone(),
    ];
    gather::combine_holes(&mut holes, &[square(-10, 130)]).unwrap();
    assert_eq!(holes, expected);
    assert_eq!(geometry_events(), [GeometryStep::HoleIntersection]);

    reset_geometry_hooks();
    let mut shell = vec![outer, hole, disjoint.clone()];
    gather::combine_shells(&mut shell, &[disjoint]).unwrap();
    assert_eq!(shell, expected);
    assert_eq!(geometry_events(), [GeometryStep::ShellUnion]);
}

fn path(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}
