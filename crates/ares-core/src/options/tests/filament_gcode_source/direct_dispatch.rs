use serde_json::{Map, Value, json};

use super::super::super::{FilamentGCodeSourceOptions, FilamentOptions};
use super::{InventoryRow, expected_default, filament_rows, inventory};

#[test]
fn every_field_dispatches_nondefault_through_child_and_flat_parent() {
    let inventory = inventory();
    for row in filament_rows(&inventory) {
        let alternate = alternate(row);
        assert_ne!(alternate, expected_default(row), "{}", row.key);
        let input = Value::Object(Map::from_iter([(row.key.clone(), alternate.clone())]));
        let child: FilamentGCodeSourceOptions = serde_json::from_value(input.clone()).unwrap();
        let parent: FilamentOptions = serde_json::from_value(input).unwrap();
        assert_eq!(serde_json::to_value(child).unwrap()[&row.key], alternate, "{}", row.key);
        assert_eq!(serde_json::to_value(parent).unwrap()[&row.key], alternate, "{}", row.key);
    }
}

#[test]
fn every_field_rejects_top_level_scalar_object_and_null_with_key() {
    let inventory = inventory();
    for row in filament_rows(&inventory) {
        for invalid in [json!("1"), json!({}), Value::Null] {
            let input = Value::Object(Map::from_iter([(row.key.clone(), invalid)]));
            let child = serde_json::from_value::<FilamentGCodeSourceOptions>(input.clone())
                .unwrap_err()
                .to_string();
            let parent = serde_json::from_value::<FilamentOptions>(input)
                .unwrap_err()
                .to_string();
            assert!(child.contains(&row.key), "{}: {child}", row.key);
            assert!(parent.contains(&row.key), "{}: {parent}", row.key);
        }
    }
}

#[test]
fn every_field_rejects_invalid_element_shape_with_key() {
    let inventory = inventory();
    for row in filament_rows(&inventory) {
        let invalid = match row.option_type.as_str() {
            "coBools" | "coFloats" | "coInts" => json!(["not-a-value"]),
            "coStrings" => json!([7]),
            kind => panic!("unexpected type {kind}"),
        };
        let input = Value::Object(Map::from_iter([(row.key.clone(), invalid)]));
        let child = serde_json::from_value::<FilamentGCodeSourceOptions>(input.clone())
            .unwrap_err()
            .to_string();
        let parent = serde_json::from_value::<FilamentOptions>(input)
            .unwrap_err()
            .to_string();
        assert!(child.contains(&row.key), "{}: {child}", row.key);
        assert!(parent.contains(&row.key), "{}: {parent}", row.key);
    }
}

#[test]
fn duplicate_unknown_and_cross_scope_fields_are_rejected() {
    for input in [
        r#"{"pressure_advance":["0.1"],"pressure_advance":["0.2"]}"#,
        r#"{"unknown_filament_option":["1"]}"#,
        r#"{"gcode":{"pressure_advance":["0.1"]}}"#,
        r#"{"layer_height":"0.2"}"#,
    ] {
        let child = serde_json::from_str::<FilamentGCodeSourceOptions>(input)
            .unwrap_err()
            .to_string();
        let parent = serde_json::from_str::<FilamentOptions>(input)
            .unwrap_err()
            .to_string();
        assert!(!child.is_empty(), "{input}");
        assert!(!parent.is_empty(), "{input}");
    }
    let error = serde_json::from_str::<FilamentOptions>(
        r#"{"pressure_advance":["0.1"],"pressure_advance":["0.2"]}"#,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("duplicate Orca option pressure_advance"), "{error}");
}

fn alternate(row: &InventoryRow) -> Value {
    match row.option_type.as_str() {
        "coBools" => json!(["1", "0", "1"]),
        "coFloats" => json!(["7.125", "8.25", "9.5"]),
        "coInts" => json!(["7", "8", "9"]),
        "coStrings" => json!(["raw task12 value", "second\nline", "third"]),
        kind => panic!("unexpected option type {kind}"),
    }
}
