use super::{LayerProject, assert_bounded, error_message};

fn ranges(body: &str) -> String {
    format!(r#"<objects>{body}</objects>"#)
}

fn invalid_document(xml: &str) -> String {
    let mut project = LayerProject::one_object();
    project.insert_ranges("Metadata/layer_config_ranges.xml", xml);
    let message = error_message(project);
    assert_bounded(&message);
    message
}

#[test]
fn zero_out_of_range_malformed_and_duplicate_ordinals_are_bounded_errors() {
    for (xml, expected) in [
        (ranges(r#"<object id="0"/>"#), "0"),
        (ranges(r#"<object id="2"/>"#), "2"),
        (ranges(r#"<object id="wat"/>"#), "wat"),
        (ranges(r#"<object id="1"/><object id="1"/>"#), "duplicate"),
    ] {
        let message = invalid_document(&xml);
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn missing_attributes_and_nonfinite_bounds_are_bounded_keyed_errors() {
    for (xml, expected) in [
        (ranges("<object/>"), "id"),
        (
            ranges(r#"<object id="1"><range max_z="1"/></object>"#),
            "min_z",
        ),
        (
            ranges(r#"<object id="1"><range min_z="0"/></object>"#),
            "max_z",
        ),
        (
            ranges(
                r#"<object id="1"><range min_z="0" max_z="1"><option>2</option></range></object>"#,
            ),
            "opt_key",
        ),
        (
            ranges(r#"<object id="1"><range min_z="NaN" max_z="1"/></object>"#),
            "min_z",
        ),
        (
            ranges(r#"<object id="1"><range min_z="0" max_z="inf"/></object>"#),
            "max_z",
        ),
    ] {
        let message = invalid_document(&xml);
        assert!(message.contains(expected), "{message}");
    }
}

#[test]
fn malformed_xml_and_unknown_fixed_vocabulary_are_rejected() {
    for xml in [
        "<objects><object id=\"1\"></objects>",
        "<objects><bogus/></objects>",
        "<objects><object id=\"1\" bogus=\"x\"/></objects>",
        r#"<objects xmlns:x="urn:not-layer-config"><x:object id="1"/></objects>"#,
    ] {
        let message = invalid_document(xml);
        assert!(
            message.contains("layer configuration ranges XML"),
            "{message}"
        );
    }
}

#[test]
fn unknown_keys_and_invalid_values_are_bounded_without_echoing_attacker_input() {
    let huge_key = "unknown_".to_owned() + &"k".repeat(4_096);
    let unknown_xml = ranges(&format!(
        r#"<object id="1"><range min_z="0" max_z="1"><option opt_key="{huge_key}">1</option></range></object>"#
    ));
    let unknown = invalid_document(&unknown_xml);
    assert!(unknown.contains("unknown_"), "{unknown}");
    assert!(!unknown.contains(&huge_key), "{unknown}");

    let huge_value = "9".repeat(4_096);
    let value_xml = ranges(&format!(
        r#"<object id="1"><range min_z="0" max_z="1"><option opt_key="wall_loops">{huge_value}</option></range></object>"#
    ));
    let invalid = invalid_document(&value_xml);
    assert!(invalid.contains("wall_loops"), "{invalid}");
    assert!(!invalid.contains(&huge_value), "{invalid}");
}

#[test]
fn unknown_xml_names_are_bounded_without_echoing_attacker_input() {
    for huge_name in [
        "element_".to_owned() + &"x".repeat(4_096),
        "element_".to_owned() + &"名".repeat(1_024),
    ] {
        let xml = format!(r#"<objects><{huge_name}/></objects>"#);

        let message = invalid_document(&xml);

        assert!(
            message.contains("layer configuration ranges XML"),
            "{message}"
        );
        assert!(!message.contains(&huge_name), "{message}");
    }
}
