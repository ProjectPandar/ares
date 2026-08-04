use super::helpers::{polygon, polygons};
use crate::geometry::{intersection_polygons_paths, union_polygons_paths};

#[test]
fn task22o20_paths_adapters_freeze_empty_and_nonzero_union_order() {
    assert!(union_polygons_paths(&[]).unwrap().is_empty());
    let input = polygons(&[
        &[(0, 0), (40, 0), (40, 40), (0, 40)],
        &[(10, 10), (30, 10), (30, 30), (10, 30)],
        &[(15, 15), (15, 25), (25, 25), (25, 15)],
    ]);
    assert_eq!(
        union_polygons_paths(&input).unwrap(),
        vec![polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)])]
    );
}

#[test]
fn task22o20_paths_union_preserves_holed_repeated_disjoint_order() {
    let outer = polygon(&[(0, 0), (40, 0), (40, 40), (0, 40)]);
    let hole = polygon(&[(10, 10), (10, 30), (30, 30), (30, 10)]);
    let disjoint = polygon(&[(100, 0), (120, 0), (120, 20), (100, 20)]);
    assert_eq!(
        union_polygons_paths(&[outer, hole, disjoint.clone(), disjoint]).unwrap(),
        polygons(&[
            &[(40, 40), (0, 40), (0, 0), (40, 0)],
            &[(10, 10), (10, 30), (30, 30), (30, 10)],
            &[(120, 20), (100, 20), (100, 0), (120, 0)],
        ])
    );
}

#[test]
fn task22o20_paths_intersection_preserves_flat_paths_output() {
    let subject = vec![polygon(&[
        (0, 0),
        (30, 0),
        (30, 10),
        (10, 10),
        (10, 30),
        (0, 30),
    ])];
    let clip = vec![polygon(&[(5, -5), (25, -5), (25, 25), (5, 25)])];
    assert_eq!(
        intersection_polygons_paths(&subject, &clip).unwrap(),
        vec![polygon(&[
            (25, 10),
            (10, 10),
            (10, 25),
            (5, 25),
            (5, 0),
            (25, 0),
        ])]
    );
    assert!(intersection_polygons_paths(&[], &clip).unwrap().is_empty());
    assert!(
        intersection_polygons_paths(&subject, &[])
            .unwrap()
            .is_empty()
    );
}
