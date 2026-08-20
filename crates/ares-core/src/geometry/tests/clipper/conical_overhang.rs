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
fn clipper_safety_difference_handles_overlapping_disjoint_and_empty_clips() {
    let subject = vec![rectangle(0, 0, 200, 200)];
    let overlapping = vec![rectangle(40, 40, 100, 160), rectangle(80, 40, 160, 160)];

    let result = difference_ex_with_safety_offset(&subject, &overlapping).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].contour().area().abs(), 40_000.0);
    assert_eq!(result[0].holes().len(), 1);
    assert_eq!(result[0].holes()[0].area().abs(), 19_600.0);
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
