use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, de::IgnoredAny};
use serde_json::{Map, Value};

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, Percent, PrinterOptions,
    ProcessObjectSourceOptions, ProcessOptions, ProjectSettings,
};

mod direct_dispatch;
mod enums;
pub(super) mod expected;
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
fn process_object_source_inventory_is_exact_typed_scalar_and_active() {
    let rows = inventory();
    let object = object_rows(&rows);
    assert_eq!(object.len(), 126);
    assert!(object.iter().all(|row| !row.nullable && row.wire_shape == "scalar_string"));
    assert_eq!(
        object.iter().fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        }),
        BTreeMap::from([
            ("coBool", 22),
            ("coEnum", 12),
            ("coFloat", 63),
            ("coFloatOrPercent", 6),
            ("coInt", 13),
            ("coPercent", 10),
        ])
    );
    let keys = object.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    assert!(!keys.contains("independent_support_layer_height"));
    assert!(!keys.contains("adaptive_layer_height"));
    for deferred in [
        "initial_layer_print_height",
        "resolution",
        "wall_loops",
        "sparse_infill_density",
        "sparse_infill_pattern",
        "top_shell_layers",
        "bottom_shell_layers",
    ] {
        assert!(!keys.contains(deferred), "{deferred}");
    }
}

#[test]
fn process_object_source_fixture_round_trips_all_126_fields_byte_for_byte() {
    let rows = inventory();
    let keys = object_rows(&rows).into_iter().map(|row| row.key.as_str()).collect::<Vec<_>>();
    let fixture = fixture_fields(keys.iter().copied());
    let parsed: ProcessObjectSourceOptions =
        serde_json::from_value(Value::Object(fixture.clone())).unwrap();
    type_assertions::assert_concrete_types(&parsed);
    let serialized = serde_json::to_string(&parsed).unwrap();
    assert_eq!(serialized.as_bytes(), serde_json::to_vec(&fixture).unwrap());
    assert_eq!(serialized_key_order(&serialized), keys);
}

#[test]
fn process_object_source_defaults_match_fixed_tag_and_fixture_has_18_overrides() {
    let rows = inventory();
    let object = object_rows(&rows);
    let defaults = serde_json::to_value(ProcessObjectSourceOptions::default()).unwrap();
    let fixture = fixture_fields(object.iter().map(|row| row.key.as_str()));
    for row in &object {
        assert_eq!(defaults[&row.key], row.default_serialized, "{}", row.key);
    }
    let changed = object
        .iter()
        .filter(|row| defaults[&row.key] != fixture[&row.key])
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(changed, BTreeSet::from(FIXTURE_OVERRIDE_KEYS));
}

#[test]
fn process_object_source_keeps_declaration_and_wire_orders_separate() {
    let rows = inventory();
    let keys = object_rows(&rows).into_iter().map(|row| row.key.as_str()).collect::<Vec<_>>();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    assert_ne!(&DECLARATION_ORDER[..], keys.as_slice());
    assert_eq!(DECLARATION_ORDER.len(), 126);
    assert_eq!(
        ProcessObjectSourceOptions::DECLARATION_ORDER,
        DECLARATION_ORDER
    );
    assert_eq!(DECLARATION_ORDER.iter().copied().collect::<BTreeSet<_>>(), keys.iter().copied().collect());
}

#[test]
fn process_object_source_scalar_codecs_preserve_categories() {
    let defaults = ProcessObjectSourceOptions::default();
    let _: &OrcaBool = &defaults.enable_support;
    let _: &OrcaFloat = &defaults.layer_height;
    let _: &OrcaInt = &defaults.raft_layers;
    let _: &Percent = &defaults.wall_transition_length;
    let _: &FloatOrPercent = &defaults.line_width;
    assert_eq!(serde_json::to_value(defaults.line_width).unwrap(), "0");
    assert_eq!(serde_json::to_value(defaults.bridge_acceleration).unwrap(), "50%");
    assert_eq!(serde_json::to_value(defaults.support_threshold_overlap).unwrap(), "50%");
    assert_eq!(serde_json::to_value(defaults.wall_transition_length).unwrap(), "100%");
    assert_eq!(serde_json::to_value(defaults.skirt_start_angle).unwrap(), "-135");
}

#[test]
fn project_settings_starts_with_typed_process_object_child() {
    assert_eq!(
        ProjectSettings::default().process.object.layer_height,
        OrcaFloat(0.2)
    );
}

#[test]
fn option_group_decode_errors_name_flat_key_and_preserve_source_error() {
    for (json, key, source) in [
        (r#"{"machine_max_speed_x":["bad"]}"#, "machine_max_speed_x", "invalid float literal"),
        (r#"{"gcode_flavor":"bad"}"#, "gcode_flavor", "unknown variant"),
        (r#"{"host_type":"bad"}"#, "host_type", "unknown variant"),
    ] {
        let error = serde_json::from_str::<PrinterOptions>(json).unwrap_err().to_string();
        assert!(error.contains(key), "{error}");
        assert!(error.contains(source), "{error}");
    }
    let error = serde_json::from_str::<ProcessObjectSourceOptions>(
        r#"{"brim_type":"invalid"}"#,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("brim_type"), "{error}");
    assert!(error.contains("unknown variant"), "{error}");
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn object_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "process" && row.static_owner == "print_object_config")
        .collect()
}

pub(super) fn fixture_fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Map<String, Value> {
    let project = crate::load_project(include_bytes!(
        "../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf"
    ))
    .unwrap();
    let fixture: Value = serde_json::from_slice(project.project_settings_bytes()).unwrap();
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
