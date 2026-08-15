use crate::project::{ArchiveLimits, PackagePath, ProjectArchive};
use crate::{
    GenerationMetadata, ORCA_SLICER_COMPATIBILITY_VERSION, Point3d, ProjectSettings, SliceError,
    load_model, load_project, slice_project,
};

use super::fixture::{FIXTURE, ProjectParts};

#[test]
fn project_import_loads_valid_synthetic_project() {
    let project = load_project(ProjectParts::valid().bytes()).unwrap();

    assert_eq!(project.objects().len(), 1);
    assert_eq!(project.objects()[0].id(), 2);
}

#[test]
fn project_import_retains_distinct_path_qualified_object_and_volume_identity() {
    let mut parts = ProjectParts::valid();
    parts.use_distinct_object_ids_across_build_paths();
    let project = load_project(parts.bytes()).unwrap();

    assert_eq!(project.objects().len(), 2);
    assert_eq!(project.objects()[0].source_model_path(), "3D/a.model");
    assert_eq!(project.objects()[1].source_model_path(), "3D/b.model");
    assert_eq!(
        project.objects()[0].volumes()[0].source_model_path(),
        "3D/a.model"
    );
    assert_eq!(
        project.objects()[0].volumes()[0]
            .transform()
            .transform_point(project.objects()[0].volumes()[0].mesh().vertices()[1])
            .x,
        1.0
    );
    assert_eq!(
        project.objects()[1].volumes()[0]
            .transform()
            .transform_point(project.objects()[1].volumes()[0].mesh().vertices()[1])
            .x,
        9.0
    );
}

#[test]
fn project_import_groups_multiple_instances_only_for_the_same_path_identity() {
    let mut parts = ProjectParts::valid();
    parts.add_second_instance_of_same_build_identity();
    let project = load_project(parts.bytes()).unwrap();

    assert_eq!(project.objects().len(), 1);
    assert_eq!(project.objects()[0].source_model_path(), "3D/root.model");
    assert_eq!(project.objects()[0].instances().len(), 2);
    assert_eq!(project.objects()[0].instances()[0].loaded_label_id(), 133);
    assert_eq!(project.objects()[0].instances()[1].loaded_label_id(), 902);
}

#[test]
fn project_import_loads_fixture_identity_transforms_and_world_bounds() {
    let project = load_project(FIXTURE).unwrap();

    assert_eq!(project.models().len(), 2);
    assert_eq!(project.models()[0].path(), "3D/3dmodel.model");
    assert_eq!(project.models()[0].object_ids(), &[2]);
    assert_eq!(project.models()[1].object_ids(), &[1]);
    assert_eq!(project.objects().len(), 1);

    let object = &project.objects()[0];
    assert_eq!(object.id(), 2);
    assert_eq!(object.volumes().len(), 1);
    assert_eq!(object.instances().len(), 1);
    let volume = &object.volumes()[0];
    assert_eq!(volume.id(), 1);
    assert_eq!(volume.mesh().vertices().len(), 6_109);
    assert_eq!(volume.mesh().triangles().len(), 12_234);
    let instance = &object.instances()[0];
    assert_eq!(instance.object_id(), 2);
    assert_eq!(instance.instance_id(), 0);
    assert_eq!(instance.loaded_label_id(), 133);
    assert!(instance.printable());
    assert!(instance.auto_drop());

    let bounds = volume
        .mesh()
        .vertices()
        .iter()
        .map(|point| {
            instance
                .transform()
                .then(volume.transform())
                .transform_point(*point)
        })
        .fold(
            [
                Point3d::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
                Point3d::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            ],
            |mut bounds, point| {
                bounds[0].x = bounds[0].x.min(point.x);
                bounds[0].y = bounds[0].y.min(point.y);
                bounds[0].z = bounds[0].z.min(point.z);
                bounds[1].x = bounds[1].x.max(point.x);
                bounds[1].y = bounds[1].y.max(point.y);
                bounds[1].z = bounds[1].z.max(point.z);
                bounds
            },
        );
    assert_point_approx(bounds[0], Point3d::new(95.539205, 80.992105, 0.0));
    assert_point_approx(bounds[1], Point3d::new(170.539205, 150.992105, 92.0));

    assert_eq!(project.plates().len(), 1);
    assert_eq!(project.plates()[0].id(), 1);
    assert_eq!(project.plates()[0].instances(), &[[2, 0, 133]]);
    assert_eq!(project.settings().metadata.name, "project_settings");
}

#[test]
fn project_import_keeps_assembly_pose_and_source_offsets_out_of_world_transform() {
    let project = load_project(FIXTURE).unwrap();
    let object = &project.objects()[0];
    let volume = &object.volumes()[0];
    let instance = &object.instances()[0];

    let first = instance
        .transform()
        .then(volume.transform())
        .transform_point(volume.mesh().vertices()[0]);
    assert_eq!(first.z, 0.5);
}

#[test]
fn project_import_composes_noncommuting_build_and_component_transforms() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/root.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"1 0 0 0 1 0 0 0 1 10 0 0\"",
    );
    parts.replace(
        "3D/root.model",
        "transform=\"1 0 0 0 1 0 0 0 1 10 20 30\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let project = load_project(parts.bytes()).unwrap();
    let object = &project.objects()[0];
    let volume = &object.volumes()[0];
    let instance = &object.instances()[0];
    let point = instance
        .transform()
        .then(volume.transform())
        .transform_point(volume.mesh().vertices()[1]);

    assert_eq!(point.x, 22.0);
    assert_ne!(point.x, 12.0);
}

