use crate::{Point3d, SliceError, project::transform::Transform3d};

#[test]
fn project_transform_maps_3mf_column_tokens_without_precision_loss() {
    let transform = Transform3d::parse_3mf("1 2 3 4 5 6 7 8 9 10.25 11.5 12.75").unwrap();

    assert_eq!(
        transform.transform_point(Point3d::new(1.0, 2.0, 3.0)),
        Point3d::new(40.25, 47.5, 54.75)
    );
}

#[test]
fn project_transform_then_means_left_matrix_times_right_matrix() {
    let build = Transform3d::parse_3mf("2 0 0 0 1 0 0 0 1 0 0 0").unwrap();
    let component = Transform3d::parse_3mf("1 0 0 0 1 0 0 0 1 10 0 0").unwrap();

    let world = build
        .then(component)
        .transform_point(Point3d::new(1.0, 0.0, 0.0));
    let reversed = component
        .then(build)
        .transform_point(Point3d::new(1.0, 0.0, 0.0));

    assert_eq!(world.x, 22.0);
    assert_eq!(reversed.x, 12.0);
    assert_ne!(world, reversed);
}

#[test]
fn project_transform_composes_build_and_component_in_orca_order() {
    let build = Transform3d::parse_3mf("2 0 0 0 1 0 0 0 1 100 0 0").unwrap();
    let component = Transform3d::parse_3mf("1 0 0 0 1 0 0 0 1 10 0 0").unwrap();

    let world = build.then(component);
    assert_eq!(
        world.transform_point(Point3d::new(1.0, 0.0, 0.0)),
        Point3d::new(122.0, 0.0, 0.0)
    );
}

#[test]
fn z_shrinkage_compensation_scales_only_the_z_row() {
    let transform = Transform3d::parse_3mf("2 0 0 0 3 0 0 0 4 100 200 5").unwrap();

    let transformed = transform
        .with_z_shrinkage_compensation(0.5)
        .transform_point(Point3d::new(2.0, 2.0, 2.0));

    assert_eq!(transformed, Point3d::new(104.0, 206.0, 6.5));
}

#[test]
fn fixed_xy_equality_ignores_z_compensation_only() {
    let transform = Transform3d::parse_3mf("2 0 0 0 3 0 0 0 4 100 200 5").unwrap();
    let z_scaled = transform.with_z_shrinkage_compensation(0.5);
    let xy_scaled = Transform3d::parse_3mf("3 0 0 0 3 0 0 0 4 100 200 5").unwrap();

    assert!(transform.fixed_xy_equal(z_scaled));
    assert!(!transform.fixed_xy_equal(xy_scaled));
}

#[test]
fn task22n_context_transform_exposes_direct_first_xy_column_bits() {
    let cases = [
        (
            "1 -0 0 0 1 0 0 0 1 10 20 30",
            [1.0_f64.to_bits(), (-0.0_f64).to_bits()],
        ),
        (
            "0 1 0 -1 0 0 0 0 1 -7 8 9",
            [0.0_f64.to_bits(), 1.0_f64.to_bits()],
        ),
    ];

    for (source, expected) in cases {
        let (m00, m10) = Transform3d::parse_3mf(source).unwrap().first_xy_column();
        assert_eq!([m00.to_bits(), m10.to_bits()], expected);
    }
}

#[test]
fn project_transform_without_xy_translation_is_value_only() {
    let original = row_major([
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    ]);
    let original_copy = original;
    let expected = row_major([
        1.0, 2.0, 3.0, 0.0, 5.0, 6.0, 7.0, 0.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    ]);

    let without_xy = original.without_xy_translation();

    assert!(without_xy.fixed_order_equal(expected));
    assert!(original.fixed_order_equal(original_copy));
}

#[test]
fn project_transform_fixed_order_uses_eigen_column_major_first_difference() {
    let lhs = row_major([
        0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]);
    let rhs = row_major([
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]);

    assert!(lhs.fixed_order_less_than(rhs));
    assert!(!rhs.fixed_order_less_than(lhs));
}

#[test]
fn project_transform_fixed_order_scans_all_16_scalars() {
    for index in 0..16 {
        let mut lhs_values = [0.0; 16];
        lhs_values[index] = -1.0;
        let lhs = row_major(lhs_values);
        let rhs = row_major([0.0; 16]);

        assert!(lhs.fixed_order_less_than(rhs), "scalar {index}");
        assert!(!rhs.fixed_order_less_than(lhs), "scalar {index}");
        assert!(!lhs.fixed_order_equal(rhs), "scalar {index}");
    }
}

