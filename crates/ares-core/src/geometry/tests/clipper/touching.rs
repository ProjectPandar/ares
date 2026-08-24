use super::helpers::{execute, polygons};
use crate::geometry::clipper::{ClipOperation, ClipperOptions, FillRule};

const LEFT: &[(i64, i64)] = &[(0, 0), (20, 0), (20, 20), (0, 20)];

fn union(subject: &[&[(i64, i64)]]) -> Vec<crate::geometry::Polygon> {
    execute(
        polygons(subject),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    )
}

#[test]
fn task22f_touching_complete_and_partial_shared_edges_match_fixed_oracle() {
    let complete = union(&[LEFT, &[(20, 0), (40, 0), (40, 20), (20, 20)]]);
    assert_eq!(complete, polygons(&[&[(0, 0), (40, 0), (40, 20), (0, 20)]]));

    let partial = union(&[LEFT, &[(20, 5), (30, 5), (30, 15), (20, 15)]]);
    assert_eq!(
        partial,
        polygons(&[&[
            (0, 20),
            (0, 0),
            (20, 0),
            (20, 5),
            (30, 5),
            (30, 15),
            (20, 15),
            (20, 20),
        ]])
    );
}

#[test]
fn task22f_touching_t_junction_and_vertex_only_contact_remain_separate_paths() {
    let t_junction = union(&[LEFT, &[(10, 20), (15, 30), (5, 30)]]);
    assert_eq!(
        t_junction,
        polygons(&[
            &[(15, 30), (5, 30), (10, 20)],
            &[(20, 20), (0, 20), (0, 0), (20, 0)],
        ])
    );

    let vertex = union(&[LEFT, &[(20, 20), (30, 20), (30, 30), (20, 30)]]);
    assert_eq!(
        vertex,
        polygons(&[
            &[(30, 30), (20, 30), (20, 20), (30, 20)],
            &[(20, 20), (0, 20), (0, 0), (20, 0)],
        ])
    );
}

#[test]
fn task22f_touching_t_junction_difference_keeps_topology_changing_notch() {
    let actual = execute(
        polygons(&[LEFT]),
        polygons(&[&[(10, 20), (5, 10), (15, 10)]]),
        ClipOperation::Difference,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );

    assert_eq!(
        actual,
        polygons(&[&[
            (20, 20),
            (10, 20),
            (15, 10),
            (5, 10),
            (10, 20),
            (0, 20),
            (0, 0),
            (20, 0),
        ]])
    );
}

#[test]
fn task22f_touching_horizontal_minima_and_maxima_preserve_ordered_vertices() {
    let actual = union(&[&[
        (0, 0),
        (30, 0),
        (30, 10),
        (20, 10),
        (20, 20),
        (10, 20),
        (10, 10),
        (0, 10),
    ]]);

    assert_eq!(
        actual,
        polygons(&[&[
            (30, 10),
            (20, 10),
            (20, 20),
            (10, 20),
            (10, 10),
            (0, 10),
            (0, 0),
            (30, 0),
        ]])
    );
}

#[test]
fn task22f_touching_multiple_crossings_in_one_scan_band_match_fixed_oracle() {
    let actual = execute(
        polygons(&[&[(0, 0), (30, 30), (60, 0), (60, 40), (30, 10), (0, 40)]]),
        polygons(&[&[(0, 20), (30, -10), (60, 20), (60, 30), (30, 0), (0, 30)]]),
        ClipOperation::Intersection,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );

    assert_eq!(
        actual,
        polygons(&[
            &[(15, 15), (0, 30), (0, 20), (10, 10)],
            &[(60, 20), (60, 30), (45, 15), (50, 10)],
        ])
    );
}

#[test]
fn task22f_touching_equal_key_intersections_match_fixed_ordered_output() {
    let actual = execute(
        polygons(&[&[(0, 0), (10, 20), (20, 0)], &[(40, 0), (50, 20), (60, 0)]]),
        polygons(&[
            &[(0, 20), (10, 0), (20, 20)],
            &[(40, 20), (50, 0), (60, 20)],
        ]),
        ClipOperation::Intersection,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );

    assert_eq!(
        actual,
        polygons(&[
            &[(15, 10), (10, 20), (5, 10), (10, 0)],
            &[(55, 10), (50, 20), (45, 10), (50, 0)],
        ])
    );
}
