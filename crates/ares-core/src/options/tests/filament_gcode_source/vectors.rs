use serde_json::{Value, json};

use super::super::super::{
    FilamentGCodeSourceOptions, FilamentOptions, Nullable, OrcaBool, OrcaFloat, OrcaInt,
};
use super::expected::{LEXICAL_KEYS, NULLABLE_KEYS};

#[test]
fn arbitrary_valid_lengths_round_trip_without_cardinality_rules() {
    for (key, values) in [
        ("adaptive_pressure_advance", json!([])),
        ("filament_cost", json!(["1"])),
        ("filament_printable", json!(["1", "2", "3"])),
        ("filament_type", json!(["a", "b", "c", "d", "e"])),
        (
            "filament_extruder_variant",
            json!(["a", "b", "c", "d", "e", "f", "g", "h"]),
        ),
    ] {
        assert_round_trip::<FilamentGCodeSourceOptions>(key, values.clone());
        assert_round_trip::<FilamentOptions>(key, values);
    }
}

#[test]
fn nullable_elements_use_direct_typed_vectors_and_exact_nil_bytes() {
    let parsed: FilamentGCodeSourceOptions = serde_json::from_value(json!({
        "filament_adaptive_volumetric_speed": ["nil", "1"],
        "filament_cooling_before_tower": ["nil", "2.5"],
        "filament_flow_ratio": ["nil", "1.1"],
        "filament_flush_temp": ["nil", "230"],
        "filament_flush_volumetric_speed": ["nil", "3.5"],
        "long_retractions_when_ec": ["nil", "0"],
        "retraction_distances_when_ec": ["nil", "18"]
    }))
    .unwrap();
    assert_eq!(
        parsed.filament_adaptive_volumetric_speed,
        [Nullable::Nil, Nullable::Value(OrcaBool(true))]
    );
    assert_eq!(
        parsed.filament_cooling_before_tower,
        [Nullable::Nil, Nullable::Value(OrcaFloat(2.5))]
    );
    assert_eq!(
        parsed.filament_flush_temp,
        [Nullable::Nil, Nullable::Value(OrcaInt(230))]
    );
    let serialized = serde_json::to_value(parsed).unwrap();
    for key in NULLABLE_KEYS {
        assert_eq!(serialized[key][0], "nil", "{key}");
    }
}

#[test]
fn nonnullable_numeric_and_boolean_arrays_reject_nil_but_strings_keep_it() {
    for key in [
        "adaptive_pressure_advance",
        "adaptive_pressure_advance_bridges",
        "filament_adhesiveness_category",
    ] {
        let input = json!({key: ["nil"]});
        let error = serde_json::from_value::<FilamentOptions>(input)
            .unwrap_err()
            .to_string();
        assert!(error.contains(key), "{key}: {error}");
    }
    for key in [
        "adaptive_pressure_advance_model",
        "default_filament_colour",
        "filament_extruder_variant",
        "filament_ramming_parameters",
        "volumetric_speed_coefficients",
    ] {
        assert_round_trip::<FilamentOptions>(key, json!(["nil"]));
    }
}

#[test]
fn raw_structured_multiline_and_empty_strings_round_trip_exactly() {
    let input = json!({
        "adaptive_pressure_advance_model": ["0.04,3.96,3000\n0.033,3.96,10000"],
        "filament_change_extrusion_role_gcode": ["", "M117 role\nG4 P0"],
        "filament_end_gcode": ["\nM117 done"],
        "filament_extruder_variant": ["Direct Drive Standard", "Bowden High Flow"],
        "filament_ramming_parameters": ["120 100| 0.05 6.6"],
        "filament_start_gcode": [" ", "M117 start\n"],
        "volumetric_speed_coefficients": ["1 2 3", ""]
    });
    let parsed: FilamentOptions = serde_json::from_value(input.clone()).unwrap();
    let output = serde_json::to_value(parsed).unwrap();
    for key in LEXICAL_KEYS {
        if input.get(key).is_some() {
            assert_eq!(output[key], input[key], "{key}");
        }
    }
}

#[test]
fn deferred_tokens_and_collision_boundary_are_recorded_without_normalization() {
    for token in [
        "Direct Drive Standard",
        "Direct Drive High Flow",
        "Bowden Standard",
        "Bowden High Flow",
        "Normal",
        "Big Traffic",
    ] {
        assert_round_trip::<FilamentOptions>("filament_extruder_variant", json!([token]));
    }
    assert_round_trip::<FilamentOptions>("filament_type", json!(["ASA-Aero"]));
    assert_eq!(LEXICAL_KEYS.len(), 53);
    assert_eq!(
        LEXICAL_KEYS
            .iter()
            .filter(|key| {
                matches!(
                    **key,
                    "adaptive_pressure_advance_model" | "adaptive_pressure_advance_overhangs"
                )
            })
            .count(),
        2
    );
    assert_eq!(LEXICAL_KEYS.len() - 2, 51);
}

fn assert_round_trip<T>(key: &str, values: Value)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let input = json!({key: values});
    let parsed: T = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()[key], input[key]);
}
