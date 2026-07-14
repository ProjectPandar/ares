use std::collections::BTreeSet;

use crate::options::{
    ExtruderType, ExtruderTypes, ExtruderVariantLists, NozzleVolumeType, NozzleVolumeTypes,
    OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaStrings, ProjectSettings, VariantStride,
    registry::{
        filament_options_with_variant, print_options_with_variant,
        printer_options_with_variant_1, printer_options_with_variant_2,
    },
};
use crate::SliceError;
use serde_json::{Map, Value, json};

pub(super) fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

pub(super) fn active_source() -> ProjectSettings {
    let mut source = ProjectSettings::default();
    source.project.print.nozzle_diameter =
        OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.6)]);
    source.printer.remaining.extruder_variant_list = ExtruderVariantLists(vec![
        "Direct Drive Standard".to_owned(),
        "Bowden Standard".to_owned(),
    ]);
    source.printer.gcode.extruder_type =
        ExtruderTypes(vec![ExtruderType::DirectDrive, ExtruderType::Bowden]);
    source.project.gcode.nozzle_volume_type = NozzleVolumeTypes(vec![
        NozzleVolumeType::Standard,
        NozzleVolumeType::Standard,
    ]);
    source.printer.gcode.printer_extruder_id = ints(&[1, 2]);
    source.printer.gcode.printer_extruder_variant = OrcaStrings(vec![
        "Direct Drive Standard".to_owned(),
        "Bowden Standard".to_owned(),
    ]);
    source.process.region.print_extruder_id = ints(&[1, 2]);
    source.process.region.print_extruder_variant = OrcaStrings(vec![
        "Direct Drive Standard".to_owned(),
        "Bowden Standard".to_owned(),
    ]);
    source.project.preset.filament_self_index = ints(&[1, 2]);
    source.filament.gcode.filament_extruder_variant = VariantStride(vec![
        "Direct Drive Standard".to_owned(),
        "Bowden Standard".to_owned(),
    ]);
    source.project.gcode.filament_map = ints(&[1, 2]);
    let source = source_with_overrides(
        &source,
        json!({
            "deretraction_speed": [0, 0, 0],
            "long_retractions_when_cut": [false, false, false],
            "nozzle_flush_dataset": [0, 0, 0],
            "nozzle_type": ["undefine", "undefine", "undefine"],
            "nozzle_volume": [0, 0, 0],
            "retract_before_wipe": ["100%", "100%", "100%"],
            "retract_length_toolchange": [10, 10, 10],
            "retract_lift_above": [0, 0, 0],
            "retract_lift_below": [0, 0, 0],
            "retract_lift_enforce": ["All Surfaces", "All Surfaces", "All Surfaces"],
            "retract_restart_extra": [0, 0, 0],
            "retract_restart_extra_toolchange": [0, 0, 0],
            "retract_when_changing_layer": [false, false, false],
            "retraction_distances_when_cut": [18, 18, 18],
            "retraction_length": [0.8, 0.8, 0.8],
            "retraction_minimum_travel": [2, 2, 2],
            "retraction_speed": [30, 30, 30],
            "travel_slope": [3, 3, 3],
            "wipe": [false, false, false],
            "wipe_distance": [1, 1, 1],
            "z_hop": [0.4, 0.4, 0.4],
            "z_hop_types": ["Slope Lift", "Slope Lift", "Slope Lift"]
        }),
    );
    let source = source_with_overrides(
        &source,
        json!({
            "machine_max_acceleration_e": [5000, 5000, 5000, 5000],
            "machine_max_acceleration_extruding": [1500, 1250, 1500, 1250],
            "machine_max_acceleration_retracting": [1500, 1250, 1500, 1250],
            "machine_max_acceleration_travel": [0, 0, 0, 0],
            "machine_max_acceleration_x": [1000, 1000, 1000, 1000],
            "machine_max_acceleration_y": [1000, 1000, 1000, 1000],
            "machine_max_acceleration_z": [500, 200, 500, 200],
            "machine_max_jerk_e": [2.5, 2.5, 2.5, 2.5],
            "machine_max_jerk_x": [10, 10, 10, 10],
            "machine_max_jerk_y": [10, 10, 10, 10],
            "machine_max_jerk_z": [0.2, 0.4, 0.2, 0.4],
            "machine_max_speed_e": [120, 120, 120, 120],
            "machine_max_speed_x": [500, 200, 500, 200],
            "machine_max_speed_y": [500, 200, 500, 200],
            "machine_max_speed_z": [12, 12, 12, 12]
        }),
    );
    with_valid_filament_pair_payloads(&source)
}

