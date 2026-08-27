use crate::load_project;

use super::fixture::{LEAF_MODEL, ProjectParts, ROOT_MODEL};

#[test]
fn project_import_rejects_non_finite_vertex_and_triangle_index_at_vertex_count() {
    for value in ["NaN", "inf", "-inf"] {
        let mut parts = ProjectParts::valid();
        parts.replace("3D/leaf.model", "x=\"0\"", &format!("x=\"{value}\""));
        assert!(load_project(parts.bytes()).is_err());
    }

    let mut parts = ProjectParts::valid();
    parts.replace("3D/leaf.model", "v3=\"2\"", "v3=\"3\"");
    assert!(load_project(parts.bytes()).is_err());
}

#[test]
fn project_import_rejects_duplicate_part_identity_and_invalid_object_shape() {
    let duplicate = r#"<object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>"#;
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/leaf.model",
        "</resources>",
        &format!("{duplicate}</resources>"),
    );
    assert!(load_project(parts.bytes()).is_err());

    let mut both = ProjectParts::valid();
    both.replace(
        "3D/leaf.model",
        "</mesh></object>",
        "</mesh><components><component objectid=\"1\"/></components></object>",
    );
    assert!(load_project(both.bytes()).is_err());

    let mut neither = ProjectParts::valid();
    neither.insert_text(
        "3D/leaf.model",
        r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"/></resources><build/></model>"#,
    );
    assert!(load_project(neither.bytes()).is_err());
}

#[test]
fn project_import_still_rejects_duplicate_retained_part_matrix() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/model_settings.config",
        r#"<metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>"#,
        r#"<metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>"#,
    );

    let error = load_project(parts.bytes()).unwrap_err().to_string();
    assert!(error.contains("repeated metadata \"matrix\""), "{error}");
}

#[test]
fn project_import_rejects_missing_or_wrong_relationship_targets() {
    let mut missing_root = ProjectParts::valid();
    missing_root.replace("_rels/.rels", "/3D/root.model", "/3D/missing.model");
    assert!(load_project(missing_root.bytes()).is_err());

    let mut missing_model = ProjectParts::valid();
    missing_model.remove("3D/leaf.model");
    assert!(load_project(missing_model.bytes()).is_err());

    let mut wrong_type = ProjectParts::valid();
    wrong_type.replace(
        "[Content_Types].xml",
        "</Types>",
        r#"<Override PartName="/3D/leaf.model" ContentType="application/octet-stream"/></Types>"#,
    );
    assert!(load_project(wrong_type.bytes()).is_err());
}

#[test]
fn project_import_tolerates_missing_declared_root_preview_targets() {
    // OrcaSlicer's exporter references preview parts unconditionally and its
    // loader ignores missing parts, so CLI exports must load.
    for (from, to) in [
        (
            r#"Target="/Metadata/plate_1.png" Id="rel-2""#,
            r#"Target="/Metadata/missing.png" Id="rel-2""#,
        ),
        (
            r#"Target="/Metadata/plate_1.png" Id="rel-4""#,
            r#"Target="/Metadata/missing.png" Id="rel-4""#,
        ),
        (
            r#"Target="/Metadata/plate_1_small.png" Id="rel-5""#,
            r#"Target="/Metadata/missing.png" Id="rel-5""#,
        ),
    ] {
        let mut parts = ProjectParts::fixture();
        parts.replace("_rels/.rels", from, to);
        assert!(load_project(parts.bytes()).is_ok(), "rejected {from}");
    }
}

#[test]
fn project_import_rejects_wrong_preview_mime_but_tolerates_missing_previews() {
    let mut wrong_mime = ProjectParts::fixture();
    wrong_mime.replace(
        "[Content_Types].xml",
        "</Types>",
        r#"<Override PartName="/Metadata/plate_1_small.png" ContentType="application/octet-stream"/></Types>"#,
    );
    assert!(load_project(wrong_mime.bytes()).is_err());

    // OrcaSlicer references plate previews unconditionally and loads
    // projects whose preview parts are absent (CLI exports).
    let mut missing_pick = ProjectParts::fixture();
    missing_pick.remove("Metadata/pick_1.png");
    assert!(load_project(missing_pick.bytes()).is_ok());
}

