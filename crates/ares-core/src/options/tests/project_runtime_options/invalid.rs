use serde_json::{Map, Value, json};

use super::direct_dispatch::alternate;
use super::expected::{Child, REAL_FIELDS};
use super::super::super::{
    ProjectGCodeSourceOptions, ProjectPrintSourceOptions, ProjectPresetSourceOptions,
    ProjectRuntimeOptions,
};
use super::{assert_keyed_bounded_error, child_error, parent_error};

#[test]
fn every_real_field_rejects_null_wrong_shape_and_invalid_elements_with_key() {
    for field in &REAL_FIELDS {
        let wrong_shape = if field.is_array { json!("1") } else { json!(["1"]) };
        for invalid in [Value::Null, wrong_shape, json!({})] {
            let input = Value::Object(Map::from_iter([(field.key.to_owned(), invalid)]));
            assert_keyed_bounded_error(&child_error(field, input.clone()), field.key);
            assert_keyed_bounded_error(&parent_error(input), field.key);
        }
        let invalid_element = invalid_lexical_value(field.kind);
        let input = Value::Object(Map::from_iter([(field.key.to_owned(), invalid_element)]));
        assert_keyed_bounded_error(&child_error(field, input.clone()), field.key);
        assert_keyed_bounded_error(&parent_error(input), field.key);
    }
}

#[test]
fn every_non_string_field_rejects_nil_as_a_nonnullable_lexical_value() {
    for field in REAL_FIELDS.iter().filter(|field| !matches!(field.kind, "coString" | "coStrings"))
    {
        let invalid = if field.is_array { json!(["nil"]) } else { json!("nil") };
        let input = Value::Object(Map::from_iter([(field.key.to_owned(), invalid)]));
        assert_keyed_bounded_error(&child_error(field, input.clone()), field.key);
        assert_keyed_bounded_error(&parent_error(input), field.key);
    }
}

#[test]
fn child_parent_boundaries_reject_non_map_top_level_shapes() {
    for input in ["null", "[]", "\"not-a-map\""] {
        assert!(serde_json::from_str::<ProjectGCodeSourceOptions>(input).is_err());
        assert!(serde_json::from_str::<ProjectPrintSourceOptions>(input).is_err());
        assert!(serde_json::from_str::<ProjectPresetSourceOptions>(input).is_err());
        assert!(serde_json::from_str::<ProjectRuntimeOptions>(input).is_err());
    }
}

#[test]
fn every_real_field_rejects_duplicates_in_child_and_flat_parent() {
    for field in &REAL_FIELDS {
        let value = serde_json::to_string(&alternate(field)).unwrap();
        let input = format!(
            "{{\"{}\":{},\"{}\":{}}}",
            field.key, value, field.key, value
        );
        let parent_error = serde_json::from_str::<ProjectRuntimeOptions>(&input)
            .unwrap_err()
            .to_string();
        assert_keyed_bounded_error(&parent_error, field.key);
        let child_error = child_raw_error(field.child, &input);
        assert_keyed_bounded_error(&child_error, field.key);
    }
}

#[test]
fn children_reject_every_cross_child_key_and_all_boundaries_reject_nested_or_unknown_keys() {
    for field in &REAL_FIELDS {
        let input = Value::Object(Map::from_iter([(
            field.key.to_owned(),
            alternate(field),
        )]));
        for target in [Child::GCode, Child::Print, Child::Preset] {
            if target != field.child {
                let error = child_value_error(target, input.clone());
                assert_keyed_bounded_error(&error, field.key);
            }
        }
    }

    for (input, key) in [
        (json!({"unknown_project_option": ["1"]}), "unknown_project_option"),
        (json!({"gcode": {"z_hop": ["0.4"]}}), "gcode"),
        (json!({"print": {"wipe": ["1"]}}), "print"),
        (json!({"preset": {"print_settings_id": "x"}}), "preset"),
    ] {
        assert_keyed_bounded_error(&parent_error(input), key);
    }
    for (child, input, key) in [
        (Child::GCode, json!({"gcode": {}}), "gcode"),
        (Child::Print, json!({"print": {}}), "print"),
        (Child::Preset, json!({"preset": {}}), "preset"),
        (Child::GCode, json!({"unknown_gcode_option": ["1"]}), "unknown_gcode_option"),
        (Child::Print, json!({"unknown_print_option": ["1"]}), "unknown_print_option"),
        (Child::Preset, json!({"unknown_preset_option": ["1"]}), "unknown_preset_option"),
    ] {
        assert_keyed_bounded_error(&child_value_error(child, input), key);
    }
}

fn child_raw_error(child: Child, input: &str) -> String {
    match child {
        Child::GCode => serde_json::from_str::<ProjectGCodeSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        Child::Print => serde_json::from_str::<ProjectPrintSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        Child::Preset => serde_json::from_str::<ProjectPresetSourceOptions>(input)
            .unwrap_err()
            .to_string(),
    }
}

fn child_value_error(child: Child, input: Value) -> String {
    match child {
        Child::GCode => serde_json::from_value::<ProjectGCodeSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        Child::Print => serde_json::from_value::<ProjectPrintSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        Child::Preset => serde_json::from_value::<ProjectPresetSourceOptions>(input)
            .unwrap_err()
            .to_string(),
    }
}

fn invalid_lexical_value(kind: &str) -> Value {
    match kind {
        "coBool" => json!("2"),
        "coBools" => json!(["2"]),
        "coEnum" => json!("not-a-token"),
        "coEnums" => json!(["not-a-token"]),
        "coFloats" => json!(["NaN"]),
        "coInt" => json!("1.5"),
        "coInts" => json!(["1.5"]),
        "coPercents" => json!(["not-a-percent"]),
        "coPoints" => json!(["1,2"]),
        "coString" => json!(7),
        "coStrings" => json!([7]),
        kind => panic!("unexpected type {kind}"),
    }
}
