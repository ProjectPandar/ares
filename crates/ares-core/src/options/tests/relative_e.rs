use super::super::*;
use serde_json::json;

#[test]
fn relative_e_distances_defaults_to_orca_enabled() {
    assert!(SliceOptions::default().use_relative_e_distances().unwrap());
}

#[test]
fn relative_e_distances_parses_explicit_booleans() {
    let enabled: SliceOptions =
        serde_json::from_value(json!({ "use_relative_e_distances": true })).unwrap();
    let disabled: SliceOptions =
        serde_json::from_value(json!({ "use_relative_e_distances": false })).unwrap();

    assert!(enabled.use_relative_e_distances().unwrap());
    assert!(!disabled.use_relative_e_distances().unwrap());
}

#[test]
fn relative_e_distances_rejects_non_boolean_values() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "use_relative_e_distances": "true" })).unwrap();
    let err = options.use_relative_e_distances().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(
        err.to_string()
            .contains("use_relative_e_distances must be a boolean")
    );
}
