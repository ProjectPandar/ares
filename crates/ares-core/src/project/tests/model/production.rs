use crate::{
    GenerationMetadata, SliceError, load_project,
    project::{
        model_xml::ModelDocument,
        xml::{XmlRole, deserialize_xml},
    },
    slice_project,
};

use super::fixture::ProjectParts;

const CORE_NAMESPACE: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";
const PRODUCTION_NAMESPACE: &str =
    "http://schemas.microsoft.com/3dmanufacturing/production/2015/06";
const MATERIAL_NAMESPACE: &str = "http://schemas.microsoft.com/3dmanufacturing/material/2015/02";

fn model_xml(namespaces: &str, required_extensions: &str, resources: &str) -> String {
    format!(
        r#"<model xmlns="{CORE_NAMESPACE}" {namespaces} requiredextensions="{required_extensions}">
  <resources>{resources}</resources>
  <build/>
</model>"#
    )
}

fn parse(xml: &str) -> Result<ModelDocument, crate::SliceError> {
    deserialize_xml(xml.as_bytes(), XmlRole::Model)
}

fn assert_bounded_rejection(xml: &str) {
    let message = parse(xml).unwrap_err().to_string();
    assert!(!message.is_empty());
    assert!(message.len() <= 512, "unbounded XML error: {message}");
}

