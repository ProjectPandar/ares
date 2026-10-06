use crate::{Point3d, load_project};

use super::fixture::ProjectParts;

#[test]
fn single_volume_offset_moves_to_every_instance_without_changing_linear_geometry() {
    let mut parts = shifted("2 0 0 0 3 0 0 0 4 3 4 5");
    parts.add_second_instance_of_same_build_identity();
    parts.replace(
        "3D/root.model",
        "1 0 0 0 1 0 0 0 1 10 20 30",
        "0 2 0 -3 0 0 0 0 -1 10 20 30",
    );
    parts.replace(
        "3D/root.model",
        "1 0 0 0 1 0 0 0 1 40 50 60",
        "-2 0 0 0 4 0 0 0 1 40 50 60",
    );
    let project = load_project(parts.bytes()).unwrap();
    let object = &project.objects()[0];
    let volume = &object.volumes()[0];
    let origin = Point3d::new(0.0, 0.0, 0.0);
    assert_eq!(volume.transform().transform_point(origin), origin);
    assert_eq!(
        volume
            .transform()
            .transform_point(Point3d::new(1.0, 1.0, 1.0)),
        Point3d::new(2.0, 3.0, 4.0)
    );
    assert_eq!(
        object.instances()[0].transform().transform_point(origin),
        Point3d::new(-2.0, 26.0, 25.0)
    );
    assert_eq!(
        object.instances()[1].transform().transform_point(origin),
        Point3d::new(34.0, 66.0, 65.0)
    );
    assert_eq!(object.instances()[1].loaded_label_id(), Some(902));
    assert_eq!(volume.source_transform().transform_point(origin), origin);
    for (instance, expected) in object.instances().iter().zip([
        [
            Point3d::new(7.0, 22.0, 25.0),
            Point3d::new(7.0, 30.0, 25.0),
            Point3d::new(-11.0, 26.0, 25.0),
        ],
        [
            Point3d::new(38.0, 54.0, 65.0),
            Point3d::new(30.0, 54.0, 65.0),
            Point3d::new(34.0, 78.0, 65.0),
        ],
    ]) {
        let world = volume
            .mesh()
            .vertices()
            .iter()
            .map(|point| {
                instance
                    .transform()
                    .then(volume.transform())
                    .transform_point(*point)
            })
            .collect::<Vec<_>>();
        assert_eq!(world, expected);
    }
}

#[test]
fn single_volume_offset_preserves_upstream_relative_near_equal_branch() {
    for (offset, expected) in [
        ("0", 0.0),
        ("0.000000000001", 1e-12),
        ("0.00000000003", 3e-11),
    ] {
        let parts = shifted(&format!("1 0 0 0 1 0 0 0 1 {offset} 0 0"));
        let project = load_project(parts.bytes()).unwrap();
        let object = &project.objects()[0];
        assert_eq!(
            object.volumes()[0]
                .transform()
                .transform_point(Point3d::new(0.0, 0.0, 0.0))
                .x,
            expected
        );
        assert_eq!(
            object.instances()[0]
                .transform()
                .transform_point(Point3d::new(0.0, 0.0, 0.0)),
            Point3d::new(10.0, 20.0, 30.0)
        );
    }
}

#[test]
fn single_volume_offset_outside_eigen_relative_threshold_is_redistributed() {
    let project = load_project(shifted("1 0 0 0 1 0 0 0 1 0.00000000004 0 0").bytes()).unwrap();
    let object = &project.objects()[0];
    let origin = Point3d::new(0.0, 0.0, 0.0);
    assert_eq!(
        object.volumes()[0].transform().transform_point(origin),
        origin
    );
    assert_eq!(
        object.instances()[0].transform().transform_point(origin),
        Point3d::new(10.00000000004, 20.0, 30.0)
    );
}

#[test]
fn single_volume_offset_at_zero_world_origin_has_no_absolute_epsilon_fallback() {
    let mut parts = shifted("1 0 0 0 1 0 0 0 1 0.000000000001 0 0");
    parts.replace(
        "3D/root.model",
        "1 0 0 0 1 0 0 0 1 10 20 30",
        "1 0 0 0 1 0 0 0 1 0 0 0",
    );
    let project = load_project(parts.bytes()).unwrap();
    let object = &project.objects()[0];
    assert_eq!(
        object.volumes()[0]
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(0.0, 0.0, 0.0)
    );
    assert_eq!(
        object.instances()[0]
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(1e-12, 0.0, 0.0)
    );
}

#[test]
fn multiple_volume_offsets_are_not_redistributed() {
    let mut parts = shifted("1 0 0 0 1 0 0 0 1 3 4 5");
    parts.replace("3D/root.model", "</components>", r#"<component p:path="/3D/leaf.model" objectid="1" transform="1 0 0 0 1 0 0 0 1 11 13 17"/></components>"#);
    let project = load_project(parts.bytes()).unwrap();
    let object = &project.objects()[0];
    assert_eq!(object.volumes().len(), 2);
    assert_eq!(
        object.volumes()[0]
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(3.0, 4.0, 5.0)
    );
    assert_eq!(
        object.volumes()[1]
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(11.0, 13.0, 17.0)
    );
    assert_eq!(
        object.instances()[0]
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(10.0, 20.0, 30.0)
    );
}

fn shifted(transform: &str) -> ProjectParts {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/leaf.model",
        "<vertex x=\"0\" y=\"0\" z=\"0\"/>",
        "<vertex x=\"-1\" y=\"-1\" z=\"0\"/>",
    );
    parts.replace(
        "3D/leaf.model",
        "<vertex x=\"1\" y=\"0\" z=\"0\"/>",
        "<vertex x=\"1\" y=\"-1\" z=\"0\"/>",
    );
    parts.replace(
        "3D/root.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        &format!("transform=\"{transform}\""),
    );
    parts
}
