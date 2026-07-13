use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value, json};

use super::super::{
    OrcaBool, OrcaFloat, ProcessGCodeSourceOptions, ProcessOptions, ProcessPrintSourceOptions,
    ProjectSettings,
};

mod direct_dispatch;
mod enums;
mod expected;
mod type_assertions;
mod vectors;

use expected::{
    FIXTURE_OVERRIDE_KEYS, GCODE_DECLARATION_ORDER, PRINT_DECLARATION_ORDER,
    PRODUCTION_LITERAL_COMPLEMENT,
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
fn process_remaining_inventory_completes_exact_352_key_scope() {
    let rows = inventory();
    let process = process_rows(&rows);
    let object = owned_rows(&process, "print_object_config");
    let region = owned_rows(&process, "print_region_config");
    let gcode = owned_rows(&process, "g_code_config");
    let print = owned_rows(&process, "print_config");
    let unowned = owned_rows(&process, "unowned");
    assert_eq!(
        [object.len(), region.len(), gcode.len(), print.len(), unowned.len()],
        [126, 149, 17, 59, 1]
    );
    let sets = [&object, &region, &gcode, &print, &unowned].map(|rows| {
        rows.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>()
    });
    for (index, set) in sets.iter().enumerate() {
        for other in &sets[index + 1..] {
            assert!(set.is_disjoint(other));
        }
    }
    assert_eq!(sets.iter().map(BTreeSet::len).sum::<usize>(), 352);

    let remaining = remaining_rows(&rows);
    assert_eq!(remaining.len(), 77);
    assert_eq!(
        remaining.iter().fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        }),
        BTreeMap::from([
            ("coBool", 25),
            ("coEnum", 6),
            ("coFloat", 24),
            ("coFloatOrPercent", 6),
            ("coFloats", 1),
            ("coInt", 6),
            ("coPercent", 4),
            ("coString", 3),
            ("coStrings", 2),
        ])
    );
    assert!(remaining.iter().all(|row| !row.nullable));
    assert_eq!(remaining.iter().filter(|row| row.wire_shape == "scalar_string").count(), 74);
    assert_eq!(
        remaining
            .iter()
            .filter(|row| row.wire_shape == "array")
            .map(|row| row.key.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "post_process",
            "small_area_infill_flow_compensation_model",
            "wiping_volumes_extruders",
        ])
    );
}

#[test]
fn process_remaining_declaration_and_wire_orders_are_exact() {
    let rows = inventory();
    let gcode_keys = owner_keys(&rows, "g_code_config");
    let print_keys = owner_keys(&rows, "print_config");
    assert_eq!(ProcessGCodeSourceOptions::DECLARATION_ORDER, GCODE_DECLARATION_ORDER);
    assert_eq!(ProcessPrintSourceOptions::DECLARATION_ORDER, PRINT_DECLARATION_ORDER);
    assert_eq!(GCODE_DECLARATION_ORDER.iter().copied().collect::<BTreeSet<_>>(), gcode_keys.iter().copied().collect());
    assert_eq!(PRINT_DECLARATION_ORDER.iter().copied().collect::<BTreeSet<_>>(), print_keys.iter().copied().collect());
    assert_ne!(GCODE_DECLARATION_ORDER.as_slice(), gcode_keys.as_slice());
    assert_ne!(PRINT_DECLARATION_ORDER.as_slice(), print_keys.as_slice());
    assert_eq!(serialized_key_order(&serde_json::to_string(&ProcessGCodeSourceOptions::default()).unwrap()), gcode_keys);
    assert_eq!(serialized_key_order(&serde_json::to_string(&ProcessPrintSourceOptions::default()).unwrap()), print_keys);
}

#[test]
fn process_remaining_defaults_match_fixed_tag_and_fixture_has_15_overrides() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    let mut defaults = serde_json::to_value(ProcessGCodeSourceOptions::default()).unwrap();
    defaults.as_object_mut().unwrap().extend(
        serde_json::to_value(ProcessPrintSourceOptions::default()).unwrap().as_object().unwrap().clone(),
    );
    defaults.as_object_mut().unwrap().insert("ironing_expansion".to_owned(), json!("0"));
    let fixture = fixture_fields(remaining.iter().map(|row| row.key.as_str()));
    for row in &remaining {
        assert_eq!(defaults[&row.key], expected_default(row), "{}", row.key);
    }
    let changed = remaining
        .iter()
        .filter(|row| defaults[&row.key] != fixture[&row.key])
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(changed, BTreeSet::from(FIXTURE_OVERRIDE_KEYS));
}

