use crate::{
    ObjectOptions, Point3d, ProjectInstance, ProjectMesh, ProjectObject, ProjectSettings,
    ProjectVolume, ProjectVolumeType, SliceError, Transform3d,
    options::{ObjectOptionOverrides, RegionOptionOverrides},
    project::effective_config::types::{ResolvedPrintObjectConfig, ResolvedProjectObject},
};

use super::super::bounds::{object_height, participating_object_heights};

#[test]
fn task22a_bounds_use_first_instance_all_vertices_and_model_parts_only() {
    let source = object(
        vec![
            instance(false, z_affine(2.0, 10.0)),
            instance(true, z_affine(1.0, 1_000.0)),
        ],
        vec![
            volume(
                ProjectVolumeType::ModelPart,
                &[
                    Point3d::new(0.0, 0.0, 1.0),
                    Point3d::new(1.0, 0.0, 2.0),
                    Point3d::new(0.0, 1.0, 3.0),
                    Point3d::new(0.0, 0.0, 4.0),
                ],
                &[[0, 1, 2]],
                z_affine(3.0, 5.0),
            ),
            volume(
                ProjectVolumeType::NegativeVolume,
                &[Point3d::new(0.0, 0.0, 10_000.0)],
                &[],
                Transform3d::IDENTITY,
            ),
            volume(
                ProjectVolumeType::ParameterModifier,
                &[Point3d::new(0.0, 0.0, 20_000.0)],
                &[],
                Transform3d::IDENTITY,
            ),
            volume(
                ProjectVolumeType::SupportEnforcer,
                &[Point3d::new(0.0, 0.0, 30_000.0)],
                &[],
                Transform3d::IDENTITY,
            ),
            volume(
                ProjectVolumeType::SupportBlocker,
                &[Point3d::new(0.0, 0.0, 40_000.0)],
                &[],
                Transform3d::IDENTITY,
            ),
        ],
    );
    assert_eq!(object_height(&source), Ok(44.0));

    let two_model_parts = object(
        vec![instance(true, Transform3d::IDENTITY)],
        vec![
            volume(
                ProjectVolumeType::ModelPart,
                &[Point3d::new(0.0, 0.0, 2.0)],
                &[],
                Transform3d::IDENTITY,
            ),
            volume(
                ProjectVolumeType::ModelPart,
                &[Point3d::new(0.0, 0.0, 11.0)],
                &[],
                Transform3d::IDENTITY,
            ),
        ],
    );
    assert_eq!(object_height(&two_model_parts), Ok(11.0));

    let vertex_only = object(
        vec![instance(true, Transform3d::IDENTITY)],
        vec![volume(
            ProjectVolumeType::ModelPart,
            &[Point3d::new(0.0, 0.0, 7.0)],
            &[],
            Transform3d::IDENTITY,
        )],
    );
    assert_eq!(object_height(&vertex_only), Ok(7.0));
}

#[test]
fn task22a_bounds_reuse_source_height_across_transform_groups() {
    let source = object(
        vec![
            instance(false, z_affine(2.0, 10.0)),
            instance(true, z_affine(1.0, 1_000.0)),
        ],
        vec![volume(
            ProjectVolumeType::ModelPart,
            &[Point3d::new(0.0, 0.0, 5.0)],
            &[],
            Transform3d::IDENTITY,
        )],
    );
    let resolved = resolved(0, &[z_affine(1.0, 1_000.0), z_affine(0.5, -25.0)]);

    assert_eq!(
        participating_object_heights(&[source], &[resolved]),
        Ok(vec![20.0])
    );
}

#[test]
fn task22a_bounds_accept_negative_or_nonzero_min_without_normalization() {
    let negative_min = object(
        vec![instance(true, Transform3d::IDENTITY)],
        vec![volume(
            ProjectVolumeType::ModelPart,
            &[Point3d::new(0.0, 0.0, -20.0), Point3d::new(0.0, 0.0, 3.0)],
            &[],
            Transform3d::IDENTITY,
        )],
    );
    let nonzero_min = object(
        vec![instance(true, Transform3d::IDENTITY)],
        vec![volume(
            ProjectVolumeType::ModelPart,
            &[Point3d::new(0.0, 0.0, 5.0), Point3d::new(0.0, 0.0, 9.0)],
            &[],
            Transform3d::IDENTITY,
        )],
    );

    assert_eq!(object_height(&negative_min), Ok(3.0));
    assert_eq!(object_height(&nonzero_min), Ok(9.0));
}

