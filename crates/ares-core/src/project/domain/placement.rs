//! `--center` placement: `Model.cpp:704-722` (`center_instances_around_point`)
//! and `OrcaSlicer.cpp:4378-4392` (Z ground drop after centering).

#[cfg(test)]
mod tests {
    use crate::project::{
        domain::{Point3d, ProjectDocuments, ProjectMesh, ProjectVolume},
        filament_sequence::FilamentSequences,
        model_settings::ModelSettings,
        raw_settings::ProjectSettingsRaw,
        slice_info::{SliceInfo, SliceInfoHeader},
    };
    use crate::{
        PlateMetadata, Project, ProjectInstance, ProjectModel, ProjectObject, ProjectSettings,
        ProjectVolumeType, Transform3d,
    };

    /// One 10 mm cube volume at [0,10]^2 x [z, z+10], one instance at `offset`.
    fn project_with_instance(offset: [f64; 3], z: f64) -> Project {
        let vertices = [
            [0.0, 0.0, z],
            [10.0, 0.0, z],
            [0.0, 10.0, z],
            [10.0, 10.0, z],
            [0.0, 0.0, z + 10.0],
            [10.0, 0.0, z + 10.0],
            [0.0, 10.0, z + 10.0],
            [10.0, 10.0, z + 10.0],
        ]
        .map(|vertex| Point3d::new(vertex[0], vertex[1], vertex[2]));
        let triangles = [
            [0, 1, 2],
            [1, 3, 2],
            [4, 6, 5],
            [5, 6, 7],
            [0, 4, 1],
            [1, 4, 5],
            [2, 3, 6],
            [3, 7, 6],
            [0, 2, 4],
            [2, 6, 4],
            [1, 5, 3],
            [3, 5, 7],
        ];
        let volume = ProjectVolume::new(
            "cube".to_owned(),
            0,
            ProjectMesh::new(vertices.to_vec(), triangles.to_vec()),
            Transform3d::IDENTITY,
            (
                "cube".to_owned(),
                ProjectVolumeType::ModelPart,
                Default::default(),
                Transform3d::IDENTITY,
            ),
        );
        let instance = ProjectInstance::new(
            [1, 0, 0],
            true,
            false,
            Transform3d::IDENTITY.pretranslated(Point3d::new(
                offset[0],
                offset[1],
                offset[2],
            )),
        );
        Project::new(
            vec![ProjectModel::new("cube".to_owned(), vec![2])],
            vec![ProjectObject::new(
                "cube".to_owned(),
                2,
                (
                    "cube".to_owned(),
                    String::new(),
                    Default::default(),
                    Default::default(),
                ),
                vec![volume],
                vec![instance],
            )],
            vec![crate::PlateMetadata::new(1, vec![[2, 0, 1]])],
            serde_json::from_str::<ProjectSettings>("{}").unwrap(),
            ProjectDocuments {
                model_settings: ModelSettings {
                    objects: Vec::new(),
                    plates: Vec::new(),
                    assemble: None,
                },
                project_settings_raw: ProjectSettingsRaw::default(),
                slice_info: SliceInfo {
                    header: SliceInfoHeader { items: Vec::new() },
                },
                filament_sequences: FilamentSequences(Default::default()),
                plate_documents: Vec::new(),
                has_painted_layer_height_profile: false,
            },
        )
    }

    #[test]
    fn centering_shifts_instance_offsets_in_xy_only() {
        let mut project = project_with_instance([30.0, 40.0, 5.0], 5.0);
        let moved = project.center_instances_around_point([118.0, 138.0]);
        assert!(moved, "off-center instances must move");
        let transform = project.objects()[0].instances()[0].transform();
        let point = transform.transform_point(Point3d::new(0.0, 0.0, 5.0));
        assert!((point.x - 113.0).abs() < 1e-9, "{point:?}");
        assert!((point.y - 133.0).abs() < 1e-9, "{point:?}");
        assert!((point.z - 5.0).abs() < 1e-9, "centering never moves Z");
    }

    #[test]
    fn centering_is_noop_within_epsilon() {
        // Model.cpp:712: |shift| < EPSILON (1e-4) leaves instances untouched.
        let mut project = project_with_instance([113.0, 133.0, 5.0], 5.0);
        let before = project.objects()[0].instances()[0].transform();
        let moved = project.center_instances_around_point([118.0, 138.0]);
        assert!(!moved, "already centered: world bbox center is (118,138)");
        assert_eq!(project.objects()[0].instances()[0].transform(), before);
    }

    #[test]
    fn drop_to_ground_subtracts_world_min_z() {
        // OrcaSlicer.cpp:4382-4392: per object, instance 0's world bbox min.z
        // is subtracted from every instance offset.
        let mut project = project_with_instance([0.0, 0.0, 5.0], 5.0);
        project.drop_instances_to_ground();
        let transform = project.objects()[0].instances()[0].transform();
        let point = transform.transform_point(Point3d::new(0.0, 0.0, 5.0));
        assert!((point.z - 0.0).abs() < 1e-9, "{point:?}");
    }
}