#[test]
fn object_name_pid_and_discarded_pindex_follow_orca_defaults() {
    let xml = model_xml(
        &format!(r#"xmlns:p="{PRODUCTION_NAMESPACE}""#),
        "p",
        r#"
    <object id="1"><components/></object>
    <object id="2" name="  Exact &amp; Name  " pid="not-a-number" pindex="19"><components/></object>
    <object id="3" name="third" pid="17"><components/></object>
    <object id="4" pid="+17suffix"><components/></object>
    <object id="5" pid="-4suffix"><components/></object>
  "#,
    );

    let document = parse(&xml).unwrap();
    let objects = &document.resources.objects;
    assert_eq!(objects[0].name, "");
    assert_eq!(objects[0].pid, 0);
    assert_eq!(objects[1].name, "  Exact & Name  ");
    assert_eq!(objects[1].pid, 0);
    assert_eq!(objects[2].name, "third");
    assert_eq!(objects[2].pid, 17);
    assert_eq!(objects[3].pid, 17);
    assert_eq!(objects[4].pid, -4);
}

#[test]
fn material_color_groups_preserve_source_order_and_last_color() {
    let xml = model_xml(
        &format!(r#"xmlns:p="{PRODUCTION_NAMESPACE}" xmlns:m="{MATERIAL_NAMESPACE}""#),
        "p m",
        r##"
    <m:colorgroup id="7">
      <m:color color="#111111"/>
      <m:color color="#222222"/>
    </m:colorgroup>
  "##,
    );

    let document = parse(&xml).unwrap();
    let groups = &document.resources.color_groups;
    assert_eq!(groups[0].id, 7);
    assert_eq!(groups[0].colors[0].color, "#111111");
    assert_eq!(groups[0].colors[1].color, "#222222");
    assert_eq!(groups[0].colors.last().unwrap().color, "#222222");
}

#[test]
fn required_extensions_resolve_distinct_exact_namespace_uris() {
    let xml = model_xml(
        &format!(r#"xmlns:prod="{PRODUCTION_NAMESPACE}" xmlns:mat="{MATERIAL_NAMESPACE}""#),
        "prod mat",
        r##"
    <object id="1" prod:UUID="object-uuid"><components/></object>
    <mat:colorgroup id="7"><mat:color color="#abcdef"/></mat:colorgroup>
  "##,
    );

    let document = parse(&xml).unwrap();
    assert_eq!(document.resources.objects[0].id, 1);
    assert_eq!(document.resources.color_groups[0].id, 7);
}

#[test]
fn required_extensions_reject_wrong_unbound_or_shared_namespace_uris() {
    let cases = [
        model_xml(r#"xmlns:m="urn:wrong-material""#, "m", ""),
        model_xml("", "m", ""),
        model_xml(
            &format!(r#"xmlns:p="{PRODUCTION_NAMESPACE}" xmlns:m="{PRODUCTION_NAMESPACE}""#),
            "p m",
            "",
        ),
    ];

    for xml in cases {
        assert_bounded_rejection(&xml);
    }
}

#[test]
fn required_extension_errors_bound_attacker_controlled_tokens() {
    for token in ["x".repeat(480), "x".repeat(4_096)] {
        assert_bounded_rejection(&model_xml("", &token, ""));
    }
}

#[test]
fn lowercase_production_uuid_requires_exact_namespace_uri() {
    let valid = model_xml(
        &format!(r#"xmlns:p="{PRODUCTION_NAMESPACE}""#),
        "p",
        r#"<object id="1" p:uuid="object-uuid"><components/></object>"#,
    );
    assert_eq!(parse(&valid).unwrap().resources.objects[0].id, 1);

    let wrong = model_xml(
        r#"xmlns:wrong="urn:wrong-production""#,
        "",
        r#"<object id="1" wrong:uuid="object-uuid"><components/></object>"#,
    );
    assert_bounded_rejection(&wrong);
}

#[test]
fn material_and_production_elements_require_their_exact_roles() {
    let namespaces = format!(
        r#"xmlns:p="{PRODUCTION_NAMESPACE}" xmlns:m="{MATERIAL_NAMESPACE}" xmlns:x="urn:extension""#
    );
    let cases = [
        model_xml(&namespaces, "p m", r#"<colorgroup id="7"/>"#),
        model_xml(&namespaces, "p m", r#"<p:colorgroup id="7"/>"#),
        model_xml(
            &namespaces,
            "p m",
            r#"<m:object id="1"><m:components/></m:object>"#,
        ),
        model_xml(&namespaces, "p m", r#"<p:path/>"#),
        model_xml(&namespaces, "p m", r#"<x:extension/>"#),
        model_xml(&namespaces, "p m", r#"<texture2d id="1"/>"#),
        model_xml("", "", r#"<m:colorgroup id="7"/>"#),
    ];

    for xml in cases {
        assert_bounded_rejection(&xml);
    }
}

#[test]
fn fixed_attribute_vocabulary_and_namespace_roles_are_enforced() {
    let namespaces = format!(
        r#"xmlns:p="{PRODUCTION_NAMESPACE}" xmlns:m="{MATERIAL_NAMESPACE}" xmlns:x="urn:extension""#
    );
    let cases = [
        model_xml(
            &namespaces,
            "p m",
            r##"<m:colorgroup id="7"><m:color m:color="#111111"/></m:colorgroup>"##,
        ),
        model_xml(
            &namespaces,
            "p m",
            r##"<m:colorgroup id="7"><m:color p:color="#111111"/></m:colorgroup>"##,
        ),
        model_xml(
            &namespaces,
            "p m",
            r#"<object id="1" x:flag="true"><components/></object>"#,
        ),
        model_xml(
            &namespaces,
            "p m",
            r#"<object id="1" bogus="true"><components/></object>"#,
        ),
        model_xml(
            &namespaces,
            "p m",
            r#"<object id="1" p:pid="3"><components/></object>"#,
        ),
    ];

    for xml in cases {
        assert_bounded_rejection(&xml);
    }
}

#[test]
fn material_group_and_color_require_typed_attributes() {
    let namespaces = format!(r#"xmlns:m="{MATERIAL_NAMESPACE}""#);
    let cases = [
        model_xml(
            &namespaces,
            "m",
            r##"<m:colorgroup><m:color color="#111111"/></m:colorgroup>"##,
        ),
        model_xml(&namespaces, "m", r#"<m:colorgroup id="not-a-number"/>"#),
        model_xml(
            &namespaces,
            "m",
            r#"<m:colorgroup id="7"><m:color/></m:colorgroup>"#,
        ),
    ];

    for xml in cases {
        assert_bounded_rejection(&xml);
    }
}

#[tokio::test]
async fn bbs_painted_triangle_attributes_remain_fail_closed_through_load_and_slice() {
    const MODEL_PATH: &str = "3D/Objects/ksr_fdmtest_v4.drc_2.model";
    const FIRST_TRIANGLE: &str = r#"<triangle v1="2" v2="0" v3="1"/>"#;
    const ERROR: &str =
        "invalid project model XML: attribute namespace does not match its 3MF meaning";

    for attribute in [r#"paint_color="1""#, r#"paint_fuzzy_skin="1""#] {
        let mut parts = ProjectParts::fixture();
        parts.replace(
            MODEL_PATH,
            FIRST_TRIANGLE,
            &format!(r#"<triangle v1="2" v2="0" v3="1" {attribute}/>"#),
        );
        let bytes = parts.bytes();

        assert_eq!(
            load_project(&bytes).unwrap_err(),
            SliceError::InvalidInput(ERROR.to_owned())
        );
        assert_eq!(
            slice_project(
                &bytes,
                GenerationMetadata::deterministic(2026, 7, 19, 1, 2, 3),
            )
            .await
            .unwrap_err(),
            SliceError::InvalidInput(ERROR.to_owned())
        );
    }
}
