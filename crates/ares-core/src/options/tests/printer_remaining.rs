use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value};

use super::super::{
    Nullable, OrcaFloat, PrinterOptions, PrinterRemainingOptions,
};

mod enums;
mod expected;
mod structured;
mod type_assertions;

use expected::{PRINT_CONFIG_ORDER, REMAINING_KEYS, RUNTIME_ORDER};

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
fn printer_options_remaining_inventory_is_exact_and_completes_132_key_union() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    assert_eq!(remaining.len(), 42);
    assert_eq!(remaining.iter().filter(|row| row.static_owner == "print_config").count(), 27);
    assert_eq!(remaining.iter().filter(|row| row.static_owner == "unowned").count(), 15);
    assert_eq!(remaining.iter().filter(|row| row.nullable).map(|row| row.key.as_str()).collect::<BTreeSet<_>>(), BTreeSet::from(["extruder_printable_height", "nozzle_volume"]));
    assert_eq!(remaining.iter().filter(|row| row.wire_shape == "array").count(), 11);
    assert_eq!(remaining.iter().filter(|row| row.wire_shape == "scalar_string").count(), 31);

    let histogram = remaining.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
        counts
    });
    assert_eq!(histogram, BTreeMap::from([
        ("coBool", 4), ("coEnum", 4), ("coEnums", 1), ("coFloat", 8),
        ("coFloats", 3), ("coInt", 1), ("coPoint", 4), ("coPoints", 3),
        ("coPointsGroups", 1), ("coString", 10), ("coStrings", 3),
    ]));

    let machine = rows.iter().filter(|row| row.raw_scope == "printer" && row.static_owner == "machine_envelope_config").map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    let gcode = rows.iter().filter(|row| row.raw_scope == "printer" && row.static_owner == "g_code_config").map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    let remaining = remaining.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    assert_eq!((machine.len(), gcode.len(), remaining.len()), (28, 62, 42));
    assert!(machine.is_disjoint(&gcode) && machine.is_disjoint(&remaining) && gcode.is_disjoint(&remaining));
    let mut union = machine.clone();
    union.extend(gcode.iter().copied());
    union.extend(remaining.iter().copied());
    let printer = rows.iter().filter(|row| row.raw_scope == "printer").map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(union, printer);
    assert_eq!(printer.len(), 132);
    let ams = rows.iter().find(|row| row.key == "extruder_ams_count").unwrap();
    assert_eq!(ams.raw_scope, "residual");
    assert!(!printer.contains("extruder_ams_count"));
}

#[test]
fn printer_options_remaining_fixture_round_trips_all_42_fields_byte_for_byte() {
    let fixture = fixture_fields(REMAINING_KEYS);
    let parsed: PrinterRemainingOptions = serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    type_assertions::assert_concrete_types(&parsed);
    assert_eq!(serde_json::to_vec(&parsed).unwrap(), serde_json::to_vec(&fixture).unwrap());
}

