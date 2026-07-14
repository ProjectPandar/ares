use std::collections::BTreeSet;

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value};

use super::super::{
    PresetMetadata, ProjectGCodeSourceOptions, ProjectPrintSourceOptions,
    ProjectPresetSourceOptions, ProjectRuntimeOptions,
};

mod direct_dispatch;
mod enums;
mod expected;
mod fixture;
mod invalid;
mod inventory_defaults;
mod metadata;
mod type_assertions;
mod wire;

use expected::{Child, ExpectedField, REAL_FIELDS};

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    nullable: bool,
    wire_shape: String,
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn residual_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter().filter(|row| row.raw_scope == "residual").collect()
}

fn real_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    residual_rows(rows)
        .into_iter()
        .filter(|row| row.option_type != "Metadata")
        .collect()
}

fn fixture() -> Map<String, Value> {
    super::project_fixture::project_settings_value()
        .as_object()
        .unwrap()
        .clone()
}

fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let fixture = fixture();
    keys.into_iter()
        .map(|key| (key.to_owned(), fixture[key].clone()))
        .collect()
}

fn expected_default(field: &ExpectedField) -> Value {
    serde_json::from_str(field.default_json).unwrap()
}

fn expected_defaults(child: Option<Child>) -> Map<String, Value> {
    REAL_FIELDS
        .iter()
        .filter(|field| child.is_none_or(|child| field.child == child))
        .map(|field| (field.key.to_owned(), expected_default(field)))
        .collect()
}

fn child_output(field: &ExpectedField, input: Value) -> Value {
    match field.child {
        Child::GCode => serde_json::to_value(
            serde_json::from_value::<ProjectGCodeSourceOptions>(input).unwrap(),
        )
        .unwrap(),
        Child::Print => serde_json::to_value(
            serde_json::from_value::<ProjectPrintSourceOptions>(input).unwrap(),
        )
        .unwrap(),
        Child::Preset => serde_json::to_value(
            serde_json::from_value::<ProjectPresetSourceOptions>(input).unwrap(),
        )
        .unwrap(),
    }
}

fn child_error(field: &ExpectedField, input: Value) -> String {
    match field.child {
        Child::GCode => serde_json::from_value::<ProjectGCodeSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        Child::Print => serde_json::from_value::<ProjectPrintSourceOptions>(input)
            .unwrap_err()
            .to_string(),
        Child::Preset => serde_json::from_value::<ProjectPresetSourceOptions>(input)
            .unwrap_err()
            .to_string(),
    }
}

fn parent_output(input: Value) -> Value {
    serde_json::to_value(serde_json::from_value::<ProjectRuntimeOptions>(input).unwrap()).unwrap()
}

fn parent_error(input: Value) -> String {
    serde_json::from_value::<ProjectRuntimeOptions>(input)
        .unwrap_err()
        .to_string()
}

fn metadata_output(input: Value) -> Value {
    serde_json::to_value(serde_json::from_value::<PresetMetadata>(input).unwrap()).unwrap()
}

fn serialized_key_order(serialized: &str) -> Vec<String> {
    serde_json::from_str::<SerializedKeys>(serialized).unwrap().0
}

fn assert_keyed_bounded_error(error: &str, key: &str) {
    assert!(error.contains(key), "{key}: {error}");
    assert!(error.len() < 1_024, "unbounded diagnostic for {key}: {error}");
}

fn assert_pairwise_disjoint(sets: &[BTreeSet<&str>]) {
    for (index, set) in sets.iter().enumerate() {
        for other in &sets[index + 1..] {
            assert!(set.is_disjoint(other));
        }
    }
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
