use serde_json::{Map, Value, json};

use super::super::super::{ProcessGCodeSourceOptions, ProcessOptions, ProcessPrintSourceOptions};
use super::{InventoryRow, expected_default, inventory, owner_keys, remaining_rows};

#[test]
fn every_child_field_dispatches_nondefault_through_standalone_and_flat_parent() {
    let rows = inventory();
    for owner in ["g_code_config", "print_config"] {
        for row in rows
            .iter()
            .filter(|row| row.raw_scope == "process" && row.static_owner == owner)
        {
            let alternate = alternate(row);
            assert_ne!(alternate, expected_default(row), "{}", row.key);
            let input = Value::Object(Map::from_iter([(row.key.clone(), alternate.clone())]));
            let child = if owner == "g_code_config" {
                serde_json::to_value(
                    serde_json::from_value::<ProcessGCodeSourceOptions>(input.clone()).unwrap(),
                )
                .unwrap()
            } else {
                serde_json::to_value(
                    serde_json::from_value::<ProcessPrintSourceOptions>(input.clone()).unwrap(),
                )
                .unwrap()
            };
            assert_eq!(child[&row.key], alternate, "{}", row.key);
            let parent: ProcessOptions = serde_json::from_value(input).unwrap();
            assert_eq!(serde_json::to_value(parent).unwrap()[&row.key], alternate, "{}", row.key);
        }
    }
}

#[test]
fn all_remaining_fields_reject_null_with_the_flat_key() {
    let rows = inventory();
    for row in remaining_rows(&rows) {
        let json = format!(r#"{{"{}":null}}"#, row.key);
        let parent_error = serde_json::from_str::<ProcessOptions>(&json).unwrap_err().to_string();
        assert!(parent_error.contains(&row.key), "{}: {parent_error}", row.key);
        let child_error = match row.static_owner.as_str() {
            "g_code_config" => serde_json::from_str::<ProcessGCodeSourceOptions>(&json)
                .unwrap_err()
                .to_string(),
            "print_config" => serde_json::from_str::<ProcessPrintSourceOptions>(&json)
                .unwrap_err()
                .to_string(),
            "unowned" => continue,
            owner => panic!("unexpected owner {owner}"),
        };
        assert!(child_error.contains(&row.key), "{}: {child_error}", row.key);
    }
}

#[test]
fn all_74_scalars_reject_array_and_object_shapes_with_the_flat_key() {
    let rows = inventory();
    for row in remaining_rows(&rows)
        .into_iter()
        .filter(|row| row.wire_shape == "scalar_string")
    {
        for invalid in [json!([]), json!({})] {
            let input = Value::Object(Map::from_iter([(row.key.clone(), invalid)]));
            let error = serde_json::from_value::<ProcessOptions>(input.clone())
                .unwrap_err()
                .to_string();
            assert!(error.contains(&row.key), "{}: {error}", row.key);
            let child_error = match row.static_owner.as_str() {
                "g_code_config" => serde_json::from_value::<ProcessGCodeSourceOptions>(input)
                    .unwrap_err()
                    .to_string(),
                "print_config" => serde_json::from_value::<ProcessPrintSourceOptions>(input)
                    .unwrap_err()
                    .to_string(),
                "unowned" => continue,
                owner => panic!("unexpected owner {owner}"),
            };
            assert!(child_error.contains(&row.key), "{}: {child_error}", row.key);
        }
    }
}

#[test]
fn child_ownership_is_strict_and_duplicates_remain_specific() {
    let rows = inventory();
    let gcode_key = owner_keys(&rows, "g_code_config")[0];
    let print_key = owner_keys(&rows, "print_config")[0];
    for json in [
        format!(r#"{{"{print_key}":"0"}}"#),
        r#"{"ironing_expansion":"0"}"#.to_owned(),
        r#"{"layer_height":"0.2"}"#.to_owned(),
    ] {
        assert!(serde_json::from_str::<ProcessGCodeSourceOptions>(&json).is_err(), "{json}");
    }
    for json in [
        format!(r#"{{"{gcode_key}":"0"}}"#),
        r#"{"ironing_expansion":"0"}"#.to_owned(),
        r#"{"wall_loops":"2"}"#.to_owned(),
    ] {
        assert!(serde_json::from_str::<ProcessPrintSourceOptions>(&json).is_err(), "{json}");
    }
    for (json, key) in [
        (r#"{"enable_arc_fitting":"0","enable_arc_fitting":"1"}"#, "enable_arc_fitting"),
        (r#"{"enable_prime_tower":"0","enable_prime_tower":"1"}"#, "enable_prime_tower"),
    ] {
        let error = serde_json::from_str::<ProcessOptions>(json).unwrap_err().to_string();
        assert!(error.contains("duplicate Orca option"), "{error}");
        assert!(error.contains(key), "{error}");
    }
}

#[test]
fn direct_ironing_expansion_accepts_value_and_rejects_duplicate_null_and_shapes() {
    let parsed: ProcessOptions = serde_json::from_str(r#"{"ironing_expansion":"7.125"}"#).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()["ironing_expansion"], "7.125");
    for invalid in [
        r#"{"ironing_expansion":"0","ironing_expansion":"1"}"#,
        r#"{"ironing_expansion":null}"#,
        r#"{"ironing_expansion":[]}"#,
        r#"{"ironing_expansion":{}}"#,
    ] {
        let error = serde_json::from_str::<ProcessOptions>(invalid).unwrap_err().to_string();
        assert!(error.contains("ironing_expansion"), "{error}");
    }
}

fn alternate(row: &InventoryRow) -> Value {
    match row.option_type.as_str() {
        "coBool" => Value::String(if row.default_serialized == "1" { "0" } else { "1" }.to_owned()),
        "coFloat" => Value::String("7.125".to_owned()),
        "coFloatOrPercent" if row.default_serialized.ends_with('%') => Value::String("7.125".to_owned()),
        "coFloatOrPercent" => Value::String("37%".to_owned()),
        "coFloats" => json!(["7.125", "8.25", "9.5"]),
        "coInt" => Value::String("7".to_owned()),
        "coPercent" => Value::String("37%".to_owned()),
        "coString" => Value::String("raw task11 value\nM117 Ares".to_owned()),
        "coStrings" => json!(["raw task11 value", "second", "third"]),
        "coEnum" => Value::String(match row.key.as_str() {
            "draft_shield" => "enabled",
            "print_order" => "as_obj_list",
            "print_sequence" => "by object",
            "skirt_type" => "perobject",
            "timelapse_type" => "1",
            "wipe_tower_wall_type" => "cone",
            key => panic!("unhandled enum {key}"),
        }.to_owned()),
        kind => panic!("unhandled option type {kind} for {}", row.key),
    }
}