#[test]
fn standalone_children_and_flat_parent_round_trip_exact_lexical_bytes() {
    let rows = inventory();
    let gcode_keys = owner_keys(&rows, "g_code_config");
    let print_keys = owner_keys(&rows, "print_config");
    let gcode_fixture = fixture_fields(gcode_keys.iter().copied());
    let print_fixture = fixture_fields(print_keys.iter().copied());
    let gcode: ProcessGCodeSourceOptions = serde_json::from_value(Value::Object(gcode_fixture.clone())).unwrap();
    let print: ProcessPrintSourceOptions = serde_json::from_value(Value::Object(print_fixture.clone())).unwrap();
    type_assertions::assert_gcode_types(&gcode);
    type_assertions::assert_print_types(&print);
    assert_eq!(serde_json::to_vec(&gcode).unwrap(), serde_json::to_vec(&gcode_fixture).unwrap());
    assert_eq!(serde_json::to_vec(&print).unwrap(), serde_json::to_vec(&print_fixture).unwrap());

    let process_keys = process_rows(&rows).into_iter().map(|row| row.key.as_str()).collect::<Vec<_>>();
    let fixture = fixture_fields(process_keys.iter().copied());
    let process: ProcessOptions = serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    assert_eq!(process.ironing_expansion, OrcaFloat(0.0));
    assert_eq!(serde_json::to_vec(&process).unwrap(), serde_json::to_vec(&fixture).unwrap());
    let serialized = serde_json::to_string(&process).unwrap();
    assert_eq!(serialized_key_order(&serialized), process_keys);
    assert!(!serialized.contains(r#""gcode":{"#));
    assert!(!serialized.contains(r#""print":{"#));
}

#[test]
fn project_settings_exposes_all_process_children_and_direct_scalar() {
    let process = &ProjectSettings::default().process;
    assert_eq!(process.gcode.enable_arc_fitting, OrcaBool(false));
    assert_eq!(process.print.enable_prime_tower, OrcaBool(false));
    assert_eq!(process.ironing_expansion, OrcaFloat(0.0));
}

#[test]
fn deferred_production_literal_collision_set_is_recorded_exactly() {
    let rows = inventory();
    let all = remaining_rows(&rows)
        .into_iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let complement = BTreeSet::from(PRODUCTION_LITERAL_COMPLEMENT);
    assert_eq!(complement.len(), 14);
    assert!(complement.is_subset(&all));
    let collisions = all.difference(&complement).copied().collect::<BTreeSet<_>>();
    assert_eq!(collisions.len(), 63);
    assert!(collisions.contains("prime_volume"));
    assert_eq!(collisions.iter().filter(|key| **key != "prime_volume").count(), 62);
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!("../../../../../tests/ksr_fdmtest_v4/options-v242.json")).unwrap()
}

fn process_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter().filter(|row| row.raw_scope == "process").collect()
}

fn remaining_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    process_rows(rows)
        .into_iter()
        .filter(|row| !matches!(row.static_owner.as_str(), "print_object_config" | "print_region_config"))
        .collect()
}

fn owned_rows<'a>(rows: &[&'a InventoryRow], owner: &str) -> Vec<&'a InventoryRow> {
    rows.iter().copied().filter(|row| row.static_owner == owner).collect()
}

fn owner_keys<'a>(rows: &'a [InventoryRow], owner: &str) -> Vec<&'a str> {
    rows.iter()
        .filter(|row| row.raw_scope == "process" && row.static_owner == owner)
        .map(|row| row.key.as_str())
        .collect()
}

fn expected_default(row: &InventoryRow) -> Value {
    match row.key.as_str() {
        "post_process" => json!([]),
        "small_area_infill_flow_compensation_model" => json!([
            "0,0", "\n0.2,0.4444", "\n0.4,0.6145", "\n0.6,0.7059", "\n0.8,0.7619",
            "\n1.5,0.8571", "\n2,0.8889", "\n3,0.9231", "\n5,0.9520", "\n10,1"
        ]),
        "wiping_volumes_extruders" => json!(["70", "70", "70", "70", "70", "70", "70", "70", "70", "70"]),
        _ => Value::String(row.default_serialized.clone()),
    }
}

fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let project = crate::load_project(include_bytes!("../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf")).unwrap();
    let fixture: Value = serde_json::from_slice(project.project_settings_bytes()).unwrap();
    let fixture = fixture.as_object().unwrap();
    keys.into_iter().map(|key| (key.to_owned(), fixture[key].clone())).collect()
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
