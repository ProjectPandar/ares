use super::helpers::polygon;
use crate::geometry::{
    ClipperError, ExPolygon, JoinType, difference_ex_with_safety_offset, offset_expolygons,
    union_expolygons, xor_ex,
};

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

fn output_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
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

fn donut(outer_min: i64, outer_max: i64, hole_min: i64, hole_max: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (outer_min, outer_min),
            (outer_max, outer_min),
            (outer_max, outer_max),
            (outer_min, outer_max),
        ]),
        vec![polygon(&[
            (hole_min, hole_min),
            (hole_min, hole_max),
            (hole_max, hole_max),
            (hole_max, hole_min),
        ])],
    )
}

#[test]
fn task22l_clipper_union_keeps_current_then_candidate_as_subject_paths() {
    let current = vec![rectangle(0, 0, 100, 100)];
    let candidate = vec![rectangle(50, 0, 150, 100)];

    assert_eq!(
        union_expolygons(&current, &candidate),
        Ok(vec![output_rectangle(0, 0, 150, 100)])
    );
    assert_eq!(
        union_expolygons(&current, &[rectangle(200, 0, 300, 100)]),
        Ok(vec![
            output_rectangle(0, 0, 100, 100),
            output_rectangle(200, 0, 300, 100),
        ])
    );
    assert_eq!(union_expolygons(&[], &[]), Ok(Vec::new()));
    assert_eq!(
        union_expolygons(&current, &[]),
        Ok(vec![output_rectangle(0, 0, 100, 100)])
    );
}

#[test]
fn task22l_clipper_union_preserves_fixed_hole_ownership_and_order() {
    let current = vec![donut(0, 200, 50, 150)];

    assert_eq!(
        union_expolygons(&current, &[]),
        Ok(vec![ExPolygon::new(
            polygon(&[(200, 200), (0, 200), (0, 0), (200, 0)]),
            vec![polygon(&[(50, 50), (50, 150), (150, 150), (150, 50)])],
        )])
    );
}

#[test]
fn task22l_clipper_xor_uses_clockwise_hole_unchanged_as_a_solid_contour() {
    let hole = polygon(&[(20, 20), (20, 80), (80, 80), (80, 20)]);
    let original_points = hole.points().to_vec();
    assert_eq!(hole.area(), -3_600.0);

    let solid_hole = vec![ExPolygon::new(hole.clone(), Vec::new())];
    assert_eq!(xor_ex(&solid_hole, &solid_hole), Ok(Vec::new()));
    assert_eq!(hole.points(), original_points);

    assert_eq!(
        xor_ex(&[rectangle(0, 0, 40, 40)], &[rectangle(100, 0, 140, 40)]),
        Ok(vec![
            output_rectangle(0, 0, 40, 40),
            output_rectangle(100, 0, 140, 40),
        ])
    );
    assert_eq!(xor_ex(&[], &[]), Ok(Vec::new()));
}

#[test]
fn task22l_clipper_fractional_miter_offset_matches_both_coordinate_scales() {
    assert_eq!(
        offset_expolygons(
            &[rectangle(0, 0, 2_000_000, 2_000_000)],
            -285_629.6_f32,
            JoinType::Miter,
            3.0,
        ),
        Ok(vec![output_rectangle(
            285_630, 285_630, 1_714_370, 1_714_370,
        )])
    );
    assert_eq!(
        offset_expolygons(
            &[rectangle(0, 0, 200_000, 200_000)],
            -28_562.96_f32,
            JoinType::Miter,
            3.0,
        ),
        Ok(vec![output_rectangle(28_563, 28_563, 171_437, 171_437)])
    );
    assert_eq!(
        offset_expolygons(&[], -28_562.96_f32, JoinType::Miter, 3.0),
        Ok(Vec::new())
    );
}

#[test]
fn task22l_clipper_safety_difference_offsets_contour_and_hole_paths_separately() {
    let subject = vec![rectangle(0, 0, 200, 200)];
    let clip = vec![donut(50, 150, 70, 130)];

    assert_eq!(
        difference_ex_with_safety_offset(&subject, &clip),
        Ok(vec![
            ExPolygon::new(
                polygon(&[(200, 200), (0, 200), (0, 0), (200, 0)]),
                vec![polygon(&[(40, 40), (40, 160), (160, 160), (160, 40)])],
            ),
            output_rectangle(80, 80, 120, 120),
        ])
    );
}

#[test]
fn task22l_clipper_safety_difference_is_exactly_ten_at_large_coordinates() {
    let subject = vec![rectangle(1_000_000, 1_000_000, 1_000_200, 1_000_200)];
    let clip = vec![donut(1_000_050, 1_000_150, 1_000_070, 1_000_130)];

    assert_eq!(
        difference_ex_with_safety_offset(&subject, &clip),
        Ok(vec![
            ExPolygon::new(
                polygon(&[
                    (1_000_200, 1_000_200),
                    (1_000_000, 1_000_200),
                    (1_000_000, 1_000_000),
                    (1_000_200, 1_000_000),
                ]),
                vec![polygon(&[
                    (1_000_040, 1_000_040),
                    (1_000_040, 1_000_160),
                    (1_000_160, 1_000_160),
                    (1_000_160, 1_000_040),
                ])],
            ),
            output_rectangle(1_000_080, 1_000_080, 1_000_120, 1_000_120),
        ])
    );
}

#[test]
fn task22l_clipper_safety_difference_handles_overlapping_disjoint_and_empty_clips() {
    let subject = vec![rectangle(0, 0, 200, 200)];
    let overlapping = vec![rectangle(40, 40, 100, 160), rectangle(80, 40, 160, 160)];

    assert_eq!(
        difference_ex_with_safety_offset(&subject, &overlapping),
        Ok(vec![ExPolygon::new(
            polygon(&[(200, 200), (0, 200), (0, 0), (200, 0)]),
            vec![polygon(&[(30, 30), (30, 170), (170, 170), (170, 30)])],
        )])
    );
    assert_eq!(
        difference_ex_with_safety_offset(&subject, &[rectangle(300, 0, 400, 100)]),
        Ok(vec![output_rectangle(0, 0, 200, 200)])
    );
    assert_eq!(
        difference_ex_with_safety_offset(&subject, &[]),
        Ok(vec![output_rectangle(0, 0, 200, 200)])
    );
    assert_eq!(
        difference_ex_with_safety_offset(&[], &overlapping),
        Ok(Vec::new())
    );
}

#[test]
fn task22l_clipper_helpers_propagate_coordinate_errors() {
    let invalid = vec![ExPolygon::new(
        polygon(&[(HI_RANGE + 1, 0), (0, 1), (0, 2)]),
        Vec::new(),
    )];
    let near_limit = vec![rectangle(HI_RANGE - 16_384, 0, HI_RANGE - 8_192, 8_192)];
    let valid = vec![rectangle(0, 0, 100, 100)];

    assert_eq!(
        union_expolygons(&invalid, &valid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        xor_ex(&valid, &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        offset_expolygons(&near_limit, 16_384.0, JoinType::Miter, 3.0),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        difference_ex_with_safety_offset(&valid, &invalid),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
