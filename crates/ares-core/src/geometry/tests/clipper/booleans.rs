use super::helpers::{execute, polygon, polygons};
use crate::geometry::clipper::{ClipOperation, Clipper, ClipperOptions, FillRule, PathRole};

const NONCONVEX: &[(i64, i64)] = &[(0, 0), (30, 0), (30, 10), (10, 10), (10, 30), (0, 30)];
const CROSSING_CLIP: &[(i64, i64)] = &[(5, -5), (25, -5), (25, 25), (5, 25)];
type OracleCoordinates = &'static [(i64, i64)];
type OperationCase = (ClipOperation, &'static [OracleCoordinates]);

fn input_rectangles(origins_and_heights: &[(i64, i64)]) -> Vec<crate::geometry::Polygon> {
    origins_and_heights
        .iter()
        .map(|&(x, height)| polygon(&[(x, 0), (x + 20, 0), (x + 20, height), (x, height)]))
        .collect()
}

fn output_rectangles(origins_and_heights: &[(i64, i64)]) -> Vec<crate::geometry::Polygon> {
    origins_and_heights
        .iter()
        .map(|&(x, height)| polygon(&[(x + 20, height), (x, height), (x, 0), (x + 20, 0)]))
        .collect()
}

#[test]
fn task22f_closed_boolean_empty_and_canonical_union_match_fixed_oracle() {
    assert!(
        execute(
            Vec::new(),
            Vec::new(),
            ClipOperation::Union,
            (FillRule::NonZero, FillRule::NonZero),
            ClipperOptions::default(),
        )
        .is_empty()
    );

    let actual = execute(
        polygons(&[
            &[(0, 0), (40, 0), (40, 40), (0, 40)],
            &[(10, 10), (30, 10), (30, 30), (10, 30)],
            &[(15, 15), (15, 25), (25, 25), (25, 15)],
        ]),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );
    assert_eq!(actual, polygons(&[&[(40, 40), (0, 40), (0, 0), (40, 0)]]));
}

#[test]
fn task22f_closed_boolean_operations_match_fixed_nonconvex_oracle() {
    let cases: &[OperationCase] = &[
        (
            ClipOperation::Intersection,
            &[&[(25, 10), (10, 10), (10, 25), (5, 25), (5, 0), (25, 0)]],
        ),
        (
            ClipOperation::Union,
            &[&[
                (25, 0),
                (30, 0),
                (30, 10),
                (25, 10),
                (25, 25),
                (10, 25),
                (10, 30),
                (0, 30),
                (0, 0),
                (5, 0),
                (5, -5),
                (25, -5),
            ]],
        ),
        (
            ClipOperation::Difference,
            &[
                &[(5, 25), (10, 25), (10, 30), (0, 30), (0, 0), (5, 0)],
                &[(30, 10), (25, 10), (25, 0), (30, 0)],
            ],
        ),
        (
            ClipOperation::Xor,
            &[&[
                (25, 0),
                (5, 0),
                (5, 25),
                (10, 25),
                (10, 10),
                (25, 10),
                (25, 0),
                (30, 0),
                (30, 10),
                (25, 10),
                (25, 25),
                (10, 25),
                (10, 30),
                (0, 30),
                (0, 0),
                (5, 0),
                (5, -5),
                (25, -5),
            ]],
        ),
    ];

    for &(operation, expected) in cases {
        let actual = execute(
            vec![polygon(NONCONVEX)],
            vec![polygon(CROSSING_CLIP)],
            operation,
            (FillRule::NonZero, FillRule::NonZero),
            ClipperOptions::default(),
        );
        assert_eq!(actual, polygons(expected), "operation {operation:?}");
    }
}

#[test]
fn task22f_closed_boolean_execute_consumes_input_until_clear_and_readd() {
    let subject = vec![polygon(NONCONVEX)];
    let clip = vec![polygon(CROSSING_CLIP)];
    let mut clipper = Clipper::new(ClipperOptions::default());
    let expected = polygons(&[&[(25, 10), (10, 10), (10, 25), (5, 25), (5, 0), (25, 0)]]);

    assert_eq!(
        clipper.add_closed_paths(&subject, PathRole::Subject),
        Ok(true)
    );
    assert_eq!(clipper.add_closed_paths(&clip, PathRole::Clip), Ok(true));
    assert_eq!(
        clipper.execute_paths(
            ClipOperation::Intersection,
            FillRule::NonZero,
            FillRule::NonZero,
        ),
        Ok(expected.clone())
    );
    assert!(
        clipper
            .execute_paths(
                ClipOperation::Intersection,
                FillRule::NonZero,
                FillRule::NonZero,
            )
            .expect("closed Clipper execution accepts flat output")
            .is_empty()
    );

    clipper.clear();
    assert_eq!(
        clipper.add_closed_paths(&subject, PathRole::Subject),
        Ok(true)
    );
    assert_eq!(clipper.add_closed_paths(&clip, PathRole::Clip), Ok(true));
    assert_eq!(
        clipper.execute_paths(
            ClipOperation::Intersection,
            FillRule::NonZero,
            FillRule::NonZero,
        ),
        Ok(expected)
    );
}

#[test]
fn task22f_equal_height_minima_match_complete_fixed_oracle_order() {
    let subject = input_rectangles(&[(0, 20), (50, 20)]);

    assert_eq!(
        execute(
            subject,
            Vec::new(),
            ClipOperation::Union,
            (FillRule::NonZero, FillRule::NonZero),
            ClipperOptions::default(),
        ),
        output_rectangles(&[(50, 20), (0, 20)])
    );
}