#[test]
fn printer_options_full_fixture_round_trips_all_132_fields_flat_and_ordered() {
    let rows = inventory();
    let keys = rows.iter().filter(|row| row.raw_scope == "printer").map(|row| row.key.as_str()).collect::<Vec<_>>();
    let fixture = fixture_fields(keys.iter().copied());
    let parsed: PrinterOptions = serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    let serialized = serde_json::to_string(&parsed).unwrap();
    assert_eq!(serialized.as_bytes(), serde_json::to_vec(&fixture).unwrap());
    assert_eq!(serialized_key_order(&serialized), keys);
    assert!(!serialized.contains(r#""machine":{"#));
    assert!(!serialized.contains(r#""gcode":{"#));
    assert!(!serialized.contains(r#""remaining":{"#));
}

#[test]
fn printer_options_remaining_defaults_match_fixed_tag() {
    let rows = inventory();
    let remaining = remaining_rows(&rows);
    let defaults = serde_json::to_value(PrinterRemainingOptions::default()).unwrap();
    for row in &remaining {
        assert_eq!(defaults[&row.key], expected_default(row), "{}", row.key);
    }
    let fixture = fixture_fields(REMAINING_KEYS);
    assert_eq!(remaining.iter().filter(|row| defaults[&row.key] != fixture[&row.key]).count(), 18);
    assert_eq!(defaults["bed_exclude_area"], serde_json::json!(["0x0"]));
    assert_eq!(fixture["bed_exclude_area"], serde_json::json!([]));
    assert_eq!(defaults["best_object_pos"], "0.5,0.5");
    assert_eq!(defaults["printable_area"], serde_json::json!(["0x0", "200x0", "200x200", "0x200"]));
}

#[test]
fn printer_options_declaration_and_registration_orders_are_separate_from_export_order() {
    assert_eq!(PrinterRemainingOptions::PRINT_CONFIG_DECLARATION_ORDER, PRINT_CONFIG_ORDER);
    assert_eq!(PrinterRemainingOptions::RUNTIME_REGISTRATION_ORDER, RUNTIME_ORDER);
    assert!(REMAINING_KEYS.windows(2).all(|pair| pair[0] < pair[1]));
    assert_ne!(&PRINT_CONFIG_ORDER[..], &REMAINING_KEYS[..27]);
}

#[test]
fn printer_options_flat_dispatch_keeps_three_children_independent() {
    let options: PrinterOptions = serde_json::from_str(r#"{"machine_max_speed_x":["321","123"],"gcode_flavor":"klipper","printable_height":"261","host_type":"moonraker"}"#).unwrap();
    assert_eq!(options.machine.machine_max_speed_x.0[0], OrcaFloat(321.0));
    assert_eq!(serde_json::to_value(options.gcode.gcode_flavor).unwrap(), "klipper");
    assert_eq!(options.remaining.printable_height, OrcaFloat(261.0));
    assert_eq!(serde_json::to_value(options.remaining.host_type).unwrap(), "moonraker");
}

#[test]
fn printer_options_preserves_fixture_extruder_variant_and_area_cardinalities() {
    let rows = inventory();
    let keys = rows.iter().filter(|row| row.raw_scope == "printer").map(|row| row.key.as_str());
    let options: PrinterOptions = serde_json::from_value(Value::Object(fixture_fields(keys))).unwrap();
    assert_eq!(options.machine.machine_max_speed_x.0.len(), 8);
    assert_eq!(options.gcode.extruder_type.0.len(), 2);
    assert_eq!(options.remaining.default_nozzle_volume_type.0.len(), 2);
    assert_eq!(options.remaining.extruder_printable_area.0.len(), 2);
    assert_eq!(options.remaining.extruder_printable_height.0.len(), 2);
    assert_eq!(options.remaining.extruder_variant_list.0.len(), 2);
    assert!(options.remaining.extruder_variant_list.0.iter().all(|variants| variants.split(',').count() == 2));
    assert_eq!(options.gcode.printer_extruder_id.0.iter().map(|id| id.0).collect::<Vec<_>>(), [1, 1, 2, 2]);
    assert_eq!(options.remaining.nozzle_volume.0.len(), 4);
    assert_eq!(options.remaining.printable_area.0.len(), 4);
    assert_eq!(options.remaining.upward_compatible_machine.0.len(), 6);
}

#[test]
fn printer_options_nullable_float_vectors_are_element_nullable() {
    let value: PrinterRemainingOptions = serde_json::from_str(r#"{"extruder_printable_height":["nil","256"],"nozzle_volume":["nil","92"]}"#).unwrap();
    assert_eq!(value.extruder_printable_height.0, [Nullable::Nil, Nullable::Value(OrcaFloat(256.0))]);
    assert_eq!(value.nozzle_volume.0, [Nullable::Nil, Nullable::Value(OrcaFloat(92.0))]);
    assert_eq!(serde_json::to_value(value.extruder_printable_height).unwrap(), serde_json::json!(["nil", "256"]));
    assert!(serde_json::from_str::<PrinterRemainingOptions>(r#"{"nozzle_volume":[null]}"#).is_err());
}

#[test]
fn printer_options_preserves_points_groups_and_explicit_empty_areas() {
    let value: PrinterRemainingOptions = serde_json::from_value(Value::Object(fixture_fields(REMAINING_KEYS))).unwrap();
    assert!(value.bed_exclude_area.0.is_empty());
    assert!(value.head_wrap_detect_zone.0.is_empty());
    assert!(value.parallel_printheads_bed_exclude_areas.0.is_empty());
    assert_eq!(value.extruder_printable_area.0.len(), 2);
    assert_eq!(value.extruder_printable_area.0[1][0].x, 20.5);
    assert_eq!(value.printable_area.0.len(), 4);
}

#[test]
fn printer_options_rejects_duplicate_unknown_cross_scope_and_wrong_shapes() {
    for invalid in [
        r#"{"printable_height":"100","printable_height":"200"}"#,
        r#"{"unknown_printer_field":"1"}"#,
        r#"{"extruder_ams_count":["1#0|4#0"]}"#,
        r#"{"extruder_offset":["0x0"]}"#,
        r#"{"layer_height":"0.2"}"#,
        r#"{"filament_type":["PLA"]}"#,
    ] {
        assert!(serde_json::from_str::<PrinterOptions>(invalid).is_err(), "{invalid}");
    }
    for invalid in [
        r#"{"machine_max_speed_x":["1"]}"#,
        r#"{"gcode_flavor":"klipper"}"#,
        r#"{"extruder_printable_area":"0x0,1x1"}"#,
        r#"{"bed_mesh_min":["0x0"]}"#,
        r#"{"host_type":"klipper"}"#,
    ] {
        assert!(serde_json::from_str::<PrinterRemainingOptions>(invalid).is_err(), "{invalid}");
    }
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!("../../../../../tests/ksr_fdmtest_v4/options-v242.json")).unwrap()
}

fn remaining_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter().filter(|row| row.raw_scope == "printer" && matches!(row.static_owner.as_str(), "print_config" | "unowned")).collect()
}

fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let project = crate::load_project(include_bytes!("../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf")).unwrap();
    let fixture: Value = serde_json::from_slice(project.project_settings_bytes()).unwrap();
    let fixture = fixture.as_object().unwrap();
    keys.into_iter().map(|key| (key.to_owned(), fixture[key].clone())).collect()
}

fn expected_default(row: &InventoryRow) -> Value {
    match row.option_type.as_str() {
        "coBool" | "coEnum" | "coFloat" | "coInt" | "coPoint" | "coString" => Value::String(row.default_serialized.clone()),
        "coEnums" | "coFloats" => strings(row.default_serialized.split(',')),
        "coPoints" => strings(row.default_serialized.split(',').filter(|value| !value.is_empty())),
        "coPointsGroups" => Value::Array(Vec::new()),
        "coStrings" if row.key == "extruder_variant_list" => Value::Array(vec![Value::String(serde_json::from_str::<String>(&row.default_serialized).unwrap())]),
        "coStrings" => Value::Array(Vec::new()),
        kind => panic!("unexpected remaining option type {kind}"),
    }
}

fn strings<'a>(values: impl IntoIterator<Item = &'a str>) -> Value {
    Value::Array(values.into_iter().map(|value| Value::String(value.to_owned())).collect())
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
