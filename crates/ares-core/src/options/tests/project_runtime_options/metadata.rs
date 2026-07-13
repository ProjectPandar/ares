use serde_json::{Value, json};

use super::expected::METADATA_KEYS;
use super::super::super::PresetMetadata;
use super::{assert_keyed_bounded_error, fixture_fields, metadata_output, serialized_key_order};

#[test]
fn metadata_defaults_nondefaults_fixture_and_lexical_bytes_are_exact() {
    let default = PresetMetadata::default();
    assert_eq!(default.from, "");
    assert_eq!(default.name, "");
    assert_eq!(default.version, "");
    let serialized = serde_json::to_string(&default).unwrap();
    assert_eq!(serialized_key_order(&serialized), METADATA_KEYS);
    assert_eq!(serialized, r#"{"from":"","name":"","version":""}"#);

    let alternate = json!({"from":"project-copy","name":"settings-copy","version":"99.88.77"});
    assert_eq!(metadata_output(alternate.clone()), alternate);

    let fixture = Value::Object(fixture_fields(METADATA_KEYS));
    assert_eq!(
        fixture,
        json!({"from":"project","name":"project_settings","version":"02.06.00.51"})
    );
    let parsed: PresetMetadata = serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), fixture);
}

#[test]
fn metadata_rejects_unknown_duplicate_null_array_object_and_nested_forms() {
    for key in METADATA_KEYS {
        for invalid in [Value::Null, json!([]), json!({}), json!(7)] {
            let error = serde_json::from_value::<PresetMetadata>(json!({key: invalid}))
                .unwrap_err()
                .to_string();
            assert_keyed_bounded_error(&error, key);
        }
        let input = format!("{{\"{key}\":\"a\",\"{key}\":\"b\"}}");
        let error = serde_json::from_str::<PresetMetadata>(&input)
            .unwrap_err()
            .to_string();
        assert_keyed_bounded_error(&error, key);
    }
    for (input, key) in [
        (json!({"unknown_metadata": "x"}), "unknown_metadata"),
        (json!({"metadata": {"from": "project"}}), "metadata"),
    ] {
        let error = serde_json::from_value::<PresetMetadata>(input)
            .unwrap_err()
            .to_string();
        assert_keyed_bounded_error(&error, key);
    }
    for input in ["null", "[]", "\"not-a-map\""] {
        assert!(serde_json::from_str::<PresetMetadata>(input).is_err());
    }
}
