use serde_json::Value;

use crate::ProjectSettings;

use super::super::project_fixture::project_settings_bytes;
use super::support::serialized_project_values;

#[test]
fn five_standalone_groups_are_an_exact_flat_semantic_oracle_for_the_real_fixture() {
    let raw = project_settings_bytes();
    let fixture: Value = serde_json::from_slice(&raw).unwrap();
    let fixture = fixture.as_object().unwrap();
    let settings: ProjectSettings = serde_json::from_slice(&raw).unwrap();
    let serialized = serialized_project_values(&settings);
    let mut expected = fixture.clone();
    assert_eq!(
        expected.insert(
            "thumbnails".to_owned(),
            Value::String("48x48/PNG, 300x300/PNG".to_owned())
        ),
        Some(Value::String("48x48/PNG,300x300/PNG".to_owned()))
    );

    assert_eq!(serialized.len(), 653);
    assert_eq!(serialized, expected);
    assert!(fixture.values().all(|value| match value {
        Value::String(_) => true,
        Value::Array(values) => values.iter().all(Value::is_string),
        _ => false,
    }));
}

#[test]
fn existing_native_scalar_inputs_canonicalize_through_their_group_serializers() {
    let settings: ProjectSettings = serde_json::from_str(
        r#"{"alternate_extra_wall":true,"layer_height":0.375}"#,
    )
    .unwrap();
    let process = serde_json::to_value(&settings.process).unwrap();

    assert_eq!(process["alternate_extra_wall"], "1");
    assert_eq!(process["layer_height"], "0.375");
}
