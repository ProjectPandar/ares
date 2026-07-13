use serde_json::{Map, Value, json};

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions,
};
use super::{InventoryRow, expected_default, inventory, remaining_rows};

#[test]
fn every_child_field_dispatches_nondefault_through_child_and_flat_parent() {
    let rows = inventory();
    for row in remaining_rows(&rows)
        .into_iter()
        .filter(|row| row.key != "pellet_flow_coefficient")
    {
        let alternate = alternate(row);
        assert_ne!(alternate, expected_default(row), "{}", row.key);
        let input = Value::Object(Map::from_iter([(row.key.clone(), alternate.clone())]));
        let child = match row.static_owner.as_str() {
            "print_config" => serde_json::to_value(
                serde_json::from_value::<FilamentPrintSourceOptions>(input.clone()).unwrap(),
            )
            .unwrap(),
            "print_region_config" => serde_json::to_value(
                serde_json::from_value::<FilamentRegionSourceOptions>(input.clone()).unwrap(),
            )
            .unwrap(),
            "unowned" => serde_json::to_value(
                serde_json::from_value::<FilamentRetractOverrideOptions>(input.clone()).unwrap(),
            )
            .unwrap(),
            owner => panic!("unexpected owner {owner}"),
        };
        assert_eq!(child[&row.key], alternate, "{}", row.key);
        let parent: FilamentOptions = serde_json::from_value(input).unwrap();
        assert_eq!(serde_json::to_value(parent).unwrap()[&row.key], alternate, "{}", row.key);
    }
}

#[test]
fn direct_pellet_dispatch_and_errors_are_specific() {
    let parsed: FilamentOptions =
        serde_json::from_str(r#"{"pellet_flow_coefficient":["0.5","0.75"]}"#).unwrap();
    assert_eq!(
        serde_json::to_value(parsed).unwrap()["pellet_flow_coefficient"],
        json!(["0.5", "0.75"])
    );
    for invalid in [
        r#"{"pellet_flow_coefficient":["0.5"],"pellet_flow_coefficient":["0.75"]}"#,
        r#"{"pellet_flow_coefficient":"0.5"}"#,
        r#"{"pellet_flow_coefficient":{}}"#,
        r#"{"pellet_flow_coefficient":null}"#,
        r#"{"pellet_flow_coefficient":["bad"]}"#,
    ] {
        let error = serde_json::from_str::<FilamentOptions>(invalid)
            .unwrap_err()
            .to_string();
        assert!(error.contains("pellet_flow_coefficient"), "{error}");
    }
}

fn alternate(row: &InventoryRow) -> Value {
    match row.option_type.as_str() {
        "coBools" => json!(["1", "0", "1"]),
        "coEnums" => json!([match row.key.as_str() {
            "overhang_fan_threshold" => "25%",
            "filament_retract_lift_enforce" => "Top Only",
            "filament_z_hop_types" => "Auto Lift",
            key => panic!("unexpected enum {key}"),
        }]),
        "coFloats" => json!(["7.125", "8.25", "9.5"]),
        "coInts" => json!(["7", "8", "9"]),
        "coPercents" => json!(["37%", "38%", "39%"]),
        "coStrings" => json!(["raw task13 value", "second\nline", "third"]),
        kind => panic!("unexpected option type {kind}"),
    }
}
