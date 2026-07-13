use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions,
};
use super::expected::PRODUCTION_LITERAL_COMPLEMENT;
use super::{fixture_fields, inventory, remaining_rows, serialized_key_order};

#[test]
fn standalone_children_emit_exact_lexical_keys_and_flat_parent_emits_122() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    for (owner, serialized) in [
        (
            "print_config",
            serde_json::to_string(&FilamentPrintSourceOptions::default()).unwrap(),
        ),
        (
            "print_region_config",
            serde_json::to_string(&FilamentRegionSourceOptions::default()).unwrap(),
        ),
    ] {
        let keys = remaining
            .iter()
            .filter(|row| row.static_owner == owner)
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(serialized_key_order(&serialized), keys, "{owner}");
    }
    let retract_keys = remaining
        .iter()
        .filter(|row| row.static_owner == "unowned" && row.key != "pellet_flow_coefficient")
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        serialized_key_order(
            &serde_json::to_string(&FilamentRetractOverrideOptions::default()).unwrap()
        ),
        retract_keys
    );
    let parent_keys = rows
        .iter()
        .filter(|row| row.raw_scope == "filament")
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    let parent = serde_json::to_string(&FilamentOptions::default()).unwrap();
    assert_eq!(serialized_key_order(&parent), parent_keys);
    assert!(!parent.contains(r#""gcode":{"#));
    assert!(!parent.contains(r#""print":{"#));
    assert!(!parent.contains(r#""region":{"#));
    assert!(!parent.contains(r#""retract_overrides":{"#));
}

#[test]
fn every_raw_field_preserves_empty_one_three_five_and_eight_entries() {
    let rows = inventory();
    for row in remaining_rows(&rows) {
        let element = valid_element(&row.key, &row.option_type);
        for length in [0, 1, 3, 5, 8] {
            let values = Value::Array(vec![element.clone(); length]);
            let input = Value::Object(Map::from_iter([(row.key.clone(), values.clone())]));
            let parent: FilamentOptions = serde_json::from_value(input.clone()).unwrap();
            assert_eq!(serde_json::to_value(parent).unwrap()[&row.key], values, "{}", row.key);
            if row.key != "pellet_flow_coefficient" {
                assert_eq!(child_output(row.static_owner.as_str(), input)[&row.key], values);
            }
        }
    }
}

#[test]
fn complete_fixture_parent_byte_round_trip_is_exact() {
    let rows = inventory();
    let keys = rows
        .iter()
        .filter(|row| row.raw_scope == "filament")
        .map(|row| row.key.as_str());
    let fixture = fixture_fields(keys);
    let parsed: FilamentOptions = serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    assert_eq!(serde_json::to_vec(&parsed).unwrap(), serde_json::to_vec(&fixture).unwrap());
}

#[test]
fn deferred_collision_and_legacy_boundary_is_recorded_without_behavior() {
    let rows = inventory();
    let remaining = remaining_rows(&rows)
        .into_iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let complement = BTreeSet::from(PRODUCTION_LITERAL_COMPLEMENT);
    assert_eq!(complement.len(), 3);
    assert!(complement.is_subset(&remaining));
    assert_eq!(remaining.difference(&complement).count(), 66);
    for legacy in [
        r#"{"bridge_fan_speed":["100"]}"#,
        r#"{"cooling":["1"]}"#,
        r#"{"chamber_temperatures":["0"]}"#,
    ] {
        assert!(serde_json::from_str::<FilamentOptions>(legacy).is_err());
    }
}

fn valid_element(key: &str, kind: &str) -> Value {
    json!(match kind {
        "coBools" => "1",
        "coEnums" if key == "overhang_fan_threshold" => "25%",
        "coEnums" if key == "filament_retract_lift_enforce" => "Top Only",
        "coEnums" if key == "filament_z_hop_types" => "Auto Lift",
        "coFloats" => "7.125",
        "coInts" => "7",
        "coPercents" => "37%",
        "coStrings" => "raw task13\n字符串",
        kind => panic!("unexpected type {kind} for {key}"),
    })
}

fn child_output(owner: &str, input: Value) -> Value {
    match owner {
        "print_config" => serde_json::to_value(
            serde_json::from_value::<FilamentPrintSourceOptions>(input).unwrap(),
        )
        .unwrap(),
        "print_region_config" => serde_json::to_value(
            serde_json::from_value::<FilamentRegionSourceOptions>(input).unwrap(),
        )
        .unwrap(),
        "unowned" => serde_json::to_value(
            serde_json::from_value::<FilamentRetractOverrideOptions>(input).unwrap(),
        )
        .unwrap(),
        owner => panic!("unexpected owner {owner}"),
    }
}
