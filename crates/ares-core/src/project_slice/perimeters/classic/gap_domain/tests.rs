use super::{geometry_error, prepare_pre_medial};
use crate::{
    SliceError,
    geometry::{ExPolygon, JoinType, Point, Polygon, offset2_ex},
};

#[test]
fn task22o11_empty_gaps_are_typed_none() {
    assert_eq!(prepare_pre_medial(&[], 101, 99, 80, 0.1), Ok(None));
}

#[test]
fn task22o11_bounds_keep_f64_source_operation_order() {
    let perimeter_width = 16_777_217_i64;
    let perimeter_spacing = 4_000_000_i64;
    let result = prepare_pre_medial(
        &[rectangle(0, 0, 20_000_000, 8_000_000)],
        perimeter_width,
        20_000_001,
        perimeter_spacing,
        0.1,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        result.min,
        0.2_f64 * perimeter_width as f64 * (1.0_f64 - 0.4_f64)
    );
    assert_eq!(result.max, 2.0_f64 * perimeter_spacing as f64);
    assert_ne!(
        result.min,
        f64::from(0.2_f32 * 16_777_217_f32 * (1.0_f32 - 0.4_f32))
    );
}

#[test]
fn task22o11_second_offset_cast_after_safety_addition_changes_literal_geometry() {
    let spacing = 16_777_217_i64;
    let gaps = vec![rectangle(0, 0, 100_000_000, 100_000_000)];
    let first = -(spacing as f32);
    let source_second = (spacing as f64 + 10.0_f64) as f32;
    let premature_second = spacing as f32 + 10.0_f32;
    assert_ne!(source_second, premature_second);

    let source = offset2_ex(&gaps, first, source_second, JoinType::Miter, 3.0).unwrap();
    let premature = offset2_ex(&gaps, first, premature_second, JoinType::Miter, 3.0).unwrap();
    assert_eq!(
        source,
        vec![offset_rectangle(-12, -12, 100_000_012, 100_000_012)]
    );
    assert_eq!(
        premature,
        vec![offset_rectangle(-10, -10, 100_000_010, 100_000_010)]
    );
}

#[test]
fn task22o11_narrow_gap_reaches_source_ordered_morphology_and_dp() {
    let result = prepare_pre_medial(
        &[ExPolygon::new(
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(10, 0),
                Point::new(20, 0),
                Point::new(20, 100),
                Point::new(0, 100),
            ]),
            Vec::new(),
        )],
        40,
        100,
        20,
        0.1,
    )
    .unwrap()
    .unwrap();
    assert_eq!(result.min, 4.8);
    assert_eq!(result.max, 40.0);
    assert_eq!(
        result.expolygons,
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(20, 100),
                Point::new(0, 100),
                Point::new(0, 0),
                Point::new(20, 0),
            ]),
            Vec::new(),
        )]
    );
}

#[test]
fn task22o11_domain_preserves_literal_contour_hole_geometry_order() {
    let input = ExPolygon::new(
        rectangle(0, 0, 200, 200).into_parts().0,
        vec![Polygon::new(vec![
            Point::new(15, 15),
            Point::new(15, 185),
            Point::new(185, 185),
            Point::new(185, 15),
        ])],
    );
    let result = prepare_pre_medial(&[input], 40, 100, 20, 0.1)
        .unwrap()
        .unwrap();
    assert_eq!(
        result.expolygons,
        vec![ExPolygon::new(
            Polygon::new(vec![
                Point::new(200, 200),
                Point::new(0, 200),
                Point::new(0, 0),
                Point::new(200, 0),
            ]),
            vec![Polygon::new(vec![
                Point::new(15, 15),
                Point::new(15, 185),
                Point::new(185, 185),
                Point::new(185, 15),
            ])],
        )]
    );
}

#[test]
fn task22o11_clipper_error_has_one_stable_message() {
    assert_eq!(
        prepare_pre_medial(
            &[rectangle(i64::MAX - 20, 0, i64::MAX, 20)],
            10,
            10,
            10,
            0.1
        ),
        Err(SliceError::InvalidInput(
            "Classic gap-domain geometry is outside the supported Clipper range".to_owned()
        ))
    );
    assert_eq!(
        geometry_error(crate::geometry::ClipperError::CoordinateOutOfRange),
        SliceError::InvalidInput(
            "Classic gap-domain geometry is outside the supported Clipper range".to_owned()
        )
    );
}

fn offset_rectangle(left: i64, bottom: i64, right: i64, top: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(right, top),
            Point::new(left, top),
            Point::new(left, bottom),
            Point::new(right, bottom),
        ]),
        Vec::new(),
    )
}

fn rectangle(left: i64, bottom: i64, right: i64, top: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(left, bottom),
            Point::new(right, bottom),
            Point::new(right, top),
            Point::new(left, top),
        ]),
        Vec::new(),
    )
}