#[test]
fn project_import_preserves_loaded_label_and_false_instance_flags() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/root.model",
        "printable=\"1\" auto_drop=\"1\"",
        "printable=\"0\" auto_drop=\"0\"",
    );
    parts.replace(
        "Metadata/model_settings.config",
        "key=\"identify_id\" value=\"133\"",
        "key=\"identify_id\" value=\"901\"",
    );
    let project = load_project(parts.bytes()).unwrap();
    let instance = &project.objects()[0].instances()[0];

    assert_eq!(instance.loaded_label_id(), 901);
    assert!(!instance.printable());
    assert!(!instance.auto_drop());
}

#[test]
fn project_import_keeps_part_matrix_as_source_provenance_only() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/model_settings.config",
        "value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"",
        "value=\"1 0 0 4 0 1 0 0 0 0 1 0 0 0 0 1\"",
    );
    let project = load_project(parts.bytes()).unwrap();
    let volume = &project.objects()[0].volumes()[0];
    let origin = Point3d::new(0.0, 0.0, 0.0);

    assert_eq!(
        volume
            .transform()
            .transform_point(volume.mesh().vertices()[0]),
        origin
    );
    assert_eq!(
        volume.source_transform().transform_point(origin),
        Point3d::new(4.0, 0.0, 0.0)
    );
}

#[test]
fn project_import_accepts_supported_production_namespace_alias() {
    let mut parts = ProjectParts::valid();
    for path in ["3D/root.model", "3D/leaf.model"] {
        parts.replace(path, "xmlns:p=", "xmlns:q=");
        parts.replace(path, "requiredextensions=\"p\"", "requiredextensions=\"q\"");
    }
    parts.replace("3D/root.model", "p:path=", "q:path=");

    assert!(load_project(parts.bytes()).is_ok());
}

#[test]
fn project_import_iterates_multiple_model_relationship_targets() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/_rels/root.model.rels",
        "</Relationships>",
        r#" <Relationship Target="/3D/leaf2.model" Id="r2" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
    );
    parts.insert_text("3D/leaf2.model", super::fixture::LEAF_MODEL);
    let project = load_project(parts.bytes()).unwrap();

    assert_eq!(project.models().len(), 3);
    assert_eq!(project.models()[2].path(), "3D/leaf2.model");
}

#[test]
fn project_import_accepts_fixed_upstream_other_object_type() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/leaf.model",
        "id=\"1\" type=\"model\"",
        "id=\"1\" type=\"other\"",
    );

    assert!(load_project(parts.bytes()).is_ok());
}

#[test]
fn project_import_retains_typed_settings_and_task3_documents() {
    let project = load_project(FIXTURE).unwrap();
    let documents = project.documents();

    assert_eq!(documents.model_settings.objects[0].id, 2);
    assert_eq!(documents.slice_info.header.items[2].value, "2.4.2");
    assert_eq!(documents.filament_sequences.0.len(), 1);
    assert_eq!(documents.plate_documents[0].version, 2);

    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    let expected: ProjectSettings = serde_json::from_slice(
        &archive
            .read(&PackagePath::entry(b"Metadata/project_settings.config").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(project.settings(), &expected);
}

#[tokio::test]
async fn project_import_slice_project_emits_after_typed_loading() {
    assert_eq!(ORCA_SLICER_COMPATIBILITY_VERSION, "2.4.2");
    let metadata = GenerationMetadata::deterministic(2026, 7, 12, 3, 4, 5);
    let output = slice_project(FIXTURE, metadata).await.unwrap();
    assert!(output.starts_with(b"; HEADER_BLOCK_START\n; generated by Ares"));

    let error = slice_project([], metadata).await.unwrap_err();
    assert_ne!(error, SliceError::ProjectSlicingIncomplete);

    let mut invalid = ProjectParts::valid();
    invalid.remove("3D/leaf.model");
    let invalid = invalid.bytes();
    let load_error = load_project(&invalid).unwrap_err();
    assert_eq!(
        slice_project(&invalid, metadata).await.unwrap_err(),
        load_error
    );
}

#[test]
fn project_import_generation_metadata_validates_local_calendar_fields() {
    assert!(GenerationMetadata::new_local(2026, 7, 12, 23, 59, 59).is_ok());
    for fields in [
        (2026, 0, 12, 23, 59, 59),
        (2026, 13, 12, 23, 59, 59),
        (2026, 7, 0, 23, 59, 59),
        (2026, 7, 32, 23, 59, 59),
        (2026, 7, 12, 24, 59, 59),
        (2026, 7, 12, 23, 60, 59),
        (2026, 7, 12, 23, 59, 60),
    ] {
        assert!(
            GenerationMetadata::new_local(
                fields.0, fields.1, fields.2, fields.3, fields.4, fields.5
            )
            .is_err()
        );
    }
    assert!(GenerationMetadata::new_local(2024, 2, 29, 0, 0, 0).is_ok());
    assert!(GenerationMetadata::new_local(2025, 2, 29, 0, 0, 0).is_err());
    assert!(GenerationMetadata::new_local(2026, 4, 31, 0, 0, 0).is_err());
}

#[test]
fn project_import_legacy_model_loader_rejects_project_boundary() {
    let error = load_model(FIXTURE).unwrap_err().to_string();
    assert!(error.contains("load_project"));
}

fn assert_point_approx(actual: Point3d, expected: Point3d) {
    assert!((actual.x - expected.x).abs() < 1e-12);
    assert!((actual.y - expected.y).abs() < 1e-12);
    assert!((actual.z - expected.z).abs() < 1e-12);
}
