use super::helpers::polygon;
use crate::geometry::{ClipperError, ExPolygon, difference_ex, intersection_ex};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn rectangle_output(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (max_x, max_y),
            (min_x, max_y),
            (min_x, min_y),
            (max_x, min_y),
        ]),
        Vec::new(),
    )
}

fn expolygon(contour: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(polygon(contour), Vec::new())
}

fn expolygon_with_holes(contour: &[(i64, i64)], holes: &[&[(i64, i64)]]) -> ExPolygon {
    ExPolygon::new(
        polygon(contour),
        holes
            .iter()
            .map(|coordinates| polygon(coordinates))
            .collect(),
    )
}

#[test]
fn task22j_boolean_ex_empty_inputs_and_empty_first_pass_are_empty() {
    let square = vec![rectangle(0, 0, 1_000, 1_000)];
    let disjoint = vec![rectangle(2_000, 0, 3_000, 1_000)];
    let expected_square = vec![rectangle_output(0, 0, 1_000, 1_000)];

    assert_eq!(difference_ex(&[], &[]), Ok(Vec::new()));
    assert_eq!(intersection_ex(&[], &[]), Ok(Vec::new()));
    assert_eq!(difference_ex(&square, &[]), Ok(expected_square));
    assert_eq!(difference_ex(&[], &square), Ok(Vec::new()));
    assert_eq!(intersection_ex(&square, &[]), Ok(Vec::new()));
    assert_eq!(intersection_ex(&[], &square), Ok(Vec::new()));
    assert_eq!(difference_ex(&square, &square), Ok(Vec::new()));
    assert_eq!(intersection_ex(&square, &disjoint), Ok(Vec::new()));
}

#[test]
fn task22j_boolean_ex_modifier_band_forbids_an_extra_paths_union() {
    let subject = vec![expolygon(&[
        (10_000_000, 1_000_000),
        (-10_000_000, 1_000_000),
        (-10_000_000, -1_000_000),
        (10_000_000, -1_000_000),
    ])];
    let band = vec![expolygon(&[
        (5_000_000, 1_000_000),
        (-5_000_000, 1_000_000),
        (-5_000_000, -1_000_000),
        (5_000_000, -1_000_000),
    ])];

    assert_eq!(
        difference_ex(&subject, &band),
        Ok(vec![
            expolygon(&[
                (-5_000_000, 1_000_000),
                (-10_000_000, 1_000_000),
                (-10_000_000, -1_000_000),
                (-5_000_000, -1_000_000),
            ]),
            expolygon(&[
                (10_000_000, 1_000_000),
                (5_000_000, 1_000_000),
                (5_000_000, -1_000_000),
                (10_000_000, -1_000_000),
            ]),
        ])
    );
    assert_eq!(
        intersection_ex(&subject, &band),
        Ok(vec![expolygon(&[
            (5_000_000, 1_000_000),
            (-5_000_000, 1_000_000),
            (-5_000_000, -1_000_000),
            (5_000_000, -1_000_000),
        ])])
    );
}

#[test]
fn task22j_boolean_ex_simple_same_and_disjoint_inputs_keep_fixed_path_order() {
    let subject = vec![rectangle(0, 0, 1_000, 1_000)];
    let disjoint = vec![rectangle(2_000, 0, 3_000, 1_000)];
    let expected = vec![rectangle_output(0, 0, 1_000, 1_000)];

    assert_eq!(intersection_ex(&subject, &subject), Ok(expected.clone()));
    assert_eq!(difference_ex(&subject, &disjoint), Ok(expected));
}

#[test]
fn task22j_boolean_ex_partial_overlap_matches_fixed_ordered_vectors() {
    let subject = vec![rectangle(0, 0, 100, 100)];
    let clip = vec![rectangle(50, -20, 120, 80)];

    assert_eq!(
        difference_ex(&subject, &clip),
        Ok(vec![expolygon(&[
            (50, 80),
            (100, 80),
            (100, 100),
            (0, 100),
            (0, 0),
            (50, 0),
        ])])
    );
    assert_eq!(
        intersection_ex(&subject, &clip),
        Ok(vec![expolygon(&[(100, 80), (50, 80), (50, 0), (100, 0),])])
    );
}

#[test]
fn task22j_boolean_ex_contained_clip_matches_fixed_hole_and_intersection() {
    let subject = vec![rectangle(0, 0, 100, 100)];
    let clip = vec![rectangle(10, 10, 90, 90)];

    assert_eq!(
        difference_ex(&subject, &clip),
        Ok(vec![expolygon_with_holes(
            &[(100, 100), (0, 100), (0, 0), (100, 0)],
            &[&[(10, 10), (10, 90), (90, 90), (90, 10)]],
        )])
    );
    assert_eq!(
        intersection_ex(&subject, &clip),
        Ok(vec![expolygon(&[(90, 90), (10, 90), (10, 10), (90, 10),])])
    );
}

#[test]
fn task22j_boolean_ex_hole_clip_preserves_nested_island_ownership_order() {
    let subject = vec![rectangle(0, 0, 100, 100)];
    let donut = vec![expolygon_with_holes(
        &[(10, 10), (90, 10), (90, 90), (10, 90)],
        &[&[(20, 20), (20, 80), (80, 80), (80, 20)]],
    )];

    assert_eq!(
        difference_ex(&subject, &donut),
        Ok(vec![
            expolygon_with_holes(
                &[(100, 100), (0, 100), (0, 0), (100, 0)],
                &[&[(10, 10), (10, 90), (90, 90), (90, 10)]],
            ),
            expolygon(&[(80, 80), (20, 80), (20, 20), (80, 20)]),
        ])
    );
    assert_eq!(
        intersection_ex(&subject, &donut),
        Ok(vec![expolygon_with_holes(
            &[(90, 90), (10, 90), (10, 10), (90, 10)],
            &[&[(20, 20), (20, 80), (80, 80), (80, 20)]],
        )])
    );
}

#[test]
fn task22j_boolean_ex_forwards_subject_and_clip_coordinate_errors() {
    let invalid = vec![ExPolygon::new(
        polygon(&[(HI_RANGE + 1, 0), (0, 1), (0, 2)]),
        Vec::new(),
    )];
    let valid = vec![rectangle(0, 0, 10, 10)];

    assert_eq!(
        difference_ex(&invalid, &valid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        difference_ex(&valid, &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_ex(&invalid, &valid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_ex(&valid, &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