#[test]
fn project_transform_fixed_order_uses_exact_equality_and_equal_signed_zeroes() {
    let positive_zero = row_major([0.0; 16]);
    let negative_zero = row_major([-0.0; 16]);

    assert!(positive_zero.fixed_order_equal(negative_zero));
    assert!(!positive_zero.fixed_order_less_than(negative_zero));
    assert!(!negative_zero.fixed_order_less_than(positive_zero));

    let mut one_values = [0.0; 16];
    one_values[15] = 1.0;
    let one = row_major(one_values);
    let mut next_values = one_values;
    next_values[15] = f64::from_bits(1.0_f64.to_bits() + 1);
    let next = row_major(next_values);

    assert!(!one.fixed_order_equal(next));
    assert!(one.fixed_order_less_than(next));
    assert!(!next.fixed_order_less_than(one));
}

#[test]
fn project_transform_composes_object_without_xy_then_volume() {
    let object = Transform3d::parse_3mf("2 0 0 0 1 0 0 0 1 100 200 3").unwrap();
    let volume = Transform3d::parse_3mf("1 0 0 0 1 0 0 0 1 10 0 0").unwrap();

    let composed = object.without_xy_translation().then(volume);
    let reversed = volume.then(object.without_xy_translation());
    let point = Point3d::new(1.0, 0.0, 0.0);

    assert_eq!(
        composed.transform_point(point),
        Point3d::new(22.0, 0.0, 3.0)
    );
    assert_eq!(
        reversed.transform_point(point),
        Point3d::new(12.0, 0.0, 3.0)
    );
    assert!(!composed.fixed_order_equal(reversed));
}

#[test]
fn project_transform_z_f32_casts_every_operand_before_arithmetic() {
    let cases = [
        (
            [16_777_217.0, 0.0, 0.0, -16_777_216.0],
            Point3d::new(1.0, 0.0, 0.0),
        ),
        (
            [0.0, 16_777_217.0, 0.0, -16_777_216.0],
            Point3d::new(0.0, 1.0, 0.0),
        ),
        (
            [0.0, 0.0, 16_777_217.0, -16_777_216.0],
            Point3d::new(0.0, 0.0, 1.0),
        ),
        (
            [-16_777_216.0, 0.0, 0.0, 16_777_217.0],
            Point3d::new(1.0, 0.0, 0.0),
        ),
        (
            [1.0, 0.0, 0.0, -16_777_216.0],
            Point3d::new(16_777_217.0, 0.0, 0.0),
        ),
        (
            [0.0, 1.0, 0.0, -16_777_216.0],
            Point3d::new(0.0, 16_777_217.0, 0.0),
        ),
        (
            [0.0, 0.0, 1.0, -16_777_216.0],
            Point3d::new(0.0, 0.0, 16_777_217.0),
        ),
    ];

    for (index, (coefficients, point)) in cases.into_iter().enumerate() {
        let transform = z_row_transform(coefficients);
        let cast_before = transform.transform_z_f32(point);
        let f64_then_cast = transform.transform_point(point).z as f32;

        assert_eq!(cast_before.to_bits(), 0.0_f32.to_bits(), "case {index}");
        assert_eq!(
            f64_then_cast.to_bits(),
            1.0_f32.to_bits(),
            "case {index} must distinguish the forbidden f64-then-cast path"
        );
    }
}

#[test]
fn project_transform_parses_row_major_part_matrix_as_provenance() {
    let part = Transform3d::parse_row_major("1 0 0 1 0 1 0 0 0 0 1 0 0 0 0 1").unwrap();

    assert_eq!(
        part.transform_point(Point3d::new(1.0, 0.0, 0.0)),
        Point3d::new(2.0, 0.0, 0.0)
    );
}

#[test]
fn project_transform_rejects_bad_12_token_inputs() {
    for value in [
        "",
        "1 0 0 0 1 0 0 0 1 0 0",
        "1 0 0 0 1 0 0 0 1 0 0 0 9",
        "1 0 0 0 1 0 0 0 1 0 nope 0",
        "1 0 0 0 1 0 0 0 1 0 NaN 0",
        "1 0 0 0 1 0 0 0 1 0 inf 0",
    ] {
        assert_invalid(Transform3d::parse_3mf(value));
    }
}

#[test]
fn project_transform_rejects_bad_16_token_inputs() {
    for value in [
        "",
        "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0",
        "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 9",
        "1 0 0 0 0 1 0 0 0 0 nope 0 0 0 0 1",
        "1 0 0 0 0 1 0 0 0 0 1 0 0 0 NaN 1",
        "1 0 0 0 0 1 0 0 0 0 1 0 0 0 -inf 1",
    ] {
        assert_invalid(Transform3d::parse_row_major(value));
    }
}

fn assert_invalid(result: Result<Transform3d, SliceError>) {
    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
}

fn row_major(values: [f64; 16]) -> Transform3d {
    let value = values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Transform3d::parse_row_major(&value).unwrap()
}

fn z_row_transform([x, y, z, translation]: [f64; 4]) -> Transform3d {
    row_major([
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        x,
        y,
        z,
        translation,
        0.0,
        0.0,
        0.0,
        1.0,
    ])
}
