use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value};

use super::super::{FilamentGCodeSourceOptions, FilamentOptions, ProjectSettings};

mod direct_dispatch;
mod expected;
mod type_assertions;
mod vectors;

use expected::{
    DECLARATION_ORDER, FIXTURE_OVERRIDE_KEYS, LEXICAL_KEYS, NULLABLE_KEYS, VARIANT_KEYS,
};

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
fn filament_gcode_source_inventory_is_exact() {
    let inventory = inventory();
    let rows = filament_rows(&inventory);
    assert_eq!(rows.len(), 53);
    assert_eq!(rows.iter().map(|row| row.key.as_str()).collect::<Vec<_>>(), LEXICAL_KEYS);
    assert_eq!(rows.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>().len(), 53);
    assert!(rows.iter().all(|row| row.wire_shape == "array"));
    assert_eq!(
        rows.iter().fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        }),
        BTreeMap::from([("coBools", 8), ("coFloats", 27), ("coInts", 7), ("coStrings", 11)])
    );
    assert_eq!(
        rows.iter().filter(|row| row.nullable).map(|row| row.key.as_str()).collect::<Vec<_>>(),
        NULLABLE_KEYS
    );
}

#[test]
fn declaration_order_and_lexical_wire_order_are_distinct_and_exact() {
    assert_eq!(FilamentGCodeSourceOptions::DECLARATION_ORDER, DECLARATION_ORDER);
    assert_eq!(
        DECLARATION_ORDER.iter().copied().collect::<BTreeSet<_>>(),
        LEXICAL_KEYS.iter().copied().collect()
    );
    assert_ne!(DECLARATION_ORDER, LEXICAL_KEYS);
    let child = serde_json::to_string(&FilamentGCodeSourceOptions::default()).unwrap();
    let parent = serde_json::to_string(&FilamentOptions::default()).unwrap();
    assert_eq!(serialized_key_order(&child), LEXICAL_KEYS);
    assert_eq!(
        serialized_key_order(&parent),
        inventory()
            .into_iter()
            .filter(|row| row.raw_scope == "filament")
            .map(|row| row.key)
            .collect::<Vec<_>>()
    );
    assert!(!parent.contains(r#""gcode":{"#));
}

#[test]
fn exact_singleton_defaults_match_fixed_tag() {
    let defaults = serde_json::to_value(FilamentGCodeSourceOptions::default()).unwrap();
    for row in filament_rows(&inventory()) {
        assert_eq!(defaults[&row.key], expected_default(row), "{}", row.key);
    }
    assert_eq!(defaults["filament_start_gcode"], serde_json::json!([" "]));
    assert_eq!(defaults["filament_end_gcode"], serde_json::json!([" "]));
    assert_eq!(
        defaults["adaptive_pressure_advance_model"],
        serde_json::json!(["0,0,0\n0,0,0"])
    );
}

#[test]
fn fixture_preserves_all_source_vectors_and_exact_bytes() {
    let fixture = fixture_fields(LEXICAL_KEYS);
    let child: FilamentGCodeSourceOptions =
        serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    let parent: FilamentOptions = serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    type_assertions::assert_concrete_types(&child);
    assert_eq!(serde_json::to_vec(&child).unwrap(), serde_json::to_vec(&fixture).unwrap());
    let parent = serde_json::to_value(parent).unwrap();
    for key in LEXICAL_KEYS {
        assert_eq!(parent[key], fixture[key], "{key}");
    }

    let lengths = fixture.values().map(|value| value.as_array().unwrap().len()).fold(
        BTreeMap::new(),
        |mut counts, length| {
            *counts.entry(length).or_insert(0) += 1;
            counts
        },
    );
    assert_eq!(lengths, BTreeMap::from([(2, 43), (8, 10)]));
    assert_eq!(
        fixture
            .iter()
            .filter(|(_, value)| value.as_array().unwrap().len() == 8)
            .map(|(key, _)| key.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(VARIANT_KEYS)
    );

    let defaults = serde_json::to_value(FilamentGCodeSourceOptions::default()).unwrap();
    let overrides = LEXICAL_KEYS
        .iter()
        .filter(|key| {
            fixture[**key]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value != &defaults[**key][0])
        })
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(overrides, BTreeSet::from(FIXTURE_OVERRIDE_KEYS));
}

#[test]
fn aggregate_exposes_filament_gcode_source_options() {
    let settings = ProjectSettings::default();
    assert_eq!(settings.filament, FilamentOptions::default());
    let _: FilamentGCodeSourceOptions = settings.filament.gcode;
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn filament_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "filament" && row.static_owner == "g_code_config")
        .collect()
}

fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let fixture = super::project_fixture::project_settings_value();
    let fixture = fixture.as_object().unwrap();
    keys.into_iter()
        .map(|key| (key.to_owned(), fixture[key].clone()))
        .collect()
}

fn expected_default(row: &InventoryRow) -> Value {
    let value = match row.option_type.as_str() {
        "coBools" | "coFloats" | "coInts" => row.default_serialized.clone(),
        "coStrings" if row.default_serialized.starts_with('"') => {
            serde_json::from_str(&row.default_serialized).unwrap()
        }
        "coStrings" => row.default_serialized.clone(),
        kind => panic!("unexpected option type {kind}"),
    };
    Value::Array(vec![Value::String(value)])
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