#[test]
fn task22a_bounds_reject_empty_nonpositive_and_nonfinite_results() {
    let cases = [
        object(
            vec![instance(true, Transform3d::IDENTITY)],
            vec![volume(
                ProjectVolumeType::NegativeVolume,
                &[Point3d::new(0.0, 0.0, 10.0)],
                &[],
                Transform3d::IDENTITY,
            )],
        ),
        object(
            vec![instance(true, Transform3d::IDENTITY)],
            vec![volume(
                ProjectVolumeType::ModelPart,
                &[Point3d::new(0.0, 0.0, -3.0), Point3d::new(0.0, 0.0, -1.0)],
                &[],
                Transform3d::IDENTITY,
            )],
        ),
        object(
            vec![instance(true, Transform3d::IDENTITY)],
            vec![volume(
                ProjectVolumeType::ModelPart,
                &[Point3d::new(0.0, 0.0, 0.0)],
                &[],
                Transform3d::IDENTITY,
            )],
        ),
        object(
            vec![instance(true, Transform3d::IDENTITY)],
            vec![volume(
                ProjectVolumeType::ModelPart,
                &[Point3d::new(0.0, 0.0, 1.0), Point3d::new(0.0, 0.0, 2.0)],
                &[],
                z_affine(f64::MAX, 0.0),
            )],
        ),
        object(
            vec![instance(true, Transform3d::IDENTITY)],
            vec![volume(
                ProjectVolumeType::ModelPart,
                &[Point3d::new(1.0, 0.0, 1.0), Point3d::new(2.0, 0.0, 1.0)],
                &[],
                x_scale(f64::MAX),
            )],
        ),
    ];

    for source in cases {
        assert_eq!(
            object_height(&source),
            Err(SliceError::InvalidInput(
                "project-object Z bounds require finite model-part vertices and a positive maximum Z"
                    .to_owned(),
            ))
        );
    }
}

fn object(instances: Vec<ProjectInstance>, volumes: Vec<ProjectVolume>) -> ProjectObject {
    ProjectObject::new(
        "synthetic.model".to_owned(),
        1,
        (
            "object".to_owned(),
            String::new(),
            ObjectOptionOverrides::default(),
            RegionOptionOverrides::default(),
        ),
        volumes,
        instances,
    )
}

fn instance(printable: bool, transform: Transform3d) -> ProjectInstance {
    ProjectInstance::new([1, 0, 1_000], printable, false, transform)
}

fn volume(
    volume_type: ProjectVolumeType,
    vertices: &[Point3d],
    triangles: &[[u32; 3]],
    transform: Transform3d,
) -> ProjectVolume {
    ProjectVolume::new(
        "synthetic.model".to_owned(),
        1,
        ProjectMesh::new(vertices.to_vec(), triangles.to_vec()),
        transform,
        (
            "volume".to_owned(),
            volume_type,
            RegionOptionOverrides::default(),
            Transform3d::IDENTITY,
        ),
    )
}

fn resolved(source_object_index: usize, transforms: &[Transform3d]) -> ResolvedProjectObject {
    ResolvedProjectObject {
        source_object_index,
        object: ObjectOptions::from_base(&ProjectSettings::default().process.object),
        print_objects: transforms
            .iter()
            .copied()
            .map(|transform| ResolvedPrintObjectConfig { transform })
            .collect(),
        layer_candidates: Vec::new(),
    }
}

fn z_affine(scale: f64, offset: f64) -> Transform3d {
    Transform3d::parse_row_major(&format!("1 0 0 0 0 1 0 0 0 0 {scale} {offset} 0 0 0 1")).unwrap()
}

fn x_scale(scale: f64) -> Transform3d {
    Transform3d::parse_row_major(&format!("{scale} 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1")).unwrap()
}
