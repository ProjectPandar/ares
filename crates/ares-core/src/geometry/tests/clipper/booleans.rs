use super::helpers::{execute, polygon, polygons, traced_fixed_sort};
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

#[test]
fn task22f_large_minima_fixed_sort_freezes_complete_sibling_order() {
    const MINIMUM_KEYS: [i64; 35] = [
        20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20,
        20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 10, 30,
    ];
    const SORTED_IDENTITIES: [usize; 35] = [
        33, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 0, 34,
    ];
    const INPUT: [(i64, i64); 35] = [
        (700, 20),
        (1700, 20),
        (2700, 20),
        (400, 20),
        (1400, 20),
        (2400, 20),
        (100, 20),
        (1100, 20),
        (2100, 20),
        (3100, 20),
        (800, 20),
        (1800, 20),
        (2800, 20),
        (500, 20),
        (1500, 20),
        (2500, 20),
        (200, 20),
        (1200, 20),
        (2200, 20),
        (3200, 20),
        (900, 20),
        (1900, 20),
        (2900, 20),
        (600, 20),
        (1600, 20),
        (2600, 20),
        (300, 20),
        (1300, 20),
        (2300, 20),
        (0, 20),
        (1000, 20),
        (2000, 20),
        (3000, 20),
        (4000, 10),
        (4100, 30),
    ];
    const EXPECTED: [(i64, i64); 35] = [
        (4100, 30),
        (1200, 20),
        (2500, 20),
        (1500, 20),
        (500, 20),
        (2800, 20),
        (1800, 20),
        (800, 20),
        (3100, 20),
        (2100, 20),
        (1100, 20),
        (100, 20),
        (2400, 20),
        (1400, 20),
        (400, 20),
        (2700, 20),
        (700, 20),
        (200, 20),
        (2200, 20),
        (3200, 20),
        (900, 20),
        (1900, 20),
        (2900, 20),
        (600, 20),
        (1600, 20),
        (2600, 20),
        (300, 20),
        (1300, 20),
        (2300, 20),
        (0, 20),
        (1000, 20),
        (2000, 20),
        (3000, 20),
        (1700, 20),
        (4000, 10),
    ];
    let (sorted_identities, _) = traced_fixed_sort(&MINIMUM_KEYS, false);
    assert_eq!(sorted_identities, SORTED_IDENTITIES);
    assert_eq!(
        execute(
            input_rectangles(&INPUT),
            Vec::new(),
            ClipOperation::Union,
            (FillRule::NonZero, FillRule::NonZero),
            ClipperOptions::default(),
        ),
        output_rectangles(&EXPECTED)
    );
}
