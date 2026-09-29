use super::{object_origin, sanitize_instance_name};
use crate::geometry::CoordinateScale;
use crate::{
    Point3d, ProjectInstance, ProjectMesh, ProjectObject, ProjectVolume, ProjectVolumeType,
    Transform3d,
};

#[test]
fn sanitize_matches_upstream_character_class() {
    assert_eq!(sanitize_instance_name("cube10.stl"), "cube10.stl");
    assert_eq!(sanitize_instance_name("a b!!c.stl"), "a_b_c.stl");
    assert_eq!(sanitize_instance_name("!!lead"), "lead");
    assert_eq!(sanitize_instance_name("trail!!"), "trail");
    assert_eq!(sanitize_instance_name("a:b,c\"d'e"), "a_b_c_d_e");
    assert_eq!(sanitize_instance_name("hyphen.dot/kept"), "hyphen.dot/kept");
    assert_eq!(sanitize_instance_name("__foo__"), "_foo_");
}

#[test]
fn instance_origin_preserves_positive_and_negative_half_digit_boundaries() {
    for sign in [1.0, -1.0] {
        let object = object(sign * 2.396785, 0.0);
        let origin = object_origin(&object, CoordinateScale::Normal);
        // The Rolohaun reference reaches exactly ±6013.5 before formatting.
        let position = origin.1 + sign * CoordinateScale::Normal.unscale(3_616_715);
        assert_eq!((position * 1000.0).round(), sign * 6014.0);
    }
}

#[test]
fn instance_origin_scales_translation_and_model_center_separately() {
    for sign in [1.0, -1.0] {
        let object = object(sign * 1.0000006, sign * 2.0000006);
        let origin = object_origin(&object, CoordinateScale::Normal);
        // Each 0.6-lattice-unit remainder is discarded before the addition.
        assert_eq!(origin, (sign * 3.0, sign * 3.0));
    }
}

#[test]
fn instance_origin_ignores_nonprinting_instance_translation() {
    let source = object(2.396785, 0.0);
    let object = ProjectObject::new(
        "origin.model".to_owned(),
        1,
        Default::default(),
        source.volumes().to_vec(),
        vec![
            ProjectInstance::new([1, 0, 0], false, false, Transform3d::IDENTITY),
            source.instances()[0].clone(),
        ],
    );
    let origin = object_origin(&object, CoordinateScale::Normal);
    assert_eq!(origin, (2.396785, 2.396785));
}

fn object(translation: f64, center: f64) -> ProjectObject {
    ProjectObject::new(
        "origin.model".to_owned(),
        1,
        Default::default(),
        vec![ProjectVolume::new(
            "origin.model".to_owned(),
            1,
            ProjectMesh::new(
                vec![
                    Point3d::new(-5.0, -5.0, 0.0),
                    Point3d::new(5.0, 5.0, 0.0),
                    Point3d::new(-5.0, 5.0, 1.0),
                ],
                vec![[0, 1, 2]],
            ),
            Transform3d::IDENTITY.pretranslated(Point3d::new(center, center, 0.0)),
            (
                "model".to_owned(),
                ProjectVolumeType::ModelPart,
                Default::default(),
                Transform3d::IDENTITY,
            ),
        )],
        vec![ProjectInstance::new(
            [1, 0, 0],
            true,
            false,
            Transform3d::IDENTITY.pretranslated(Point3d::new(translation, translation, 0.0)),
        )],
    )
}
