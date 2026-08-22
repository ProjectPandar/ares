use super::helpers::{execute, polygons, traced_fixed_sort};
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

#[test]
fn task22f_large_intersections_freeze_output_and_pre_adjacency_permutation() {
    const INTERSECTION_Y_KEYS: [i64; 36] = [
        10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
        10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 11, 11,
    ];
    const PRE_ADJACENCY_IDENTITIES: [usize; 36] = [
        35, 34, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 1, 18,
    ];

    let (pre_adjacency, _) = traced_fixed_sort(&INTERSECTION_Y_KEYS, true);
    assert_eq!(pre_adjacency, PRE_ADJACENCY_IDENTITIES);

    let subject = polygons(&[
        &[(0, 0), (10, 20), (20, 0)],
        &[(1300, 0), (1310, 20), (1320, 0)],
        &[(900, 0), (910, 20), (920, 0)],
        &[(500, 0), (510, 20), (520, 0)],
        &[(100, 0), (110, 20), (120, 0)],
        &[(1400, 0), (1410, 20), (1420, 0)],
        &[(1000, 0), (1010, 20), (1020, 0)],
        &[(600, 0), (610, 20), (620, 0)],
        &[(200, 0), (210, 20), (220, 0)],
        &[(1500, 0), (1510, 20), (1520, 0)],
        &[(1100, 0), (1110, 20), (1120, 0)],
        &[(700, 0), (710, 20), (720, 0)],
        &[(300, 0), (310, 20), (320, 0)],
        &[(1600, 0), (1610, 20), (1620, 0)],
        &[(1200, 0), (1210, 20), (1220, 0)],
        &[(800, 0), (810, 20), (820, 0)],
        &[(400, 0), (410, 20), (420, 0)],
        &[(2000, 0), (2010, 20), (2020, 0)],
    ]);
    let clip = polygons(&[
        &[(300, 20), (310, 0), (320, 20)],
        &[(1000, 20), (1010, 0), (1020, 20)],
        &[(0, 20), (10, 0), (20, 20)],
        &[(700, 20), (710, 0), (720, 20)],
        &[(1400, 20), (1410, 0), (1420, 20)],
        &[(400, 20), (410, 0), (420, 20)],
        &[(1100, 20), (1110, 0), (1120, 20)],
        &[(100, 20), (110, 0), (120, 20)],
        &[(800, 20), (810, 0), (820, 20)],
        &[(1500, 20), (1510, 0), (1520, 20)],
        &[(500, 20), (510, 0), (520, 20)],
        &[(1200, 20), (1210, 0), (1220, 20)],
        &[(200, 20), (210, 0), (220, 20)],
        &[(900, 20), (910, 0), (920, 20)],
        &[(1600, 20), (1610, 0), (1620, 20)],
        &[(600, 20), (610, 0), (620, 20)],
        &[(1300, 20), (1310, 0), (1320, 20)],
        &[(2000, 20), (2010, 4), (2020, 20)],
    ]);
    let expected = polygons(&[
        &[(2014, 11), (2010, 20), (2006, 11), (2010, 4)],
        &[(1015, 10), (1010, 20), (1005, 10), (1010, 0)],
        &[(15, 10), (10, 20), (5, 10), (10, 0)],
        &[(715, 10), (710, 20), (705, 10), (710, 0)],
        &[(1415, 10), (1410, 20), (1405, 10), (1410, 0)],
        &[(415, 10), (410, 20), (405, 10), (410, 0)],
        &[(1115, 10), (1110, 20), (1105, 10), (1110, 0)],
        &[(115, 10), (110, 20), (105, 10), (110, 0)],
        &[(815, 10), (810, 20), (805, 10), (810, 0)],
        &[(1515, 10), (1510, 20), (1505, 10), (1510, 0)],
        &[(515, 10), (510, 20), (505, 10), (510, 0)],
        &[(1215, 10), (1210, 20), (1205, 10), (1210, 0)],
        &[(215, 10), (210, 20), (205, 10), (210, 0)],
        &[(915, 10), (910, 20), (905, 10), (910, 0)],
        &[(1615, 10), (1610, 20), (1605, 10), (1610, 0)],
        &[(615, 10), (610, 20), (605, 10), (610, 0)],
        &[(1315, 10), (1310, 20), (1305, 10), (1310, 0)],
        &[(315, 10), (310, 20), (305, 10), (310, 0)],
    ]);

    assert_eq!(
        execute(
            subject,
            clip,
            ClipOperation::Intersection,
            (FillRule::NonZero, FillRule::NonZero),
            ClipperOptions::default(),
        ),
        expected
    );
}
