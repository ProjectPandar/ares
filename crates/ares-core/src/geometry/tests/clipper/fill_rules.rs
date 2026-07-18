use super::helpers::{execute, polygons};
use crate::geometry::clipper::{ClipOperation, ClipperOptions, FillRule};

const OUTER: &[(i64, i64)] = &[(0, 0), (40, 0), (40, 40), (0, 40)];
const OUTER_REVERSED: &[(i64, i64)] = &[(0, 0), (0, 40), (40, 40), (40, 0)];
const INNER_SAME: &[(i64, i64)] = &[(10, 10), (30, 10), (30, 30), (10, 30)];
const INNER_OPPOSITE: &[(i64, i64)] = &[(10, 10), (10, 30), (30, 30), (30, 10)];
const OUTER_OUTPUT: &[&[(i64, i64)]] = &[&[(40, 40), (0, 40), (0, 0), (40, 0)]];
const HOLE_OUTPUT: &[&[(i64, i64)]] = &[
    &[(40, 40), (0, 40), (0, 0), (40, 0)],
    &[(10, 10), (10, 30), (30, 30), (30, 10)],
];
const INNER_OUTPUT: &[&[(i64, i64)]] = &[&[(30, 30), (10, 30), (10, 10), (30, 10)]];
type OracleCoordinates = &'static [(i64, i64)];
type SubjectFillCase = (FillRule, &'static [OracleCoordinates]);

#[test]
fn task22f_fill_rules_distinguish_same_and_opposite_nested_winding() {
    let cases = [
        (FillRule::EvenOdd, HOLE_OUTPUT, HOLE_OUTPUT),
        (FillRule::NonZero, OUTER_OUTPUT, HOLE_OUTPUT),
        (FillRule::Positive, OUTER_OUTPUT, HOLE_OUTPUT),
        (FillRule::Negative, &[][..], &[][..]),
    ];

    for (fill, expected_same, expected_opposite) in cases {
        let same = execute(
            polygons(&[OUTER, INNER_SAME]),
            Vec::new(),
            ClipOperation::Union,
            (fill, fill),
            ClipperOptions::default(),
        );
        assert_eq!(same, polygons(expected_same), "same winding {fill:?}");

        let opposite = execute(
            polygons(&[OUTER, INNER_OPPOSITE]),
            Vec::new(),
            ClipOperation::Union,
            (fill, fill),
            ClipperOptions::default(),
        );
        assert_eq!(
            opposite,
            polygons(expected_opposite),
            "opposite winding {fill:?}"
        );
    }
}

#[test]
fn task22f_fill_rules_distinguish_duplicate_and_coincident_reversed_rings() {
    let even_odd = execute(
        polygons(&[OUTER, OUTER]),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::EvenOdd, FillRule::EvenOdd),
        ClipperOptions::default(),
    );
    assert!(even_odd.is_empty());

    let non_zero = execute(
        polygons(&[OUTER, OUTER]),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );
    assert_eq!(non_zero, polygons(OUTER_OUTPUT));

    let reversed = execute(
        polygons(&[OUTER, OUTER_REVERSED]),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );
    assert!(reversed.is_empty());
}

#[test]
fn task22f_fill_rules_apply_subject_and_clip_rules_independently() {
    let subject = polygons(&[OUTER, OUTER]);
    let clip = polygons(&[INNER_SAME]);
    let subject_cases: &[SubjectFillCase] = &[
        (FillRule::EvenOdd, &[][..]),
        (FillRule::NonZero, INNER_OUTPUT),
    ];

    for &(subject_fill, expected) in subject_cases {
        let actual = execute(
            subject.clone(),
            clip.clone(),
            ClipOperation::Intersection,
            (subject_fill, FillRule::Positive),
            ClipperOptions::default(),
        );
        assert_eq!(actual, polygons(expected), "subject fill {subject_fill:?}");
    }

    let negative_clip_excluded = execute(
        subject.clone(),
        polygons(&[INNER_OPPOSITE]),
        ClipOperation::Intersection,
        (FillRule::NonZero, FillRule::Positive),
        ClipperOptions::default(),
    );
    assert!(negative_clip_excluded.is_empty());

    let negative_clip_included = execute(
        subject,
        polygons(&[INNER_OPPOSITE]),
        ClipOperation::Intersection,
        (FillRule::NonZero, FillRule::Negative),
        ClipperOptions::default(),
    );
    assert_eq!(
        negative_clip_included,
        polygons(&[&[(30, 30), (10, 30), (10, 10), (30, 10)]])
    );
}

#[test]
fn task22f_negative_fill_contributes_a_negative_winding_outer_ring() {
    let actual = execute(
        polygons(&[OUTER_REVERSED]),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::Negative, FillRule::Negative),
        ClipperOptions::default(),
    );

    assert_eq!(actual, polygons(OUTER_OUTPUT));
}
