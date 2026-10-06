use crate::ProjectSettings;

#[test]
fn unknown_project_options_are_ignored_like_orca_config_loading() {
    let settings: ProjectSettings =
        serde_json::from_str(r#"{"future_option":"1","layer_height":"0.2"}"#).unwrap();
    // unknown keys are skipped; known keys still parse
    assert!(!format!("{settings:?}").is_empty());
}

#[test]
fn duplicate_diagnostics_are_compact_and_exact_keyed() {
    // Option-group duplicates now overwrite (upstream `set_deserialize_raw`,
    // Config.cpp:573-587); the hand-written metadata guard still rejects
    // duplicates of the metadata keys.
    assert!(keyed_error(
        r#"{"from":"project","from":"copy"}"#,
        "from",
    )
    .starts_with("duplicate Orca option from"));
}

#[test]
fn malformed_canonical_values_have_compact_keyed_diagnostics() {
    for (input, key, reason) in [
        (
            r#"{"layer_height":"not-a-float"}"#,
            "layer_height",
            "invalid float literal",
        ),
        (r#"{"layer_height":null}"#, "layer_height", "invalid type"),
        (
            r#"{"gcode_flavor":"not-a-flavor"}"#,
            "gcode_flavor",
            "unknown variant",
        ),
        (
            r#"{"nozzle_diameter":"0.4"}"#,
            "nozzle_diameter",
            "invalid type",
        ),
        (
            r#"{"nozzle_diameter":["0.4","not-a-float"]}"#,
            "nozzle_diameter",
            "invalid float literal",
        ),
    ] {
        let error = keyed_error(input, key);
        assert!(error.contains(reason), "diagnostic omitted {reason}: {error}");
    }
}

#[test]
fn null_and_wrong_top_level_containers_are_rejected_compactly() {
    for input in ["null", "[]", r#""not-a-map""#] {
        let error = serde_json::from_str::<ProjectSettings>(input)
            .unwrap_err()
            .to_string();
        assert!(!error.is_empty());
        assert!(error.contains("Orca project settings"), "{error}");
        assert!(error.len() < 1_024, "oversized diagnostic: {}", error.len());
    }
}

fn keyed_error(input: &str, key: &str) -> String {
    let error = serde_json::from_str::<ProjectSettings>(input)
        .unwrap_err()
        .to_string();
    assert!(error.contains(key), "diagnostic omitted {key}: {error}");
    assert!(error.len() < 1_024, "oversized diagnostic: {}", error.len());
    error
}