#[test]
fn project_import_rejects_wrong_relationship_part_content_type() {
    for path in ["/_rels/.rels", "/3D/_rels/3dmodel.model.rels"] {
        let mut parts = ProjectParts::fixture();
        parts.replace(
            "[Content_Types].xml",
            "</Types>",
            &format!(
                r#"<Override PartName="{path}" ContentType="application/octet-stream"/></Types>"#
            ),
        );
        assert!(load_project(parts.bytes()).is_err(), "accepted {path}");
    }

    let mut unreferenced = ProjectParts::valid();
    unreferenced.insert_text("3D/unreferenced.model", LEAF_MODEL);
    unreferenced.insert_text(
        "3D/_rels/unreferenced.model.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
    );
    unreferenced.replace(
        "[Content_Types].xml",
        "</Types>",
        r#"<Override PartName="/3D/_rels/unreferenced.model.rels" ContentType="application/octet-stream"/></Types>"#,
    );
    assert!(load_project(unreferenced.bytes()).is_err());
}

#[test]
fn project_import_rejects_wrong_root_or_model_relationship_type() {
    for path in ["_rels/.rels", "3D/_rels/root.model.rels"] {
        let mut parts = ProjectParts::valid();
        parts.replace(
            path,
            "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel",
            "https://example.invalid/not-a-model-relationship",
        );
        assert!(load_project(parts.bytes()).is_err());
    }
}

#[test]
fn project_import_rejects_duplicate_root_relationship_ids_across_types() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "_rels/.rels",
        "</Relationships>",
        r#"<Relationship Target="/Metadata/plate_1.png" Id="r1" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"/></Relationships>"#,
    );

    let error = load_project(parts.bytes()).unwrap_err().to_string();
    assert!(
        error.contains("duplicate relationship ID \"r1\""),
        "{error}"
    );
    assert!(error.contains("package root"), "{error}");
}

#[test]
fn project_import_rejects_duplicate_reachable_model_relationship_ids() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/_rels/root.model.rels",
        "</Relationships>",
        r#"<Relationship Target="/3D/leaf2.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
    );
    parts.insert_text("3D/leaf2.model", LEAF_MODEL);

    let error = load_project(parts.bytes()).unwrap_err().to_string();
    assert!(
        error.contains("duplicate relationship ID \"r1\""),
        "{error}"
    );
    assert!(error.contains("3D/root.model"), "{error}");
}

#[test]
fn project_import_rejects_unauthorized_and_wrong_owner_models_but_ignores_extra_entries() {
    let mut unauthorized = ProjectParts::valid();
    unauthorized.replace("3D/root.model", "/3D/leaf.model", "/3D/other.model");
    unauthorized.insert_text("3D/other.model", LEAF_MODEL);
    assert!(load_project(unauthorized.bytes()).is_err());

    let mut unreferenced = ProjectParts::valid();
    unreferenced.insert_text("3D/unreferenced.model", LEAF_MODEL);
    assert!(load_project(unreferenced.bytes()).is_ok());

    let mut unreferenced_with_relationships = ProjectParts::valid();
    unreferenced_with_relationships.insert_text("3D/unreferenced.model", LEAF_MODEL);
    unreferenced_with_relationships.insert_text(
        "3D/_rels/unreferenced.model.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
    );
    assert!(load_project(unreferenced_with_relationships.bytes()).is_ok());

    let mut wrong_owner = ProjectParts::valid();
    wrong_owner.remove("3D/_rels/root.model.rels");
    wrong_owner.insert_text(
        "Wrong/_rels/root.model.rels",
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/leaf.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
    );
    assert!(load_project(wrong_owner.bytes()).is_err());
}

#[test]
fn project_import_rejects_required_extension_and_production_binding_errors() {
    let mut unknown = ProjectParts::valid();
    unknown.replace(
        "3D/root.model",
        "requiredextensions=\"p\"",
        "requiredextensions=\"p q\"",
    );
    assert!(load_project(unknown.bytes()).is_err());

    let mut wrong_uri = ProjectParts::valid();
    wrong_uri.replace(
        "3D/root.model",
        "http://schemas.microsoft.com/3dmanufacturing/production/2015/06",
        "https://example.invalid/production",
    );
    assert!(load_project(wrong_uri.bytes()).is_err());

    let mut missing_binding = ProjectParts::valid();
    missing_binding.replace(
        "3D/root.model",
        " xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\"",
        "",
    );
    assert!(load_project(missing_binding.bytes()).is_err());
}

#[test]
fn project_import_rejects_wrong_namespace_local_name_spoofing() {
    for attribute in ["path", "UUID"] {
        let mut parts = ProjectParts::valid();
        parts.replace(
            "3D/root.model",
            "requiredextensions=\"p\"",
            "xmlns:evil=\"https://example.invalid/production\" requiredextensions=\"p\"",
        );
        if attribute == "path" {
            parts.replace("3D/root.model", "p:path=", "evil:path=");
        } else {
            parts.replace(
                "3D/root.model",
                "<object id=\"2\" type=\"model\"",
                "<object id=\"2\" type=\"model\" evil:UUID=\"spoofed\"",
            );
        }
        assert!(load_project(parts.bytes()).is_err());
    }
}

