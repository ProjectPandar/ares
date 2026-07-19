use super::super::{
    ExPolygon, Point, Polygon,
    simplification::{
        append_simplified_expolygon, distance_to_segment_squared, douglas_peucker,
        simplify_closed_points,
    },
};

const TRANSLATION: i64 = 4_000_000_000_000_000_000;

#[test]
fn task22i_numeric_distance_uses_the_finite_segment() {
    for (point, start, end, expected) in [
        (point(4, 6), point(1, 2), point(1, 2), 25.0),
        (point(-3, 4), point(0, 0), point(4, 0), 25.0),
        (point(7, 4), point(0, 0), point(4, 0), 25.0),
        (point(2, 3), point(0, 0), point(4, 0), 9.0),
    ] {
        assert_eq!(distance_to_segment_squared(point, start, end), expected);
    }
}

#[test]
fn task22i_numeric_distance_subtracts_large_coordinates_before_casting() {
    assert_eq!(
        distance_to_segment_squared(
            point(TRANSLATION, 1),
            point(TRANSLATION, 0),
            point(TRANSLATION + 1, 1),
        ),
        0.5
    );
    assert_eq!(
        distance_to_segment_squared(
            point(TRANSLATION + 1, TRANSLATION + 1),
            point(TRANSLATION, TRANSLATION),
            point(TRANSLATION + 2, TRANSLATION),
        ),
        1.0
    );
}

#[test]
fn task22i_numeric_open_dp_preserves_short_inputs() {
    for (input, tolerance, expected) in [
        (Vec::new(), 1.0, Vec::new()),
        (vec![point(7, -3)], 1.0, vec![point(7, -3)]),
        (
            vec![point(-2, 5), point(9, -4)],
            100.0,
            vec![point(-2, 5), point(9, -4)],
        ),
    ] {
        assert_eq!(douglas_peucker(&input, tolerance), expected);
    }
}

#[test]
fn task22i_numeric_open_dp_removes_equality_and_retains_strict_deviation() {
    let input = vec![point(0, 0), point(1, 1), point(2, 0)];
    assert_eq!(douglas_peucker(&input, 1.0), vec![point(0, 0), point(2, 0)]);
    assert_eq!(douglas_peucker(&input, 0.5), input);
}

#[test]
fn task22i_numeric_open_dp_keeps_the_first_farthest_tie() {
    assert_eq!(
        douglas_peucker(&[point(0, 0), point(1, 1), point(2, 1), point(3, 0)], 0.5,),
        vec![point(0, 0), point(1, 1), point(3, 0)]
    );
    assert_eq!(
        douglas_peucker(
            &[
                point(TRANSLATION, TRANSLATION),
                point(TRANSLATION + 1, TRANSLATION + 1),
                point(TRANSLATION + 2, TRANSLATION + 1),
                point(TRANSLATION + 3, TRANSLATION),
            ],
            0.5,
        ),
        vec![
            point(TRANSLATION, TRANSLATION),
            point(TRANSLATION + 1, TRANSLATION + 1),
            point(TRANSLATION + 3, TRANSLATION),
        ]
    );
}

#[test]
fn task22i_numeric_open_dp_uses_endpoint_stack_order_and_keeps_the_endpoint() {
    assert_eq!(
        douglas_peucker(
            &[
                point(0, 0),
                point(1, 0),
                point(2, 4),
                point(3, 4),
                point(4, 0),
                point(5, 0),
                point(6, 3),
                point(7, 3),
                point(8, 0),
            ],
            1.0,
        ),
        vec![
            point(0, 0),
            point(2, 4),
            point(4, 0),
            point(6, 3),
            point(8, 0),
        ]
    );
    assert_eq!(
        douglas_peucker(&[point(0, 0), point(3, 0), point(0, 0)], 1.0),
        vec![point(0, 0), point(3, 0), point(0, 0)]
    );
}

#[test]
fn task22i_numeric_closed_dp_is_start_dependent_and_appends_unconditionally() {
    let a = point(0, 0);
    let b = point(4, 0);
    let c = point(5, 2);
    let d = point(4, 4);
    let e = point(0, 4);
    let f = point(-1, 2);
    assert_eq!(
        simplify_closed_points(vec![a, b, c, d, e, f], 1.0),
        vec![a, b, d, e]
    );
    assert_eq!(
        simplify_closed_points(vec![c, d, e, f, a, b], 1.0),
        vec![c, d, e, f, a, b]
    );
    assert_eq!(
        simplify_closed_points(vec![a, b, c, d, e, f, a], 1.0),
        vec![a, b, d, e]
    );
}

