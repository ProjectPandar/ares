use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions,
};
use super::expected::{FIXTURE_CONCRETE_NULLABLE_KEYS, FIXTURE_NIL_KEYS, NULLABLE_KEYS};
use super::{fixture_fields, inventory, remaining_rows};

#[test]
fn exact_twenty_nullable_fields_accept_mixed_nil_without_normalization() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    assert_eq!(
        remaining
            .iter()
            .filter(|row| row.nullable)
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        NULLABLE_KEYS
    );
    for row in remaining.iter().filter(|row| row.nullable) {
        let concrete = match row.option_type.as_str() {
            "coBools" => "1",
            "coEnums" if row.key == "filament_retract_lift_enforce" => "Top Only",
            "coEnums" if row.key == "filament_z_hop_types" => "Normal Lift",
            "coFloats" => "7.125",
            "coPercents" => "37%",
            kind => panic!("unexpected nullable {kind} for {}", row.key),
        };
        let values = json!(["nil", concrete, "nil"]);
        let input = Value::Object(Map::from_iter([(row.key.clone(), values.clone())]));
        assert_eq!(child_output(row.static_owner.as_str(), input.clone())[&row.key], values);
        let parent: FilamentOptions = serde_json::from_value(input).unwrap();
        assert_eq!(serde_json::to_value(parent).unwrap()[&row.key], values);
    }
}

#[test]
fn fixture_nullable_payloads_are_exactly_fifteen_nil_and_five_concrete() {
    let fixture = fixture_fields(NULLABLE_KEYS);
    let fully_nil = fixture
        .iter()
        .filter(|(_, value)| value.as_array().unwrap().iter().all(|value| value == "nil"))
        .map(|(key, _)| key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(fully_nil, BTreeSet::from(FIXTURE_NIL_KEYS));
    assert_eq!(
        BTreeSet::from(NULLABLE_KEYS).difference(&fully_nil).copied().collect::<BTreeSet<_>>(),
        BTreeSet::from(FIXTURE_CONCRETE_NULLABLE_KEYS)
    );
}

#[test]
fn exact_48_non_string_nonnullable_fields_reject_nil_but_notes_keeps_text() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    let rejected = remaining
        .iter()
        .filter(|row| !row.nullable && row.option_type != "coStrings")
        .collect::<Vec<_>>();
    assert_eq!(rejected.len(), 48);
    for row in rejected {
        let input = Value::Object(Map::from_iter([(row.key.clone(), json!(["nil"]))]));
        let parent = serde_json::from_value::<FilamentOptions>(input.clone())
            .unwrap_err()
            .to_string();
        assert!(parent.contains(&row.key), "{}: {parent}", row.key);
        if row.key != "pellet_flow_coefficient" {
            let child = child_error(row.static_owner.as_str(), input);
            assert!(child.contains(&row.key), "{}: {child}", row.key);
        }
    }
    let notes = json!(["nil", "", "line one\n第二行"]);
    let parsed: FilamentOptions =
        serde_json::from_value(json!({"filament_notes": notes.clone()})).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()["filament_notes"], notes);
}

fn child_output(owner: &str, input: Value) -> Value {
    match owner {
        "print_region_config" => serde_json::to_value(
            serde_json::from_value::<FilamentRegionSourceOptions>(input).unwrap(),
        )
        .unwrap(),
        "unowned" => serde_json::to_value(
            serde_json::from_value::<FilamentRetractOverrideOptions>(input).unwrap(),
        )
        .unwrap(),
        owner => panic!("unexpected nullable owner {owner}"),
    }
}

fn child_error(owner: &str, input: Value) -> String {
    match owner {
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
