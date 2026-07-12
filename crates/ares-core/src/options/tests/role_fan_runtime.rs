use super::super::*;
use crate::PrintPathRole;
use serde_json::{Value, json};

#[test]
fn role_fan_control_uses_overhang_speed_above_baseline() {
    let options = options(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }));
    let layer = layer_role_fans(&options, 0, Some(40));

    assert_eq!(layer.speed_for_role(PrintPathRole::Bridge), Some(75));
    assert_eq!(
        layer.speed_for_role(PrintPathRole::InternalBridge),
        Some(75)
    );
    assert_eq!(layer.speed_for_role(PrintPathRole::SparseInfill), None);
    assert_eq!(layer.speed_for_role(PrintPathRole::SolidInfill), None);
}

#[test]
fn role_fan_control_suppresses_overhang_speed_not_above_baseline() {
    let options = options(json!({
        "fan_min_speed": 100,
        "fan_max_speed": 100,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 100
    }));
    let layer = layer_role_fans(&options, 0, Some(100));

    assert_eq!(layer.speed_for_role(PrintPathRole::Bridge), None);
    assert_eq!(layer.speed_for_role(PrintPathRole::InternalBridge), None);
}

#[test]
fn role_fan_control_accepts_overhang_speed_first_entry_forms() {
    for value in [
        json!(75),
        json!("75"),
        json!([75, 40]),
        json!("75;40"),
        json!("75,40"),
    ] {
        let options = options(json!({
            "fan_max_speed": 0,
            "close_fan_the_first_x_layers": 0,
            "overhang_fan_speed": value
        }));
        let layer = layer_role_fans(&options, 0, None);

        assert_eq!(layer.speed_for_role(PrintPathRole::Bridge), Some(75));
    }
}

#[test]
fn role_fan_control_ramps_overhang_and_fallback_but_not_explicit_internal_bridge() {
    let fallback_options = options(json!({
        "fan_max_speed": 0,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 100,
        "internal_bridge_fan_speed": -1
    }));
    let fallback_layer = layer_role_fans(&fallback_options, 0, None);

    assert_eq!(fallback_layer.speed_for_role(PrintPathRole::Bridge), Some(25));
    assert_eq!(
        fallback_layer.speed_for_role(PrintPathRole::InternalBridge),
        Some(25)
    );

    let explicit_options = options(json!({
        "fan_max_speed": 0,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 100,
        "internal_bridge_fan_speed": 75
    }));
    let explicit_layer = layer_role_fans(&explicit_options, 0, None);

    assert_eq!(explicit_layer.speed_for_role(PrintPathRole::Bridge), Some(25));
    assert_eq!(
        explicit_layer.speed_for_role(PrintPathRole::InternalBridge),
        Some(75)
    );
}

#[test]
fn role_fan_control_respects_enabled_gate_and_close_first_layers() {
    let disabled = options(json!({
        "enable_overhang_bridge_fan": false,
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "internal_bridge_fan_speed": 75
    }));
    let disabled_layer = layer_role_fans(&disabled, 0, None);

    assert_eq!(disabled_layer.speed_for_role(PrintPathRole::Bridge), None);
    assert_eq!(
        disabled_layer.speed_for_role(PrintPathRole::InternalBridge),
        None
    );

    let first_layer_closed = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 1,
        "overhang_fan_speed": 75,
        "internal_bridge_fan_speed": 75
    }));
    let closed_layer = layer_role_fans(&first_layer_closed, 0, None);

    assert_eq!(closed_layer.speed_for_role(PrintPathRole::Bridge), None);
    assert_eq!(
        closed_layer.speed_for_role(PrintPathRole::InternalBridge),
        None
    );
}

#[test]
fn role_fan_control_rejects_invalid_inputs() {
    for (key, value) in [
        ("enable_overhang_bridge_fan", json!("true")),
        ("enable_overhang_bridge_fan", json!(1)),
        ("enable_overhang_bridge_fan", Value::Null),
        ("overhang_fan_speed", json!(-1)),
        ("overhang_fan_speed", json!(101)),
        ("overhang_fan_speed", json!(75.5)),
        ("overhang_fan_speed", json!("75.5")),
        ("overhang_fan_speed", json!("75;")),
        ("overhang_fan_speed", json!([])),
        ("overhang_fan_speed", json!(["75"])),
        ("overhang_fan_speed", json!({"value": 75})),
        ("overhang_fan_speed", json!(true)),
        ("overhang_fan_speed", Value::Null),
        ("internal_bridge_fan_speed", json!(101)),
    ] {
        let mut values = serde_json::Map::new();
        values.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

        let err = options.role_fan_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

fn layer_role_fans(
    options: &SliceOptions,
    layer_index: usize,
    baseline_speed: Option<u8>,
) -> LayerRoleFanControl {
    options.role_fan_control().unwrap().for_layer(
        options.part_cooling_fan_ramp().unwrap(),
        layer_index,
        baseline_speed,
    )
}

fn options(extra: serde_json::Value) -> SliceOptions {
    serde_json::from_value(extra).unwrap()
}