pub(super) fn one_extruder_source() -> ProjectSettings {
    let mut source = active_source();
    source.project.print.nozzle_diameter.0.truncate(1);
    source.printer.remaining.extruder_variant_list.0 =
        vec!["Direct Drive Standard".to_owned()];
    source.printer.gcode.extruder_type.0.truncate(1);
    source.project.gcode.nozzle_volume_type.0.truncate(1);
    source.printer.gcode.printer_extruder_id = ints(&[1]);
    source.printer.gcode.printer_extruder_variant.0.truncate(1);
    source.process.region.print_extruder_id = ints(&[1]);
    source.process.region.print_extruder_variant.0.truncate(1);
    source.project.preset.filament_self_index = ints(&[1]);
    source.filament.gcode.filament_extruder_variant.0.truncate(1);
    source.project.gcode.filament_map = ints(&[1]);
    source
}

pub(super) fn filament_sentinel_source() -> ProjectSettings {
    let mut source = active_source();
    source.project.preset.filament_self_index = ints(&[1, 9, 2, 9, 1, 9, 2, 9]);
    let mut overrides = Map::new();
    insert_overrides(
        &mut overrides,
        &[
            "activate_air_filtration",
            "activate_air_filtration_during_print",
            "activate_air_filtration_on_completion",
        ],
        json!([true, true, false, true, false, true, true, false]),
    );
    insert_overrides(
        &mut overrides,
        &[
            "complete_print_exhaust_fan_speed",
            "during_print_exhaust_fan_speed",
            "nozzle_temperature",
            "nozzle_temperature_initial_layer",
        ],
        json!([101, 102, 103, 104, 105, 106, 107, 108]),
    );
    insert_overrides(
        &mut overrides,
        &[
            "filament_adaptive_volumetric_speed",
            "filament_long_retractions_when_cut",
            "filament_retract_when_changing_layer",
            "filament_wipe",
            "long_retractions_when_ec",
        ],
        json!([true, false, false, true, "nil", false, "nil", true]),
    );
    insert_overrides(
        &mut overrides,
        &[
            "filament_cooling_before_tower",
            "filament_deretraction_speed",
            "filament_flush_volumetric_speed",
            "filament_ironing_inset",
            "filament_ironing_spacing",
            "filament_ironing_speed",
            "filament_retract_lift_above",
            "filament_retract_lift_below",
            "filament_retract_restart_extra",
            "filament_retraction_distances_when_cut",
            "filament_retraction_length",
            "filament_retraction_minimum_travel",
            "filament_retraction_speed",
            "filament_wipe_distance",
            "filament_z_hop",
            "retraction_distances_when_ec",
        ],
        json!([1.01, 1.02, 1.03, 1.04, "nil", 1.06, 1.07, 1.08]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_flow_ratio"],
        json!([1.11, 1.12, 1.13, 1.14, "nil", 1.16, 1.17, 1.18]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_flush_temp"],
        json!([201, 202, 203, 204, "nil", 206, 207, 208]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_max_volumetric_speed"],
        json!([2.01, 2.02, 2.03, 2.04, 2.05, 2.06, 2.07, 2.08]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_ironing_flow", "filament_retract_before_wipe"],
        json!(["11%", "12%", "13%", "14%", "nil", "16%", "17%", "18%"]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_extruder_variant"],
        json!([
            "Direct Drive Standard",
            "unused-1",
            "Direct Drive Standard",
            "unused-3",
            "Bowden Standard",
            "unused-5",
            "Bowden Standard",
            "unused-7"
        ]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_retract_lift_enforce"],
        json!([
            "All Surfaces",
            "Top Only",
            "All Surfaces",
            "Top Only",
            "Bottom Only",
            "All Surfaces",
            "Top Only",
            "Bottom Only"
        ]),
    );
    insert_overrides(
        &mut overrides,
        &["filament_z_hop_types"],
        json!([
            "Auto Lift",
            "Normal Lift",
            "Slope Lift",
            "Auto Lift",
            "Spiral Lift",
            "Normal Lift",
            "Normal Lift",
            "Slope Lift"
        ]),
    );
    insert_overrides(
        &mut overrides,
        &["volumetric_speed_coefficients"],
        json!(["1 2 3", "2 3 4", "3 4 5", "4 5 6", "5 6 7", "6 7 8", "7 8 9", "8 9 10"]),
    );
    source_with_overrides(&source, Value::Object(overrides))
}

pub(super) fn assert_invalid_key<T: std::fmt::Debug>(
    result: Result<T, SliceError>,
    key: &str,
) {
    match result.unwrap_err() {
        SliceError::InvalidInput(message) => assert!(
            message.contains(key),
            "expected error to name {key}, got {message}"
        ),
        error => panic!("expected InvalidInput naming {key}, got {error:?}"),
    }
}

pub(super) fn flat_settings(source: &ProjectSettings) -> Map<String, Value> {
    let mut flat = Map::new();
    for owner in [
        serde_json::to_value(&source.printer).unwrap(),
        serde_json::to_value(&source.process).unwrap(),
        serde_json::to_value(&source.filament).unwrap(),
        serde_json::to_value(&source.project).unwrap(),
        serde_json::to_value(&source.metadata).unwrap(),
    ] {
        for (key, value) in owner.as_object().unwrap() {
            assert!(flat.insert(key.clone(), value.clone()).is_none());
        }
    }
    flat
}

pub(super) fn source_with_overrides(
    source: &ProjectSettings,
    overrides: Value,
) -> ProjectSettings {
    let mut flat = flat_settings(source);
    for (key, value) in overrides.as_object().unwrap() {
        assert!(flat.contains_key(key), "unknown test override {key}");
        flat.insert(key.clone(), value.clone());
    }
    serde_json::from_value(Value::Object(flat)).unwrap()
}

pub(super) fn assert_selected_indices(
    source: &ProjectSettings,
    materialized: &ProjectSettings,
    keys: &[&str],
    indices: &[usize],
) {
    let source = flat_settings(source);
    let materialized = flat_settings(materialized);
    for key in keys {
        let values = source[*key].as_array().unwrap();
        let expected = Value::Array(indices.iter().map(|&index| values[index].clone()).collect());
        assert_eq!(materialized[*key], expected, "wrong selection for {key}");
        assert_eq!(
            materialized[*key].as_array().unwrap().len(),
            indices.len(),
            "wrong cardinality for {key}"
        );
    }
}

pub(super) fn assert_outside_variant_families_unchanged(
    source: &ProjectSettings,
    materialized: &ProjectSettings,
) {
    let allowed = variant_family_keys()
        .chain(["filament_map"])
        .collect::<BTreeSet<_>>();
    let source = flat_settings(source);
    let materialized = flat_settings(materialized);
    for (key, value) in &source {
        if !allowed.contains(key.as_str()) {
            assert_eq!(materialized[key], *value, "unexpected write to {key}");
        }
    }
}

pub(super) fn assert_changed_keys(
    left: &ProjectSettings,
    right: &ProjectSettings,
    expected: impl IntoIterator<Item = &'static str>,
) {
    let left = flat_settings(left);
    let right = flat_settings(right);
    let actual = left
        .iter()
        .filter_map(|(key, value)| (right[key] != *value).then_some(key.as_str()))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected.into_iter().collect());
}

pub(super) fn assert_family_cardinalities(materialized: &ProjectSettings) {
    let flat = flat_settings(materialized);
    for (keys, cardinality) in [
        (print_options_with_variant(), 2),
        (filament_options_with_variant(), 2),
        (printer_options_with_variant_1(), 2),
        (printer_options_with_variant_2(), 4),
    ] {
        for key in keys {
            assert_eq!(flat[*key].as_array().unwrap().len(), cardinality, "{key}");
        }
    }
}

fn insert_overrides(overrides: &mut Map<String, Value>, keys: &[&str], value: Value) {
    for key in keys {
        assert!(overrides.insert((*key).to_owned(), value.clone()).is_none());
    }
}

fn variant_family_keys() -> impl Iterator<Item = &'static str> {
    print_options_with_variant()
        .iter()
        .chain(filament_options_with_variant())
        .chain(printer_options_with_variant_1())
        .chain(printer_options_with_variant_2())
        .copied()
}

fn with_valid_filament_pair_payloads(source: &ProjectSettings) -> ProjectSettings {
    let mut flat = flat_settings(source);
    for key in filament_options_with_variant() {
        if *key == "filament_extruder_variant" {
            continue;
        }
        let first = flat[*key].as_array().unwrap()[0].clone();
        flat.insert((*key).to_owned(), Value::Array(vec![first.clone(), first]));
    }
    serde_json::from_value(Value::Object(flat)).unwrap()
}
