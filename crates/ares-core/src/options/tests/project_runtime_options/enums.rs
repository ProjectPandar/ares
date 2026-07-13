use serde_json::{Value, json};

use super::super::super::{
    ProjectGCodeSourceOptions, ProjectPrintSourceOptions, ProjectRuntimeOptions,
};
use super::{assert_keyed_bounded_error, parent_error};

#[test]
fn strict_raw_enums_accept_every_canonical_fixed_map_token() {
    for token in [
        "Default Plate",
        "Supertack Plate",
        "Cool Plate",
        "Engineering Plate",
        "High Temp Plate",
        "Textured PEI Plate",
        "Textured Cool Plate",
    ] {
        assert_scalar_round_trip::<ProjectPrintSourceOptions>("curr_bed_type", token);
        assert_scalar_round_trip::<ProjectRuntimeOptions>("curr_bed_type", token);
    }
    for token in ["Auto For Flush", "Auto For Match", "Manual"] {
        assert_scalar_round_trip::<ProjectGCodeSourceOptions>("filament_map_mode", token);
        assert_scalar_round_trip::<ProjectRuntimeOptions>("filament_map_mode", token);
    }
    for token in ["Standard", "High Flow"] {
        assert_vector_round_trip::<ProjectGCodeSourceOptions>("nozzle_volume_type", token);
        assert_vector_round_trip::<ProjectRuntimeOptions>("nozzle_volume_type", token);
    }
}

#[test]
fn strict_raw_enums_reject_unknown_case_numeric_ui_and_legacy_spellings_with_key() {
    for (key, invalid, vector) in [
        ("curr_bed_type", json!("cool plate"), false),
        ("curr_bed_type", json!("SuperTack Plate"), false),
        ("curr_bed_type", json!(7), false),
        ("filament_map_mode", json!("Default"), false),
        ("filament_map_mode", json!("Auto"), false),
        ("filament_map_mode", json!(7), false),
        ("nozzle_volume_type", json!("standard"), true),
        ("nozzle_volume_type", json!("Normal"), true),
        ("nozzle_volume_type", json!("Big Traffic"), true),
        ("nozzle_volume_type", json!(7), true),
    ] {
        let value = if vector { Value::Array(vec![invalid]) } else { invalid };
        let input = json!({key: value});
        let child_error = if key == "curr_bed_type" {
            serde_json::from_value::<ProjectPrintSourceOptions>(input.clone())
                .unwrap_err()
                .to_string()
        } else {
            serde_json::from_value::<ProjectGCodeSourceOptions>(input.clone())
                .unwrap_err()
                .to_string()
        };
        assert_keyed_bounded_error(&child_error, key);
        assert_keyed_bounded_error(&parent_error(input), key);
    }
}

fn assert_scalar_round_trip<T>(key: &str, token: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let input = json!({key: token});
    let parsed: T = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()[key], json!(token));
}

fn assert_vector_round_trip<T>(key: &str, token: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let input = json!({key: [token]});
    let parsed: T = serde_json::from_value(input).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()[key], json!([token]));
}
