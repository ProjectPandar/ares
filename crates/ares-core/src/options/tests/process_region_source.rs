use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value, json};

use super::super::{
    OrcaFloat, OrcaInt, OrcaInts, OrcaStrings, ProcessOptions, ProcessRegionSourceOptions,
    ProjectSettings,
};

mod direct_dispatch;
mod enums;
mod expected;
mod type_assertions;

use expected::{DECLARATION_ORDER, FIXTURE_OVERRIDE_KEYS};

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    nullable: bool,
    default_serialized: String,
    wire_shape: String,
}

#[test]
fn process_region_source_inventory_is_exact_typed_and_disjoint() {
    let rows = inventory();
    let region = region_rows(&rows);
    assert_eq!(region.len(), 149);
    assert_eq!(
        region.iter().fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        }),
        BTreeMap::from([
            ("coBool", 31),
            ("coEnum", 14),
            ("coFloat", 49),
            ("coFloatOrPercent", 24),
            ("coInt", 15),
            ("coInts", 1),
            ("coPercent", 11),
            ("coString", 3),
            ("coStrings", 1),
        ])
    );
    assert!(region.iter().all(|row| !row.nullable));
    assert_eq!(
        region
            .iter()
            .filter(|row| row.wire_shape == "scalar_string")
            .count(),
        147
    );
    assert_eq!(
        region
            .iter()
            .filter(|row| row.wire_shape == "array")
            .count(),
        2
    );
    let region_keys = region
        .iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let object_keys = object_rows(&rows)
        .into_iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert!(region_keys.is_disjoint(&object_keys));
}

#[test]
fn process_region_source_declaration_and_wire_orders_are_exact() {
    let rows = inventory();
    let wire_order = region_rows(&rows)
        .into_iter()
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    assert!(wire_order.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        ProcessRegionSourceOptions::DECLARATION_ORDER,
        DECLARATION_ORDER
    );
    assert_ne!(&DECLARATION_ORDER[..], wire_order.as_slice());
    assert_eq!(
        DECLARATION_ORDER
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        wire_order.iter().copied().collect()
    );
    let serialized = serde_json::to_string(&ProcessRegionSourceOptions::default()).unwrap();
    assert_eq!(serialized_key_order(&serialized), wire_order);
}

#[test]
fn process_region_source_defaults_match_fixed_tag_and_fixture_has_30_overrides() {
    let rows = inventory();
    let region = region_rows(&rows);
    let defaults = serde_json::to_value(ProcessRegionSourceOptions::default()).unwrap();
    let fixture = fixture_fields(region.iter().map(|row| row.key.as_str()));
    for row in &region {
        assert_eq!(defaults[&row.key], expected_default(row), "{}", row.key);
    }
    let changed = region
        .iter()
        .filter(|row| defaults[&row.key] != fixture[&row.key])
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(changed, BTreeSet::from(FIXTURE_OVERRIDE_KEYS));
}

#[test]
fn process_region_source_fixture_round_trips_all_149_fields_byte_for_byte() {
    let rows = inventory();
    let keys = region_rows(&rows)
        .into_iter()
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    let fixture = fixture_fields(keys.iter().copied());
    let parsed: ProcessRegionSourceOptions =
        serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    type_assertions::assert_concrete_types(&parsed);
    let serialized = serde_json::to_string(&parsed).unwrap();
    assert_eq!(serialized.as_bytes(), serde_json::to_vec(&fixture).unwrap());
    assert_eq!(serialized_key_order(&serialized), keys);
}

#[test]
fn process_region_vectors_preserve_arbitrary_lengths_and_reject_wrong_shapes() {
    for ids in [json!([]), json!(["7", "8"]), json!(["1", "2", "3", "4", "5"])] {
        let parsed: ProcessRegionSourceOptions =
            serde_json::from_value(json!({"print_extruder_id": ids.clone()})).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap()["print_extruder_id"], ids);
    }
    for variants in [
        json!([]),
        json!(["Direct Drive Standard", "Bowden High Flow"]),
        json!(["one", "two", "three", "four", "five"]),
    ] {
        let parsed: ProcessRegionSourceOptions =
            serde_json::from_value(json!({"print_extruder_variant": variants.clone()})).unwrap();
        assert_eq!(
            serde_json::to_value(parsed).unwrap()["print_extruder_variant"],
            variants
        );
    }
    for input in [
        json!({"print_extruder_id": "1"}),
        json!({"print_extruder_id": ["bad"]}),
        json!({"print_extruder_variant": "Direct Drive Standard"}),
        json!({"print_extruder_variant": [7]}),
    ] {
        assert!(serde_json::from_value::<ProcessRegionSourceOptions>(input).is_err());
    }
}

