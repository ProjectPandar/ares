use serde_json::{Map, Value, json};

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions, config_types::Nullable,
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
    // Literal duplicate keys now overwrite (upstream `set_deserialize_raw`,
    // Config.cpp:573-587): the later value wins in the parent and in every
    // owning child scope.
    let parsed: FilamentOptions =
        serde_json::from_str(
            r#"{"additional_cooling_fan_speed":["1"],"additional_cooling_fan_speed":["2"]}"#,
        )
        .unwrap();
    assert_eq!(parsed.print.additional_cooling_fan_speed.0[0].0, 2);
    let parsed: FilamentPrintSourceOptions =
        serde_json::from_str(
            r#"{"additional_cooling_fan_speed":["1"],"additional_cooling_fan_speed":["2"]}"#,
        )
        .unwrap();
    assert_eq!(parsed.additional_cooling_fan_speed.0[0].0, 2);
    let parsed: FilamentOptions =
        serde_json::from_str(
            r#"{"filament_ironing_flow":["nil"],"filament_ironing_flow":["37%"]}"#,
        )
        .unwrap();
    assert!(matches!(
        parsed.region.filament_ironing_flow.as_slice(),
        [Nullable::Value(percent)] if percent.0 == 37.0
    ));
    let parsed: FilamentRegionSourceOptions =
        serde_json::from_str(
            r#"{"filament_ironing_flow":["nil"],"filament_ironing_flow":["37%"]}"#,
        )
        .unwrap();
    assert!(matches!(
        parsed.filament_ironing_flow.as_slice(),
        [Nullable::Value(percent)] if percent.0 == 37.0
    ));
    let parsed: FilamentOptions =
        serde_json::from_str(
            r#"{"filament_retraction_length":["0.8"],"filament_retraction_length":["1"]}"#,
        )
        .unwrap();
    assert!(matches!(
        parsed.retract_overrides.filament_retraction_length.as_slice(),
        [Nullable::Value(value)] if value.0 == 1.0
    ));
    let parsed: FilamentRetractOverrideOptions =
        serde_json::from_str(
            r#"{"filament_retraction_length":["0.8"],"filament_retraction_length":["1"]}"#,
        )
        .unwrap();
    assert!(matches!(
        parsed.filament_retraction_length.as_slice(),
        [Nullable::Value(value)] if value.0 == 1.0
    ));

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
