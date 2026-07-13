use serde_json::{Map, Value, json};

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions,
};
use super::{InventoryRow, inventory, remaining_rows};

#[test]
fn every_remaining_field_rejects_scalar_object_and_null_with_key() {
    let rows = inventory();
    for row in remaining_rows(&rows) {
        for invalid in [json!("1"), json!({}), Value::Null] {
            let input = Value::Object(Map::from_iter([(row.key.clone(), invalid)]));
            let parent = serde_json::from_value::<FilamentOptions>(input.clone())
                .unwrap_err()
                .to_string();
            assert!(parent.contains(&row.key), "{}: {parent}", row.key);
            if row.key != "pellet_flow_coefficient" {
                let child = child_error(row, input);
                assert!(child.contains(&row.key), "{}: {child}", row.key);
            }
        }
    }
}

#[test]
fn every_remaining_field_rejects_invalid_element_with_key() {
    let rows = inventory();
    for row in remaining_rows(&rows) {
        let invalid = if row.option_type == "coStrings" {
            json!([7])
        } else {
            json!(["not-a-value"])
        };
        let input = Value::Object(Map::from_iter([(row.key.clone(), invalid)]));
        let parent = serde_json::from_value::<FilamentOptions>(input.clone())
            .unwrap_err()
            .to_string();
        assert!(parent.contains(&row.key), "{}: {parent}", row.key);
        if row.key != "pellet_flow_coefficient" {
            let child = child_error(row, input);
            assert!(child.contains(&row.key), "{}: {child}", row.key);
        }
    }
}

#[test]
fn child_ownership_duplicates_unknown_and_nested_groups_are_rejected() {
    for (json, key) in [
        (
            r#"{"additional_cooling_fan_speed":["1"],"additional_cooling_fan_speed":["2"]}"#,
            "additional_cooling_fan_speed",
        ),
        (
            r#"{"filament_ironing_flow":["nil"],"filament_ironing_flow":["37%"]}"#,
            "filament_ironing_flow",
        ),
        (
            r#"{"filament_retraction_length":["0.8"],"filament_retraction_length":["1"]}"#,
            "filament_retraction_length",
        ),
    ] {
        let error = serde_json::from_str::<FilamentOptions>(json)
            .unwrap_err()
            .to_string();
        assert!(error.contains("duplicate Orca option"), "{error}");
        assert!(error.contains(key), "{error}");
        let child_error = match key {
            "additional_cooling_fan_speed" => {
                serde_json::from_str::<FilamentPrintSourceOptions>(json)
                    .unwrap_err()
                    .to_string()
            }
            "filament_ironing_flow" => {
                serde_json::from_str::<FilamentRegionSourceOptions>(json)
                    .unwrap_err()
                    .to_string()
            }
            "filament_retraction_length" => {
                serde_json::from_str::<FilamentRetractOverrideOptions>(json)
                    .unwrap_err()
                    .to_string()
            }
            _ => unreachable!(),
        };
        assert!(child_error.contains("duplicate Orca option"), "{child_error}");
        assert!(child_error.contains(key), "{child_error}");
    }

    for input in [
        r#"{"print":{"additional_cooling_fan_speed":["1"]}}"#,
        r#"{"region":{"filament_ironing_flow":["nil"]}}"#,
        r#"{"retract_overrides":{"filament_retraction_length":["0.8"]}}"#,
        r#"{"unknown_filament_option":["1"]}"#,
        r#"{"layer_height":"0.2"}"#,
    ] {
        assert!(serde_json::from_str::<FilamentOptions>(input).is_err(), "{input}");
    }
    assert!(
        serde_json::from_str::<FilamentPrintSourceOptions>(
            r#"{"filament_ironing_flow":["nil"]}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<FilamentRegionSourceOptions>(
            r#"{"filament_retraction_length":["0.8"]}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<FilamentRetractOverrideOptions>(
            r#"{"additional_cooling_fan_speed":["1"]}"#
        )
        .is_err()
    );
}

fn child_error(row: &InventoryRow, input: Value) -> String {
    match row.static_owner.as_str() {
        "print_config" => serde_json::from_value::<FilamentPrintSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        "print_region_config" => serde_json::from_value::<FilamentRegionSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        "unowned" => serde_json::from_value::<FilamentRetractOverrideOptions>(input)
            .unwrap_err()
            .to_string(),
        owner => panic!("unexpected owner {owner}"),
    }
}
