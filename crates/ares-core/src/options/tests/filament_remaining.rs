use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value};

use super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRegionSourceOptions,
    FilamentRetractOverrideOptions, OrcaFloat, ProjectSettings,
};

mod direct_dispatch;
mod enums;
mod expected;
mod fixture;
mod invalid;
mod inventory_defaults;
mod nullable;
mod type_assertions;
mod wire;

use expected::{
    PRINT_DECLARATION_ORDER, REGION_DECLARATION_ORDER, REMAINING_LEXICAL_KEYS,
    RETRACT_DECLARATION_ORDER,
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
fn remaining_inventory_is_exact_disjoint_all_array_and_partitioned() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    assert_eq!(remaining.len(), 69);
    assert_eq!(
        remaining.iter().map(|row| row.key.as_str()).collect::<Vec<_>>(),
        REMAINING_LEXICAL_KEYS
    );
    assert_eq!(remaining.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>().len(), 69);
    assert!(remaining.iter().all(|row| row.wire_shape == "array"));
    assert_eq!(
        remaining.iter().fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        }),
        BTreeMap::from([
            ("coBools", 11),
            ("coEnums", 3),
            ("coFloats", 20),
            ("coInts", 30),
            ("coPercents", 4),
            ("coStrings", 1),
        ])
    );
    assert_eq!(owner_rows(&remaining, "print_config").len(), 48);
    assert_eq!(owner_rows(&remaining, "print_region_config").len(), 4);
    assert_eq!(owner_rows(&remaining, "unowned").len(), 17);
    let task12 = rows
        .iter()
        .filter(|row| row.raw_scope == "filament" && row.static_owner == "g_code_config")
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let task13 = remaining.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    assert!(task12.is_disjoint(&task13));
    assert_eq!(task12.len() + task13.len(), 122);
}

#[test]
fn fixed_child_declaration_orders_are_exact() {
    assert_eq!(FilamentPrintSourceOptions::DECLARATION_ORDER, PRINT_DECLARATION_ORDER);
    assert_eq!(FilamentRegionSourceOptions::DECLARATION_ORDER, REGION_DECLARATION_ORDER);
    assert_eq!(
        FilamentRetractOverrideOptions::DECLARATION_ORDER,
        RETRACT_DECLARATION_ORDER
    );
}

#[test]
fn aggregate_exposes_remaining_children_direct_pellet_and_122_flat_keys() {
    let settings = ProjectSettings::default();
    let filament = settings.filament;
    let _: FilamentPrintSourceOptions = filament.print;
    let _: FilamentRegionSourceOptions = filament.region;
    let _: FilamentRetractOverrideOptions = filament.retract_overrides;
    assert_eq!(filament.pellet_flow_coefficient.0, vec![OrcaFloat(0.4157)]);

    let serialized = serde_json::to_value(FilamentOptions::default()).unwrap();
    let object = serialized.as_object().unwrap();
    assert_eq!(object.len(), 122);
    assert!(object.get("adaptive_pressure_advance").is_some());
    assert_eq!(object["pellet_flow_coefficient"], serde_json::json!(["0.4157"]));
    assert!(object.get("textured_plate_temp_initial_layer").is_some());
    assert!(matches!(serialized, Value::Object(_)));
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn remaining_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "filament" && row.static_owner != "g_code_config")
        .collect()
}

fn owner_rows<'a>(rows: &[&'a InventoryRow], owner: &str) -> Vec<&'a InventoryRow> {
    rows.iter().copied().filter(|row| row.static_owner == owner).collect()
}

fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let fixture = super::project_fixture::project_settings_value();
    let fixture = fixture.as_object().unwrap();
    keys.into_iter()
        .map(|key| (key.to_owned(), fixture[key].clone()))
        .collect()
}

fn expected_default(row: &InventoryRow) -> Value {
    if row.nullable {
        return Value::Array(vec![Value::String("nil".to_owned())]);
    }
    let scalar = match row.option_type.as_str() {
        "coBools" | "coEnums" | "coFloats" | "coInts" | "coPercents" => {
            row.default_serialized.clone()
        }
        "coStrings" if row.default_serialized.starts_with('"') => {
            serde_json::from_str(&row.default_serialized).unwrap()
        }
        kind => panic!("unexpected option type {kind}"),
    };
    Value::Array(vec![Value::String(scalar)])
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
