use super::super::filament_change::FilamentChangeOptions;
use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

fn options(extra: Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}

fn filament_change_options(extra: Value) -> Result<FilamentChangeOptions, SliceError> {
    options(extra).filament_change_options()
}

#[test]
fn filament_change_options_default_to_orca_values() {
    let options = SliceOptions::default().filament_change_options().unwrap();

    assert!(options.single_extruder_multi_material());
    assert!(!options.manual_filament_change());
    assert!(!options.single_extruder_multi_material_priming());
}

#[test]
fn filament_change_options_accept_boolean_values() {
    for (single_extruder_multi_material, manual_filament_change, priming) in [
        (true, true, true),
        (true, false, false),
        (false, true, true),
        (false, false, false),
    ] {
        let options = filament_change_options(json!({
            "single_extruder_multi_material": single_extruder_multi_material,
            "manual_filament_change": manual_filament_change,
            "single_extruder_multi_material_priming": priming
        }))
        .unwrap();

        assert_eq!(
            options.single_extruder_multi_material(),
            single_extruder_multi_material
        );
        assert_eq!(options.manual_filament_change(), manual_filament_change);
        assert_eq!(options.single_extruder_multi_material_priming(), priming);
    }
}

#[test]
fn single_extruder_multi_material_rejects_non_boolean_values() {
    for value in [
        json!(0),
        json!("true"),
        json!([]),
        json!({ "value": true }),
        Value::Null,
    ] {
        let err =
            filament_change_options(json!({ "single_extruder_multi_material": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("single_extruder_multi_material"));
    }
}

#[test]
fn manual_filament_change_rejects_non_boolean_values() {
    for value in [
        json!(0),
        json!("false"),
        json!([]),
        json!({ "value": false }),
        Value::Null,
    ] {
        let err = filament_change_options(json!({ "manual_filament_change": value })).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("manual_filament_change"));
    }
}

#[test]
fn single_extruder_multi_material_priming_rejects_non_boolean_values() {
    for value in [
        json!(0),
        json!("true"),
        json!([]),
        json!({ "value": true }),
        Value::Null,
    ] {
        let err = filament_change_options(json!({
            "single_extruder_multi_material_priming": value
        }))
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("single_extruder_multi_material_priming")
        );
    }
}

#[test]
fn filament_change_options_can_be_consumed_as_runtime_state() {
    filament_change_options(json!({
        "single_extruder_multi_material_priming": true
    }))
    .unwrap()
    .consume_runtime();
}