#[test]
fn project_import_rejects_prefixed_typed_attributes_for_every_xml_role() {
    let mutations = [
        (
            "[Content_Types].xml",
            "<Types xmlns=",
            "<Types xmlns:evil=\"https://example.invalid/spoof\" xmlns=",
            "Extension=\"rels\" ContentType=",
            "evil:Extension=\"rels\" evil:ContentType=",
        ),
        (
            "_rels/.rels",
            "<Relationships xmlns=",
            "<Relationships xmlns:evil=\"https://example.invalid/spoof\" xmlns=",
            "Target=\"/3D/root.model\" Id=\"r1\" Type=",
            "evil:Target=\"/3D/root.model\" evil:Id=\"r1\" evil:Type=",
        ),
        (
            "Metadata/model_settings.config",
            "<config>",
            "<config xmlns:evil=\"https://example.invalid/spoof\">",
            "key=\"identify_id\" value=\"133\"",
            "evil:key=\"identify_id\" evil:value=\"133\"",
        ),
        (
            "Metadata/slice_info.config",
            "<config>",
            "<config xmlns:evil=\"https://example.invalid/spoof\">",
            "key=\"OrcaSlicer-Version\" value=\"2.4.2\"",
            "evil:key=\"OrcaSlicer-Version\" evil:value=\"2.4.2\"",
        ),
    ];
    for (path, root_from, root_to, attribute_from, attribute_to) in mutations {
        let mut parts = ProjectParts::valid();
        parts.replace(path, root_from, root_to);
        parts.replace(path, attribute_from, attribute_to);
        assert!(
            load_project(parts.bytes()).is_err(),
            "accepted spoof in {path}"
        );
    }
}

#[test]
fn project_import_rejects_ambiguous_bare_metadata_for_reused_object_id_paths() {
    let mut parts = ProjectParts::valid();
    parts.reuse_object_id_across_build_paths();

    let error = load_project(parts.bytes()).unwrap_err().to_string();

    assert!(error.contains("ambiguous object ID 1"), "{error}");
    assert!(error.contains("3D/a.model"), "{error}");
    assert!(error.contains("3D/b.model"), "{error}");
}

#[test]
fn project_import_rejects_missing_build_component_settings_plate_and_assemble_references() {
    for (path, from, to) in [
        (
            "3D/root.model",
            "<item objectid=\"2\"",
            "<item objectid=\"99\"",
        ),
        (
            "3D/root.model",
            "component p:path=\"/3D/leaf.model\" objectid=\"1\"",
            "component p:path=\"/3D/leaf.model\" objectid=\"99\"",
        ),
        (
            "Metadata/model_settings.config",
            "key=\"object_id\" value=\"2\"",
            "key=\"object_id\" value=\"99\"",
        ),
        (
            "Metadata/model_settings.config",
            "assemble_item object_id=\"2\"",
            "assemble_item object_id=\"99\"",
        ),
        (
            "Metadata/model_settings.config",
            "instance_id=\"0\" transform",
            "instance_id=\"1\" transform",
        ),
    ] {
        let mut parts = ProjectParts::valid();
        parts.replace(path, from, to);
        assert!(
            load_project(parts.bytes()).is_err(),
            "accepted mutation {from:?}"
        );
    }
}

#[test]
fn project_import_rejects_missing_component_in_unused_reachable_object() {
    let mut parts = ProjectParts::valid();
    parts.replace(
        "3D/leaf.model",
        "</resources>",
        r#"<object id="2" type="model"><components><component objectid="999"/></components></object></resources>"#,
    );

    let error = load_project(parts.bytes()).unwrap_err().to_string();
    assert!(
        error.contains("component references missing object"),
        "{error}"
    );
    assert!(error.contains("3D/leaf.model"), "{error}");
    assert!(error.contains("999"), "{error}");
}

#[test]
fn project_import_rejects_component_cycles() {
    let mut cycle = ProjectParts::valid();
    cycle.make_single_model(&ROOT_MODEL.replace(
        "<component p:path=\"/3D/leaf.model\" objectid=\"1\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"/>",
        "<component objectid=\"2\"/>",
    ));
    assert!(load_project(cycle.bytes()).is_err());
}