#[test]
fn region_and_parent_decode_errors_are_keyed_strict_and_scope_aware() {
    for (json, key, source) in [
        (r#"{"wall_loops":[]}"#, "wall_loops", "invalid type"),
        (
            r#"{"wall_sequence":"Outer/Inner"}"#,
            "wall_sequence",
            "unknown variant",
        ),
        (
            r#"{"print_extruder_id":["bad"]}"#,
            "print_extruder_id",
            "invalid digit",
        ),
    ] {
        let error = serde_json::from_str::<ProcessRegionSourceOptions>(json)
            .unwrap_err()
            .to_string();
        assert!(error.contains(key), "{error}");
        assert!(error.contains(source), "{error}");
    }
    for json in [
        r#"{"layer_height":"0.3"}"#,
        r#"{"wall_loops":"2","wall_loops":"3"}"#,
        r#"{"not_an_orca_option":"0"}"#,
    ] {
        assert!(serde_json::from_str::<ProcessRegionSourceOptions>(json).is_err());
    }

    let rows = inventory();
    let remaining = rows
        .iter()
        .filter(|row| {
            row.raw_scope == "process"
                && row.static_owner != "print_object_config"
                && row.static_owner != "print_region_config"
        })
        .collect::<Vec<_>>();
    assert_eq!(remaining.len(), 77);
    for row in remaining {
        let fixture = fixture_fields([row.key.as_str()]);
        assert!(
            serde_json::from_value::<ProcessRegionSourceOptions>(Value::Object(fixture.clone()))
                .is_err(),
            "{}",
            row.key
        );
        serde_json::from_value::<ProcessOptions>(Value::Object(fixture)).unwrap();
    }
}

#[test]
fn process_parent_flattens_all_four_sources_in_global_lexicographic_order() {
    let rows = inventory();
    let keys = rows
        .iter()
        .filter(|row| row.raw_scope == "process")
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys.len(), 352);
    let fixture = fixture_fields(keys.iter().copied());
    let process: ProcessOptions =
        serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    assert_eq!(process.object.layer_height, OrcaFloat(0.2));
    assert_eq!(process.region.wall_loops, OrcaInt(2));
    let serialized = serde_json::to_string(&process).unwrap();
    assert_eq!(serialized.as_bytes(), serde_json::to_vec(&fixture).unwrap());
    assert_eq!(serialized_key_order(&serialized), keys);
    assert!(!serialized.contains(r#""object":{"#));
    assert!(!serialized.contains(r#""region":{"#));
}

#[test]
fn process_parent_accepts_mixed_children_and_rejects_duplicate_across_flat_map() {
    let process: ProcessOptions =
        serde_json::from_str(r#"{"wall_loops":"7","layer_height":"0.3","print_extruder_id":["2","3"]}"#)
            .unwrap();
    assert_eq!(process.object.layer_height, OrcaFloat(0.3));
    assert_eq!(process.region.wall_loops, OrcaInt(7));
    assert_eq!(
        process.region.print_extruder_id,
        OrcaInts(vec![OrcaInt(2), OrcaInt(3)])
    );
    assert!(
        serde_json::from_str::<ProcessOptions>(
            r#"{"wall_loops":"2","wall_loops":"3","layer_height":"0.2"}"#
        )
        .is_err()
    );
}

#[test]
fn project_settings_exposes_typed_process_region_child() {
    let process = &ProjectSettings::default().process;
    assert_eq!(process.region.wall_loops, OrcaInt(2));
    assert_eq!(
        process.region.print_extruder_id,
        OrcaInts(vec![OrcaInt(1)])
    );
    assert_eq!(
        process.region.print_extruder_variant,
        OrcaStrings(vec!["Direct Drive Standard".to_owned()])
    );
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn region_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "process" && row.static_owner == "print_region_config")
        .collect()
}

fn object_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "process" && row.static_owner == "print_object_config")
        .collect()
}

fn expected_default(row: &InventoryRow) -> Value {
    match row.key.as_str() {
        "print_extruder_id" => json!(["1"]),
        "print_extruder_variant" => json!(["Direct Drive Standard"]),
        _ => Value::String(row.default_serialized.clone()),
    }
}

fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let fixture = super::project_fixture::project_settings_value();
    let fixture = fixture.as_object().unwrap();
    keys.into_iter()
        .map(|key| (key.to_owned(), fixture[key].clone()))
        .collect()
}

fn serialized_key_order(serialized: &str) -> Vec<String> {
    serde_json::from_str::<SerializedKeys>(serialized).unwrap().0
}

struct SerializedKeys(Vec<String>);

impl<'de> Deserialize<'de> for SerializedKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(SerializedKeysVisitor)
    }
}

struct SerializedKeysVisitor;

impl<'de> serde::de::Visitor<'de> for SerializedKeysVisitor {
    type Value = SerializedKeys;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut order = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            order.push(key);
            map.next_value::<IgnoredAny>()?;
        }
        Ok(SerializedKeys(order))
    }
}
