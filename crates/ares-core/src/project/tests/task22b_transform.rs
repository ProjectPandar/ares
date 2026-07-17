use crate::{Point3d, project::transform::Transform3d};

#[test]
fn task22b_transform_removes_xyz_translation() {
    let transform = row_major([
        0.0, -2.0, 0.0, 11.0, 3.0, 0.0, 0.0, 13.0, 0.0, 0.0, 4.0, 17.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let expected = row_major([
        0.0, -2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);

    assert!(transform.without_translation().fixed_order_equal(expected));
    assert!(
        !transform
            .without_xy_translation()
            .fixed_order_equal(expected)
    );
}

#[test]
fn task22b_transform_local_translation_is_c_times_t() {
    let transform = row_major([
        0.0, -2.0, 0.0, 5.0, 3.0, 0.0, 0.0, 7.0, 0.0, 0.0, 4.0, 11.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let shift = Point3d::new(13.0, 17.0, 19.0);
    let translation = translation(shift);

    let translated = transform.translated(shift);

    assert!(translated.fixed_order_equal(transform.then(translation)));
    assert!(!translated.fixed_order_equal(translation.then(transform)));
    assert_eq!(
        translated.transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(-29.0, 46.0, 87.0)
    );
}

#[test]
fn task22b_transform_pretranslation_acts_after_linear() {
    let transform = row_major([
        0.0, -2.0, 0.0, 5.0, 3.0, 0.0, 0.0, 7.0, 0.0, 0.0, 4.0, 11.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let shift = Point3d::new(13.0, 17.0, 19.0);
    let translation = translation(shift);

    let pretranslated = transform.pretranslated(shift);

    assert!(pretranslated.fixed_order_equal(translation.then(transform)));
    assert!(!pretranslated.fixed_order_equal(transform.then(translation)));
    assert_eq!(
        pretranslated.transform_point(Point3d::new(2.0, 3.0, 5.0)),
        Point3d::new(12.0, 30.0, 50.0)
    );
}

#[test]
fn task22b_transform_casts_matrix_and_point_before_f32_arithmetic() {
    let matrix_discriminator = row_major([
        16_777_217.0,
        0.0,
        0.0,
        -16_777_216.0,
        0.0,
        16_777_217.0,
        0.0,
        -16_777_216.0,
        0.0,
        0.0,
        16_777_217.0,
        -16_777_216.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]);
    let point = Point3d::new(1.0, 1.0, 1.0);

    let matrix_cast_before = matrix_discriminator.transform_point_f32(point);
    let matrix_f64_then_cast = matrix_discriminator.transform_point(point);

    assert_eq!(matrix_cast_before.map(f32::to_bits), [0.0_f32.to_bits(); 3]);
    assert_eq!(
        [
            (matrix_f64_then_cast.x as f32).to_bits(),
            (matrix_f64_then_cast.y as f32).to_bits(),
            (matrix_f64_then_cast.z as f32).to_bits(),
        ],
        [1.0_f32.to_bits(); 3]
    );

    let point_discriminator = row_major([
        1.0,
        0.0,
        0.0,
        -16_777_216.0,
        0.0,
        1.0,
        0.0,
        -16_777_216.0,
        0.0,
        0.0,
        1.0,
        -16_777_216.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]);
    let wide_point = Point3d::new(16_777_217.0, 16_777_217.0, 16_777_217.0);

    let point_cast_before = point_discriminator.transform_point_f32(wide_point);
    let point_f64_then_cast = point_discriminator.transform_point(wide_point);

    assert_eq!(point_cast_before.map(f32::to_bits), [0.0_f32.to_bits(); 3]);
    assert_eq!(
        [
            (point_f64_then_cast.x as f32).to_bits(),
            (point_f64_then_cast.y as f32).to_bits(),
            (point_f64_then_cast.z as f32).to_bits(),
        ],
        [1.0_f32.to_bits(); 3]
    );
}

fn translation(point: Point3d) -> Transform3d {
    row_major([
        1.0, 0.0, 0.0, point.x, 0.0, 1.0, 0.0, point.y, 0.0, 0.0, 1.0, point.z, 0.0, 0.0, 0.0, 1.0,
    ])
}

fn row_major(values: [f64; 16]) -> Transform3d {
    let value = values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Transform3d::parse_row_major(&value).unwrap()
}
