use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::expected::{
    Child, DEFAULT_EQUAL_KEYS, METADATA_KEYS, REAL_FIELDS, RESIDUAL_LEXICAL_KEYS,
    SINGLETON_ARRAY_KEYS,
};
use super::super::super::{
    FilamentOptions, PresetMetadata, PrinterOptions, ProcessOptions, ProjectGCodeSourceOptions,
    ProjectPrintSourceOptions, ProjectPresetSourceOptions, ProjectRuntimeOptions,
};
use super::{expected_defaults, fixture, fixture_fields, inventory};

#[test]
fn real_3mf_round_trips_each_child_flat_runtime_and_metadata_exactly() {
    for child in [Child::GCode, Child::Print, Child::Preset] {
        let keys = REAL_FIELDS
            .iter()
            .filter(|field| field.child == child)
            .map(|field| field.key);
        let fields = fixture_fields(keys);
        let output = match child {
            Child::GCode => serde_json::to_vec(
                &serde_json::from_value::<ProjectGCodeSourceOptions>(Value::Object(fields.clone()))
                    .unwrap(),
            )
            .unwrap(),
            Child::Print => serde_json::to_vec(
                &serde_json::from_value::<ProjectPrintSourceOptions>(Value::Object(fields.clone()))
                    .unwrap(),
            )
            .unwrap(),
            Child::Preset => serde_json::to_vec(
                &serde_json::from_value::<ProjectPresetSourceOptions>(Value::Object(fields.clone()))
                    .unwrap(),
            )
            .unwrap(),
        };
        assert_eq!(output, serde_json::to_vec(&fields).unwrap());
    }

    let runtime_fields = fixture_fields(REAL_FIELDS.iter().map(|field| field.key));
    let runtime: ProjectRuntimeOptions =
        serde_json::from_value(Value::Object(runtime_fields.clone())).unwrap();
    assert_eq!(
        serde_json::to_vec(&runtime).unwrap(),
        serde_json::to_vec(&runtime_fields).unwrap()
    );

    let metadata_fields = fixture_fields(METADATA_KEYS);
    let metadata: PresetMetadata =
        serde_json::from_value(Value::Object(metadata_fields.clone())).unwrap();
    assert_eq!(
        serde_json::to_vec(&metadata).unwrap(),
        serde_json::to_vec(&metadata_fields).unwrap()
    );
}

#[test]
fn real_3mf_has_exact_37_arrays_ten_scalars_singletons_and_cardinalities() {
    let residual = fixture_fields(RESIDUAL_LEXICAL_KEYS);
    assert_eq!(residual.values().filter(|value| value.is_array()).count(), 37);
    assert_eq!(residual.values().filter(|value| value.is_string()).count(), 10);

    let arrays = REAL_FIELDS
        .iter()
        .filter(|field| field.is_array)
        .map(|field| (field.key, residual[field.key].as_array().unwrap().len()))
        .collect::<Vec<_>>();
    let histogram = arrays.iter().fold(BTreeMap::new(), |mut counts, (_, length)| {
        *counts.entry(*length).or_insert(0) += 1;
        counts
    });
    assert_eq!(histogram, BTreeMap::from([(1, 6), (2, 14), (4, 15), (8, 2)]));
    assert_eq!(
        arrays
            .iter()
            .filter(|(_, length)| *length == 1)
            .map(|(key, _)| *key)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(SINGLETON_ARRAY_KEYS)
    );
    for key in SINGLETON_ARRAY_KEYS {
        assert!(residual[key].is_array(), "{key}");
        assert!(!residual[key].is_string(), "{key}");
    }
}

#[test]
fn exactly_seven_real_fixture_values_equal_defaults_and_37_differ() {
    let fixture = fixture_fields(REAL_FIELDS.iter().map(|field| field.key));
    let defaults = expected_defaults(None);
    let equal = REAL_FIELDS
        .iter()
        .filter(|field| fixture[field.key] == defaults[field.key])
        .map(|field| field.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(equal, BTreeSet::from(DEFAULT_EQUAL_KEYS));
    assert_eq!(equal.len(), 7);
    assert_eq!(REAL_FIELDS.len() - equal.len(), 37);
}

#[test]
fn typed_partition_serialization_merges_to_all_653_fixture_fields_without_shape_loss() {
    let rows = inventory();
    let fixture = fixture();
    let mut merged = Map::new();

    let printer_fields = fields_for_scope(&rows, &fixture, "printer");
    extend_serialized(
        &mut merged,
        serde_json::to_value(
            serde_json::from_value::<PrinterOptions>(Value::Object(printer_fields)).unwrap(),
        )
        .unwrap(),
    );
    let process_fields = fields_for_scope(&rows, &fixture, "process");
    extend_serialized(
        &mut merged,
        serde_json::to_value(
            serde_json::from_value::<ProcessOptions>(Value::Object(process_fields)).unwrap(),
        )
        .unwrap(),
    );
    let filament_fields = fields_for_scope(&rows, &fixture, "filament");
    extend_serialized(
        &mut merged,
        serde_json::to_value(
            serde_json::from_value::<FilamentOptions>(Value::Object(filament_fields)).unwrap(),
        )
        .unwrap(),
    );
    let runtime_fields = fixture_fields(REAL_FIELDS.iter().map(|field| field.key));
    extend_serialized(
        &mut merged,
        serde_json::to_value(
            serde_json::from_value::<ProjectRuntimeOptions>(Value::Object(runtime_fields)).unwrap(),
        )
        .unwrap(),
    );
    let metadata_fields = fixture_fields(METADATA_KEYS);
    extend_serialized(
        &mut merged,
        serde_json::to_value(
            serde_json::from_value::<PresetMetadata>(Value::Object(metadata_fields)).unwrap(),
        )
        .unwrap(),
    );

    assert_eq!(merged.len(), 653);
    assert_eq!(Value::Object(merged), Value::Object(fixture));
    assert_eq!(json!(132 + 352 + 122 + 47), json!(653));
}

fn fields_for_scope(
    rows: &[super::InventoryRow],
    fixture: &Map<String, Value>,
    scope: &str,
) -> Map<String, Value> {
    rows.iter()
        .filter(|row| row.raw_scope == scope)
        .map(|row| (row.key.clone(), fixture[&row.key].clone()))
        .collect()
}

fn extend_serialized(target: &mut Map<String, Value>, value: Value) {
    for (key, value) in value.as_object().unwrap() {
        assert!(target.insert(key.clone(), value.clone()).is_none(), "duplicate {key}");
    }
}