#[test]
fn task22i_numeric_closed_dp_preserves_exact_threshold_and_short_semantics() {
    let triangle = vec![point(0, 0), point(2, 0), point(1, 1)];
    assert_eq!(
        simplify_closed_points(triangle.clone(), 2.0),
        vec![point(0, 0)]
    );
    assert_eq!(
        simplify_closed_points(triangle.clone(), 1.0),
        vec![point(0, 0), point(2, 0)]
    );
    assert_eq!(simplify_closed_points(triangle.clone(), 0.5), triangle);
    assert_eq!(simplify_closed_points(Vec::new(), 1.0), Vec::new());
    assert_eq!(
        simplify_closed_points(vec![point(7, -3)], 1.0),
        vec![point(7, -3)]
    );
    assert_eq!(
        simplify_closed_points(vec![point(0, 0), point(3, 0)], 1.0),
        vec![point(0, 0), point(3, 0)]
    );
}

#[test]
fn task22i_expolygon_preserves_contour_then_hole_source_order() {
    let input = expolygon(
        &[(25, 25), (20, 30), (15, 0)],
        &[&[(25, 15), (5, 30), (20, 30)]],
    );
    assert_eq!(
        simplify_expolygon(input, 0.1),
        vec![
            expolygon(&[(20, 30), (5, 30), (18, 20)], &[]),
            expolygon(&[(25, 25), (20, 30), (23, 20)], &[]),
            expolygon(&[(22, 17), (18, 20), (15, 0)], &[]),
            expolygon(&[(23, 20), (22, 17), (25, 15)], &[]),
        ]
    );
    let input = expolygon(
        &[(5, 30), (10, 0), (10, 15)],
        &[
            &[(5, 35), (20, 20), (20, 10)],
            &[(25, 10), (15, 35), (25, 20)],
        ],
    );
    assert_eq!(
        simplify_expolygon(input, 0.1),
        vec![
            expolygon(&[(25, 20), (15, 35), (25, 10)], &[]),
            expolygon(&[(20, 20), (5, 35), (20, 10)], &[]),
            expolygon(&[(10, 15), (5, 30), (10, 0)], &[]),
        ]
    );
}

#[test]
fn task22i_expolygon_drops_collapsed_holes_and_contours() {
    let input = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        &[&[(20, 20), (22, 20), (21, 21)]],
    );
    assert_eq!(
        simplify_expolygon(input, 2.0),
        vec![expolygon(&[(100, 100), (0, 100), (0, 0), (100, 0)], &[],)]
    );
    assert_eq!(
        simplify_expolygon(expolygon(&[(0, 0), (2, 0), (1, 1)], &[]), 2.0),
        Vec::new()
    );
}

#[test]
fn task22i_expolygon_preserves_the_three_union_pass_order() {
    let input = expolygon(&[(10, 0), (15, 30), (10, 5), (25, 30)], &[]);
    assert_eq!(
        simplify_expolygon(input, 0.1),
        vec![
            expolygon(&[(25, 30), (11, 7), (10, 0)], &[]),
            expolygon(&[(11, 7), (15, 30), (10, 5)], &[]),
        ]
    );
}

#[test]
fn task22i_expolygon_preserves_nested_island_order() {
    let input = expolygon(
        &[
            (100, 0),
            (100, 100),
            (0, 100),
            (0, 0),
            (100, 0),
            (20, 20),
            (20, 80),
            (80, 80),
            (80, 20),
        ],
        &[&[(40, 40), (60, 40), (60, 60), (40, 60)]],
    );
    assert_eq!(
        simplify_expolygon(input, 0.1),
        vec![
            expolygon(
                &[(100, 100), (0, 100), (0, 0), (100, 0)],
                &[&[(20, 20), (20, 80), (80, 80), (80, 20), (100, 0)]],
            ),
            expolygon(&[(60, 60), (40, 60), (40, 40), (60, 40)], &[]),
        ]
    );
}

#[test]
fn task22i_expolygons_keep_independent_union_scope_and_contiguous_append() {
    let first = expolygon(
        &[
            (0, 0),
            (10, 0),
            (10, 10),
            (20, 10),
            (20, 20),
            (10, 20),
            (10, 10),
            (0, 10),
        ],
        &[],
    );
    let second = expolygon(&[(0, 0), (10, 0), (10, 10), (0, 10)], &[]);
    let mut output = Vec::new();
    append_simplified_expolygon(first, 0.1, &mut output).expect("fixed input is in range");
    append_simplified_expolygon(second, 0.1, &mut output).expect("fixed input is in range");
    assert_eq!(
        output,
        vec![
            expolygon(&[(20, 20), (10, 20), (10, 10), (20, 10)], &[]),
            expolygon(&[(10, 10), (0, 10), (0, 0), (10, 0)], &[]),
            expolygon(&[(10, 10), (0, 10), (0, 0), (10, 0)], &[]),
        ]
    );
}

fn simplify_expolygon(input: ExPolygon, tolerance: f64) -> Vec<ExPolygon> {
    let mut output = Vec::new();
    append_simplified_expolygon(input, tolerance, &mut output).unwrap();
    output
}

fn expolygon(contour: &[(i64, i64)], holes: &[&[(i64, i64)]]) -> ExPolygon {
    ExPolygon::new(polygon(contour), holes.iter().map(|p| polygon(p)).collect())
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| point(x, y)).collect())
}

const fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}
