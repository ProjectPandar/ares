use serde_json::{Map, Value, json};

use super::expected::{ExpectedField, REAL_FIELDS};
use super::{child_output, expected_default, parent_output};

#[test]
fn every_real_field_dispatches_a_valid_nondefault_through_child_and_flat_parent() {
    for field in &REAL_FIELDS {
        let alternate = alternate(field);
        assert_ne!(alternate, expected_default(field), "{}", field.key);
        let input = Value::Object(Map::from_iter([(field.key.to_owned(), alternate.clone())]));
        assert_eq!(child_output(field, input.clone())[field.key], alternate, "{}", field.key);
        assert_eq!(parent_output(input)[field.key], alternate, "{}", field.key);
    }
}

pub(super) fn alternate(field: &ExpectedField) -> Value {
    match field.kind {
        "coBool" => {
            if field.default_json == "\"1\"" {
                json!("0")
            } else {
                json!("1")
            }
        }
        "coBools" => json!(["1", "0", "1"]),
        "coEnum" if field.key == "curr_bed_type" => json!("Engineering Plate"),
        "coEnum" if field.key == "filament_map_mode" => json!("Manual"),
        "coEnums" => json!(["High Flow", "Standard", "High Flow"]),
        "coFloats" => json!(["7.125", "8.25", "9.5"]),
        "coInt" => json!("7"),
        "coInts" => json!(["7", "8", "9"]),
        "coPercents" => json!(["37%", "38%", "39%"]),
        "coPoints" => json!(["7x8", "-9.5x10", "0x0"]),
        "coString" => json!("raw task14 scalar\nvalue"),
        "coStrings" => json!(["raw task14 value", "second\nline", "third"]),
        kind => panic!("unexpected type {kind} for {}", field.key),
    }
}
