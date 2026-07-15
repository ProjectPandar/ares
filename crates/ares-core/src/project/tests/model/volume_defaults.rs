use crate::project::load::selected_volume_metadata_for_test;
use crate::{Point3d, ProjectVolumeType, Transform3d, load_project};

use super::fixture::ProjectParts;

fn mesh(id: u32, name: Option<&str>) -> String {
    let name = name.map_or_else(String::new, |name| format!(r#" name="{name}""#));
    format!(
        r#"<object id="{id}"{name} type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>"#
    )
}

fn single_model(resources: &str, build: &str) -> String {
    format!(
        r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>{resources}</resources><build>{build}</build></model>"#
    )
}

#[test]
fn absent_settings_use_xml_names_and_final_object_ordinals() {
    let model = single_model(
        &format!("{}{}", mesh(10, Some("XML Name")), mesh(20, None)),
        r#"<item objectid="10"/><item objectid="10"/><item objectid="20"/>"#,
    );
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model);
    parts.set_model_settings_objects("", &[10, 10, 20]);
    let project = load_project(parts.bytes()).unwrap();

    assert_eq!(project.objects().len(), 2);
    assert_eq!(project.objects()[0].name(), "XML Name");
    assert_eq!(project.objects()[0].volumes()[0].name(), "XML Name");
    assert_eq!(project.objects()[0].instances().len(), 2);
    assert_eq!(project.objects()[1].name(), "Object_2");
    assert_eq!(project.objects()[1].volumes()[0].name(), "Object_2");
    assert!(project.documents().model_settings.objects.is_empty());
}

#[test]
fn same_index_then_first_source_match_handles_repeated_part_ids() {
    let resources = format!(
        "{}{}<object id=\"10\" type=\"model\"><components><component objectid=\"2\"/><component objectid=\"1\"/><component objectid=\"1\"/></components></object>",
        mesh(1, None),
        mesh(2, None)
    );
    let model = single_model(&resources, r#"<item objectid="10"/>"#);
    let object = r#"<object id="10"><metadata key="name" value="Configured"/>
        <part id="1" subtype="normal_part"><metadata key="name" value="First"/></part>
        <part id="1" subtype="normal_part"><metadata key="name" value="Later"/></part>
        <part id="2" subtype="normal_part"><metadata key="name" value="Two"/></part>
        <part id="999" subtype="normal_part"><metadata key="name" value="Extra"/></part>
        </object>"#;
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model);
    parts.set_model_settings_objects(object, &[10]);
    let project = load_project(parts.bytes()).unwrap();
    let object = &project.objects()[0];

    assert_eq!(
        object
            .volumes()
            .iter()
            .map(|volume| volume.id())
            .collect::<Vec<_>>(),
        [2, 1, 1]
    );
    assert_eq!(
        object
            .volumes()
            .iter()
            .map(|volume| volume.name())
            .collect::<Vec<_>>(),
        ["Two", "Later", "First"]
    );
    assert_eq!(project.documents().model_settings.objects[0].parts.len(), 4);
}

#[test]
fn unmatched_parts_default_without_mutating_source_or_component_transform() {
    let resources = format!(
        "{}<object id=\"10\" type=\"model\"><components><component objectid=\"1\" transform=\"1 0 0 0 1 0 0 0 1 4 0 0\"/></components></object>",
        mesh(1, None)
    );
    let model = single_model(&resources, r#"<item objectid="10"/>"#);
    let object = r#"<object id="10"><metadata key="name" value="Configured"/><part id="99" subtype="negative_part"><metadata key="name" value="unused"/></part></object>"#;
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model);
    parts.set_model_settings_objects(object, &[10]);
    let project = load_project(parts.bytes()).unwrap();
    let volume = &project.objects()[0].volumes()[0];

    assert_eq!(volume.id(), 1);
    assert_eq!(volume.name(), "Configured");
    assert_eq!(volume.volume_type(), ProjectVolumeType::ModelPart);
    assert!(volume.region_overrides().present_keys().is_empty());
    assert_eq!(volume.source_transform(), Transform3d::IDENTITY);
    assert_eq!(
        volume
            .transform()
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(4.0, 0.0, 0.0)
    );
    let source = &project.documents().model_settings.objects[0];
    assert_eq!(source.parts.len(), 1);
    assert_eq!(source.parts[0].id, 99);
}

#[test]
fn unnamed_counter_ignores_interleaved_explicit_names() {
    let meshes = (1..=5).map(|id| mesh(id, None)).collect::<String>();
    let components = (1..=5)
        .map(|id| format!(r#"<component objectid="{id}"/>"#))
        .collect::<String>();
    let model = single_model(
        &format!(
            "{meshes}<object id=\"10\" type=\"model\"><components>{components}</components></object>"
        ),
        r#"<item objectid="10"/>"#,
    );
    let object = r#"<object id="10"><metadata key="name" value="Widget"/>
        <part id="1" subtype="normal_part"/>
        <part id="2" subtype="normal_part"><metadata key="name" value="Explicit"/></part>
        <part id="3" subtype="normal_part"/>
        <part id="4" subtype="normal_part"><metadata key="name" value="Also"/></part>
        <part id="5" subtype="normal_part"/>
        </object>"#;
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model);
    parts.set_model_settings_objects(object, &[10]);
    let project = load_project(parts.bytes()).unwrap();

    assert_eq!(
        project.objects()[0]
            .volumes()
            .iter()
            .map(|volume| volume.name())
            .collect::<Vec<_>>(),
        ["Widget", "Explicit", "Widget_2", "Also", "Widget_3"]
    );
}

#[test]
fn private_default_selection_exposes_fixed_zero_provenance_and_statistics() {
    let selected = selected_volume_metadata_for_test(&[], 3, 77).unwrap();

    assert_eq!(selected.id, 77);
    assert!(selected.name.is_empty());
    assert_eq!(selected.volume_type, ProjectVolumeType::ModelPart);
    assert!(selected.region_overrides.present_keys().is_empty());
    assert_eq!(selected.source_transform, Transform3d::IDENTITY);
    assert!(selected.source_provenance.input_file.is_empty());
    assert_eq!(selected.source_provenance.object_index, -1);
    assert_eq!(selected.source_provenance.volume_index, -1);
    assert_eq!(selected.source_provenance.offset, [0.0; 3]);
    assert!(!selected.source_provenance.converted_from_inches);
    assert!(!selected.source_provenance.converted_from_meters);
    assert!(!selected.source_provenance.from_builtin_objects);
    assert_eq!(selected.mesh_statistics.as_array(), [0; 5]);
}

#[test]
fn private_matched_selection_projects_source_transform_provenance_and_statistics() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/model_settings.config",
        "value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"",
        "value=\"1 0 0 4 0 1 0 0 0 0 1 0 0 0 0 1\"",
    );
    parts.replace(
        "Metadata/model_settings.config",
        "</part>",
        r#"<metadata key="source_file" value="part.stl"/><metadata key="source_object_id" value="7"/><metadata key="source_volume_id" value="8"/><metadata key="source_offset_x" value="1"/><metadata key="source_offset_y" value="2"/><metadata key="source_offset_z" value="3"/><metadata key="source_in_inches" value="1"/><metadata key="source_in_meters" value="0"/><mesh_stat edges_fixed="1" degenerate_facets="2" facets_removed="3" facets_reversed="4" backwards_edges="5"/></part>"#,
    );
    let project = load_project(parts.bytes()).unwrap();
    let source = &project.documents().model_settings.objects[0].parts;
    let selected = selected_volume_metadata_for_test(source, 0, 1).unwrap();

    assert_eq!(selected.source_provenance.input_file, "part.stl");
    assert_eq!(selected.source_provenance.object_index, 7);
    assert_eq!(selected.source_provenance.volume_index, 8);
    assert_eq!(selected.source_provenance.offset, [1.0, 2.0, 3.0]);
    assert!(selected.source_provenance.converted_from_inches);
    assert!(!selected.source_provenance.converted_from_meters);
    assert_eq!(selected.mesh_statistics.as_array(), [1, 2, 3, 4, 5]);
    assert_eq!(
        selected
            .source_transform
            .transform_point(Point3d::new(0.0, 0.0, 0.0)),
        Point3d::new(4.0, 0.0, 0.0)
    );
    assert_eq!(source.len(), 1);
}

#[test]
fn invalid_matrix_errors_are_bounded_and_keyed() {
    let invalid_matrix = format!(
        "not-a-number{} 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0",
        "x".repeat(4_096)
    );
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/model_settings.config",
        "value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"",
        &format!("value=\"{invalid_matrix}\""),
    );

    let message = load_project(parts.bytes()).unwrap_err().to_string();

    assert!(message.contains("matrix"), "{message}");
    assert!(message.len() <= 512, "unbounded error: {}", message.len());
}
