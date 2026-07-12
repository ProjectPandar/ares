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
