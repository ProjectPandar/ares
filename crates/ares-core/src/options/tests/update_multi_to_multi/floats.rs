use crate::SliceError;
use serde_json::json;

use super::{options, update};

#[test]
fn float_keys_copy_source_values_when_no_old_value_is_lower() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [150.0, 160.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "fan_max_speed": [90.0, 80.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([90.0, 80.0]));
}

#[test]
fn float_keys_preserve_lower_old_same_variant_values() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [40.0, 120.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "fan_max_speed": [90.0, 80.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([40.0, 80.0]));
}

#[test]
fn float_keys_consider_duplicate_old_variant_indices() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [70.0, 30.0, 20.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "fan_max_speed": [90.0, 80.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([30.0, 20.0]));
}

#[test]
fn float_keys_fallback_to_all_old_indices_when_variant_missing() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Legacy A", "Legacy B"],
        "fan_max_speed": [60.0, 20.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "fan_max_speed": [90.0, 80.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([20.0, 20.0]));
}

#[test]
fn float_keys_skip_merge_when_new_variant_index_is_missing() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [40.0, 20.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [2, 1],
        "fan_max_speed": [90.0, 80.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([90.0, 80.0]));
}

#[test]
fn missing_current_float_uses_registry_default_before_merge() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "fan_max_speed": [150.0, 160.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([100.0, 100.0]));
}

#[test]
fn invalid_float_values_or_lengths_return_invalid_input_without_mutation() {
    let cases = [
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "fan_max_speed": [50.0]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "fan_max_speed": ["fast"]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "fan_max_speed": ["slow"]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "fan_max_speed": [90.0]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "fan_max_speed": [50.0]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "printer_extruder_id": [1, 2],
                "fan_max_speed": [90.0, 80.0]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "fan_max_speed": [50.0]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "printer_extruder_id": [1, 2],
                "fan_max_speed": [90.0]
            }),
        ),
    ];

    for (target_value, source_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let source = options(source_value);

        let result = update(&mut target, &source, &["fan_max_speed"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn representative_float_option_names_merge() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_printable_height": [90.0, 80.0],
        "fan_cooling_layer_time": [50.0, 60.0],
        "fan_max_speed": [70.0, 20.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "extruder_printable_height": [100.0, 70.0],
        "fan_cooling_layer_time": [40.0, 90.0],
        "fan_max_speed": [90.0, 80.0]
    }));

    update(
        &mut target,
        &source,
        &[
            "extruder_printable_height",
            "fan_cooling_layer_time",
            "fan_max_speed",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["extruder_printable_height"],
        json!([90.0, 70.0])
    );
    assert_eq!(
        target.values()["fan_cooling_layer_time"],
        json!([40.0, 60.0])
    );
    assert_eq!(target.values()["fan_max_speed"], json!([70.0, 20.0]));
}
