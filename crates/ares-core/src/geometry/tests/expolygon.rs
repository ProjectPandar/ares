use crate::geometry::{ExPolygon, Point, Polygon, keep_largest_contour_only};

#[test]
fn task22f_polygon_into_points_preserves_order_without_normalization() {
    let points = vec![
        Point::new(5, 5),
        Point::new(0, 5),
        Point::new(0, 0),
        Point::new(5, 5),
    ];

    assert_eq!(Polygon::new(points.clone()).into_points(), points);
}

#[test]
fn task22f_expolygon_owns_contour_and_ordered_holes() {
    let contour = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(20, 0),
        Point::new(20, 20),
        Point::new(0, 20),
    ]);
    let holes = vec![
        Polygon::new(vec![
            Point::new(2, 2),
            Point::new(2, 4),
            Point::new(4, 4),
            Point::new(4, 2),
        ]),
        Polygon::new(vec![
            Point::new(10, 10),
            Point::new(10, 14),
            Point::new(14, 14),
            Point::new(14, 10),
        ]),
    ];

    let expolygon = ExPolygon::new(contour.clone(), holes.clone());
    assert_eq!(expolygon.contour(), &contour);
    assert_eq!(expolygon.holes(), holes.as_slice());
    assert_eq!(expolygon.into_parts(), (contour, holes));
}

#[test]
fn task22h_empty_expolygons_are_an_exact_identity() {
    assert_largest_contour_identity(Vec::new());
}

#[test]
fn task22h_single_clockwise_expolygon_with_holes_is_an_exact_identity() {
    assert_largest_contour_identity(vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(3, 3),
            Point::new(3, 23),
            Point::new(23, 23),
            Point::new(23, 3),
        ]),
        vec![
            Polygon::new(vec![
                Point::new(7, 7),
                Point::new(13, 7),
                Point::new(13, 13),
                Point::new(7, 13),
            ]),
            Polygon::new(vec![
                Point::new(17, 17),
                Point::new(17, 15),
                Point::new(15, 15),
                Point::new(15, 17),
            ]),
        ],
    )]);
}

#[test]
fn task22h_single_degenerate_expolygon_is_an_exact_identity() {
    assert_largest_contour_identity(vec![ExPolygon::new(
        Polygon::new(vec![Point::new(0, 0), Point::new(5, 5), Point::new(10, 10)]),
        Vec::new(),
    )]);
}

fn assert_largest_contour_identity(mut expolygons: Vec<ExPolygon>) {
    let expected = expolygons.clone();
    keep_largest_contour_only(&mut expolygons);
    assert_eq!(expolygons, expected);
}

#[test]
fn task22h_multiple_expolygons_keep_the_strict_signed_maximum() {
    let small = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ]),
        Vec::new(),
    );
    let largest = ExPolygon::new(
        Polygon::new(vec![
            Point::new(30, 30),
            Point::new(50, 30),
            Point::new(50, 50),
            Point::new(30, 50),
        ]),
        Vec::new(),
    );
    let later = ExPolygon::new(
        Polygon::new(vec![
            Point::new(60, 60),
            Point::new(75, 60),
            Point::new(75, 75),
            Point::new(60, 75),
        ]),
        Vec::new(),
    );
    let mut expolygons = vec![small, largest.clone(), later];
    keep_largest_contour_only(&mut expolygons);
    assert_eq!(expolygons, vec![largest]);
}

#[test]
fn task22h_negative_absolute_area_decoy_does_not_beat_a_positive_contour() {
    let negative_decoy = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(0, 1_000),
            Point::new(1_000, 1_000),
            Point::new(1_000, 0),
        ]),
        Vec::new(),
    );
    let positive = ExPolygon::new(
        Polygon::new(vec![
            Point::new(2_000, 2_000),
            Point::new(2_005, 2_000),
            Point::new(2_005, 2_005),
            Point::new(2_000, 2_005),
        ]),
        Vec::new(),
    );
    let mut expolygons = vec![negative_decoy, positive.clone()];
    keep_largest_contour_only(&mut expolygons);
    assert_eq!(expolygons, vec![positive]);
}

#[test]
fn task22h_equal_signed_areas_keep_the_first_distinct_expolygon() {
    let first = ExPolygon::new(
        Polygon::new(vec![
            Point::new(1, 1),
            Point::new(11, 1),
            Point::new(11, 11),
            Point::new(1, 11),
        ]),
        Vec::new(),
    );
    let second = ExPolygon::new(
        Polygon::new(vec![
            Point::new(101, 101),
            Point::new(111, 101),
            Point::new(111, 111),
            Point::new(101, 111),
        ]),
        Vec::new(),
    );
    let mut expolygons = vec![first.clone(), second];

    keep_largest_contour_only(&mut expolygons);

    assert_eq!(expolygons, vec![first]);
}

#[test]
fn task22h_ranking_uses_contour_area_and_preserves_ordered_holes() {
    let large_contour_with_larger_hole = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(20, 0),
            Point::new(20, 20),
            Point::new(0, 20),
        ]),
        vec![
            Polygon::new(vec![
                Point::new(1, 1),
                Point::new(1, 19),
                Point::new(19, 19),
                Point::new(19, 1),
            ]),
            Polygon::new(vec![
                Point::new(2, 2),
                Point::new(4, 2),
                Point::new(4, 4),
                Point::new(2, 4),
            ]),
        ],
    );
    let smaller_solid = ExPolygon::new(
        Polygon::new(vec![
            Point::new(30, 30),
            Point::new(45, 30),
            Point::new(45, 45),
            Point::new(30, 45),
        ]),
        Vec::new(),
    );
    let mut expolygons = vec![large_contour_with_larger_hole.clone(), smaller_solid];

    keep_largest_contour_only(&mut expolygons);

    assert_eq!(expolygons, vec![large_contour_with_larger_hole]);
}

#[test]
#[should_panic]
fn task22h_multiple_all_nonpositive_contours_violate_the_internal_invariant() {
    let mut expolygons = vec![
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(0, 10),
                Point::new(10, 10),
                Point::new(10, 0),
            ]),
            Vec::new(),
        ),
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(20, 20),
                Point::new(25, 25),
                Point::new(30, 30),
            ]),
            Vec::new(),
        ),
    ];

    keep_largest_contour_only(&mut expolygons);
}
