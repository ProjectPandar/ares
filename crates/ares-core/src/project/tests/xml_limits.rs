use std::fmt::Write as _;

use crate::project::{
    content_types::ContentTypes,
    filament_sequence::FilamentSequences,
    model_settings::ModelSettings,
    plate::PlateJson,
    relationships::Relationships,
    xml::{JsonRole, XmlRole, deserialize_json, deserialize_xml, validate_xml_for_test},
};

const TEXT_LIMIT: usize = 64 * 1024 * 1024;

#[test]
fn project_xml_limits_reject_wrong_or_rebound_namespaces() {
    assert!(
        deserialize_xml::<ContentTypes>(br#"<Types xmlns="urn:wrong"/>"#, XmlRole::ContentTypes)
            .is_err()
    );
    assert!(
        deserialize_xml::<Relationships>(
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship xmlns="urn:wrong" Target="a" Id="r" Type="t"/></Relationships>"#,
            XmlRole::Relationships,
        )
        .is_err()
    );
    assert!(
        deserialize_xml::<ModelSettings>(
            br#"<config xmlns="urn:unexpected"/>"#,
            XmlRole::ModelSettings
        )
        .is_err()
    );
}

#[test]
fn project_xml_limits_reject_depth_257() {
    let mut xml = "<config>".to_owned();
    xml.push_str(&"<n>".repeat(256));
    xml.push_str(&"</n>".repeat(256));
    xml.push_str("</config>");

    assert!(deserialize_xml::<ModelSettings>(xml.as_bytes(), XmlRole::ModelSettings).is_err());
}

#[test]
fn project_xml_limits_accept_1024_attributes_and_reject_1025() {
    let exact = config_with_attributes(1_024);
    let over = config_with_attributes(1_025);

    deserialize_xml::<ModelSettings>(exact.as_bytes(), XmlRole::ModelSettings).unwrap();
    assert!(deserialize_xml::<ModelSettings>(over.as_bytes(), XmlRole::ModelSettings).is_err());
}

#[test]
fn project_xml_limits_reject_document_over_64_mib() {
    let xml = vec![b' '; TEXT_LIMIT + 1];

    assert!(deserialize_xml::<ModelSettings>(&xml, XmlRole::ModelSettings).is_err());
}

#[test]
fn project_xml_limits_reject_decoded_text_over_64_mib() {
    let mut xml = Vec::with_capacity(TEXT_LIMIT + 32);
    xml.extend_from_slice(b"<config>");
    xml.resize(xml.len() + TEXT_LIMIT + 1, b'x');
    xml.extend_from_slice(b"</config>");
    let document_limit = xml.len();

    assert!(
        validate_xml_for_test(&xml, XmlRole::ModelSettings, document_limit, TEXT_LIMIT,).is_err()
    );
}

#[test]
fn project_xml_limits_reject_dtd_and_entity_expansion_constructs() {
    for xml in [
        br#"<!DOCTYPE config><config/>"#.as_slice(),
        br#"<!DOCTYPE config [<!ENTITY external SYSTEM "file:///etc/passwd">]><config>&external;</config>"#,
        br#"<!DOCTYPE config [<!ENTITY payload "expanded">]><config>&payload;</config>"#,
        br#"<!DOCTYPE config [<!ENTITY a "x"><!ENTITY b "&a;&a;">]><config>&b;</config>"#,
    ] {
        assert!(deserialize_xml::<ModelSettings>(xml, XmlRole::ModelSettings).is_err());
    }
}

#[test]
fn project_xml_limits_reject_general_entity_references_without_dtd() {
    for xml in [
        br#"<config>&payload;</config>"#.as_slice(),
        br#"<config unknown="&payload;"/>"#,
    ] {
        assert!(deserialize_xml::<ModelSettings>(xml, XmlRole::ModelSettings).is_err());
    }
}

#[test]
fn object_settings_metadata_project_xml_limits_allow_predefined_and_numeric_character_references() {
    let settings: ModelSettings = deserialize_xml(
        br#"<config><object id="2"><metadata key="name" value="a&amp;b&#x21;"/></object></config>"#,
        XmlRole::ModelSettings,
    )
    .unwrap();

    assert_eq!(settings.objects[0].name, "a&b!");
}

#[test]
fn project_xml_limits_reject_xml10_illegal_characters_in_text_and_attributes() {
    for reference in ["&#x1;", "&#xFFFE;", "&#xFFFF;"] {
        let text = format!("<config>{reference}</config>");
        let attribute = format!("<config value=\"{reference}\"/>");
        assert!(
            validate_xml_for_test(
                text.as_bytes(),
                XmlRole::ModelSettings,
                text.len(),
                TEXT_LIMIT,
            )
            .is_err(),
            "accepted text reference {reference}"
        );
        assert!(
            validate_xml_for_test(
                attribute.as_bytes(),
                XmlRole::ModelSettings,
                attribute.len(),
                TEXT_LIMIT,
            )
            .is_err(),
            "accepted attribute reference {reference}"
        );
    }

    for character in ['\u{1}', '\u{fffe}', '\u{ffff}'] {
        let text = format!("<config>{character}</config>");
        let attribute = format!("<config value=\"{character}\"/>");
        assert!(
            validate_xml_for_test(
                text.as_bytes(),
                XmlRole::ModelSettings,
                text.len(),
                TEXT_LIMIT,
            )
            .is_err(),
            "accepted literal text U+{:04X}",
            character as u32
        );
        assert!(
            validate_xml_for_test(
                attribute.as_bytes(),
                XmlRole::ModelSettings,
                attribute.len(),
                TEXT_LIMIT,
            )
            .is_err(),
            "accepted literal attribute U+{:04X}",
            character as u32
        );
    }
}

#[test]
fn project_xml_limits_preserve_xml10_and_xml11_character_semantics() {
    let xml10 = "<config value=\"\t\n\r&#x9;&#xA;&#xD;&#x20;&#xE000;&#x10000;\">\t\n\r</config>";
    validate_xml_for_test(
        xml10.as_bytes(),
        XmlRole::ModelSettings,
        xml10.len(),
        TEXT_LIMIT,
    )
    .unwrap();

    let xml11_reference = "<?xml version=\"1.1\"?><config value=\"&#x1;\">&#x1;</config>";
    validate_xml_for_test(
        xml11_reference.as_bytes(),
        XmlRole::ModelSettings,
        xml11_reference.len(),
        TEXT_LIMIT,
    )
    .unwrap();

    for reference in ["&#xFFFE;", "&#xFFFF;"] {
        let xml11 = format!("<?xml version=\"1.1\"?><config>{reference}</config>");
        assert!(
            validate_xml_for_test(
                xml11.as_bytes(),
                XmlRole::ModelSettings,
                xml11.len(),
                TEXT_LIMIT,
            )
            .is_err()
        );
    }

    let xml11_literal = "<?xml version=\"1.1\"?><config value=\"\u{1}\">\u{1}</config>";
    assert!(
        validate_xml_for_test(
            xml11_literal.as_bytes(),
            XmlRole::ModelSettings,
            xml11_literal.len(),
            TEXT_LIMIT,
        )
        .is_err()
    );
}

#[test]
fn project_xml_limits_reject_malformed_concrete_xml_types() {
    assert!(
        deserialize_xml::<ModelSettings>(
            br#"<config><object id="not-an-integer"/></config>"#,
            XmlRole::ModelSettings,
        )
        .is_err()
    );
}

#[test]
fn project_xml_limits_reject_json_over_64_mib() {
    let json = vec![b' '; TEXT_LIMIT + 1];

    assert!(deserialize_json::<PlateJson>(&json, JsonRole::Plate).is_err());
}

#[test]
fn project_xml_limits_reject_malformed_concrete_json_types() {
    let plate = br#"{"bbox_all":[0,0,1,1],"bbox_objects":[],"bed_type":"hot_plate","filament_colors":[],"filament_ids":[],"first_extruder":0,"first_layer_time":"not-a-number","is_seq_print":false,"nozzle_diameter":0.4,"version":2}"#;
    assert!(deserialize_json::<PlateJson>(plate, JsonRole::Plate).is_err());

    let sequences =
        br#"{"plate_1":{"nozzle_sequence":[],"optimal_assignment":[],"sequence":"not-an-array"}}"#;
    assert!(deserialize_json::<FilamentSequences>(sequences, JsonRole::FilamentSequences).is_err());
}

#[test]
fn project_xml_limits_plate_id_accepts_only_canonical_positive_keys() {
    for key in [
        "plate_0", "plate_01", "plate_-1", "plate_+1", "plate_x", "1",
    ] {
        let json = format!(
            r#"{{"{key}":{{"nozzle_sequence":[],"optimal_assignment":[],"sequence":[]}}}}"#
        );
        assert!(
            deserialize_json::<FilamentSequences>(json.as_bytes(), JsonRole::FilamentSequences,)
                .is_err(),
            "accepted {key:?}"
        );
    }

    let canonical: FilamentSequences = deserialize_json(
        br#"{"plate_42":{"nozzle_sequence":[],"optimal_assignment":[],"sequence":[]}}"#,
        JsonRole::FilamentSequences,
    )
    .unwrap();
    assert_eq!(canonical.0.first_key_value().unwrap().0.get(), 42);
}

fn config_with_attributes(count: usize) -> String {
    let mut xml = "<config".to_owned();
    for index in 0..count {
        write!(xml, " a{index}=\"x\"").unwrap();
    }
    xml.push_str("/>");
    xml
}
