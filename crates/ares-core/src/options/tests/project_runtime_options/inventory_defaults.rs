use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::super::super::{
    PresetMetadata, ProjectGCodeSourceOptions, ProjectPrintSourceOptions, ProjectPresetSourceOptions,
    ProjectRuntimeOptions, ProjectSettings,
};
use super::expected::{
    Child, GCODE_DECLARATION_ORDER, METADATA_KEYS, PRESET_DECLARATION_ORDER,
    PRINT_DECLARATION_ORDER, REAL_FIELDS, RESIDUAL_LEXICAL_KEYS,
};
use super::{
    assert_pairwise_disjoint, expected_default, inventory, real_rows, residual_rows,
};

#[test]
fn project_settings_requires_the_genuine_task14_aggregate_interfaces() {
    let settings = ProjectSettings::default();
    let _: ProjectRuntimeOptions = settings.project;
    let _: PresetMetadata = settings.metadata;
}

#[test]
fn residual_inventory_partition_shapes_and_histograms_are_exact() {
    let rows = inventory();
    let residual = residual_rows(&rows);
    assert_eq!(residual.len(), 47);
    assert_eq!(
        residual.iter().map(|row| row.key.as_str()).collect::<Vec<_>>(),
        RESIDUAL_LEXICAL_KEYS
    );
    assert_eq!(
        residual.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>().len(),
        47
    );
    assert!(residual.iter().all(|row| !row.nullable));
    assert_eq!(residual.iter().filter(|row| row.wire_shape == "array").count(), 37);
    assert_eq!(
        residual
            .iter()
            .filter(|row| row.wire_shape == "scalar_string")
            .count(),
        10
    );

    let histogram = residual.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
        counts
    });
    assert_eq!(
        histogram,
        BTreeMap::from([
            ("Metadata", 3),
            ("coBool", 2),
            ("coBools", 2),
            ("coEnum", 2),
            ("coEnums", 1),
            ("coFloats", 19),
            ("coInt", 1),
            ("coInts", 4),
            ("coPercents", 1),
            ("coPoints", 2),
            ("coString", 2),
            ("coStrings", 8),
        ])
    );

    for field in &REAL_FIELDS {
        let row = residual.iter().find(|row| row.key == field.key).unwrap();
        assert_eq!(row.option_type, field.kind, "{}", field.key);
        assert_eq!(row.wire_shape == "array", field.is_array, "{}", field.key);
        let expected_owner = match field.child {
            Child::GCode => "g_code_config",
            Child::Print => "print_config",
            Child::Preset => "unowned",
        };
        assert_eq!(row.static_owner, expected_owner, "{}", field.key);
    }
    assert_eq!(
        residual
            .iter()
            .filter(|row| row.option_type == "Metadata")
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        METADATA_KEYS
    );
}

#[test]
fn four_raw_scopes_are_disjoint_complete_653_and_keep_the_650_real_histogram() {
    let rows = inventory();
    assert_eq!(rows.len(), 653);
    let scope_sets = ["printer", "process", "filament", "residual"].map(|scope| {
        rows.iter()
            .filter(|row| row.raw_scope == scope)
            .map(|row| row.key.as_str())
            .collect::<BTreeSet<_>>()
    });
    assert_pairwise_disjoint(&scope_sets);
    assert_eq!(
        scope_sets.iter().map(BTreeSet::len).collect::<Vec<_>>(),
        [132, 352, 122, 47]
    );
    assert_eq!(scope_sets.iter().map(BTreeSet::len).sum::<usize>(), 653);

    let histogram = rows
        .iter()
        .filter(|row| row.option_type != "Metadata")
        .fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        });
    assert_eq!(
        histogram,
        BTreeMap::from([
            ("coBool", 105),
            ("coBools", 22),
            ("coEnum", 44),
            ("coEnums", 9),
            ("coFloat", 160),
            ("coFloatOrPercent", 36),
            ("coFloats", 90),
            ("coInt", 41),
            ("coInts", 45),
            ("coPercent", 25),
            ("coPercents", 5),
            ("coPoint", 4),
            ("coPoints", 6),
            ("coPointsGroups", 1),
            ("coString", 30),
            ("coStrings", 27),
        ])
    );
}

#[test]
fn declaration_orders_and_all_fixed_defaults_are_independent_and_exact() {
    assert_eq!(ProjectGCodeSourceOptions::DECLARATION_ORDER, GCODE_DECLARATION_ORDER);
    assert_eq!(ProjectPrintSourceOptions::DECLARATION_ORDER, PRINT_DECLARATION_ORDER);
    assert_eq!(ProjectPresetSourceOptions::DECLARATION_ORDER, PRESET_DECLARATION_ORDER);

    let runtime = serde_json::to_value(ProjectRuntimeOptions::default()).unwrap();
    assert_eq!(runtime.as_object().unwrap().len(), 44);
    for field in &REAL_FIELDS {
        assert_eq!(runtime[field.key], expected_default(field), "{}", field.key);
    }
    assert_eq!(runtime["extruder_ams_count"], Value::Array(Vec::new()));
    assert_ne!(runtime["extruder_ams_count"], serde_json::json!([""]));

    let metadata = PresetMetadata::default();
    assert_eq!(metadata.from, "");
    assert_eq!(metadata.name, "");
    assert_eq!(metadata.version, "");
    assert_eq!(real_rows(&inventory()).len(), 44);
}
