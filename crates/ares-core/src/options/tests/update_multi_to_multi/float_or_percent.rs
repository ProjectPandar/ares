use crate::SliceError;
use serde_json::json;

use super::{options, update};

#[test]
fn float_or_percent_keys_copy_source_values_when_no_old_value_is_lower() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "bridge_acceleration": ["120%", 150.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "bridge_acceleration": ["90%", 80.0]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!(["90%", 80.0]));
}

#[test]
fn float_or_percent_keys_preserve_lower_old_same_variant_values_and_percent_flag() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "bridge_acceleration": ["40%", 120.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "bridge_acceleration": [90.0, "80%"]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!(["40%", "80%"])
    );
}

#[test]
fn float_or_percent_equal_numeric_value_prefers_absolute_old_value() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "bridge_acceleration": [50.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [1],
        "bridge_acceleration": ["50%"]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([50.0]));
}

#[test]
fn float_or_percent_keys_consider_duplicate_old_variant_indices() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard", "Bowden Standard"],
        "bridge_acceleration": ["70%", 30.0, "20%"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "bridge_acceleration": [90.0, 80.0]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([30.0, "20%"]));
}

#[test]
fn float_or_percent_keys_fallback_to_all_old_indices_when_variant_missing() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Legacy A", "Legacy B"],
        "bridge_acceleration": ["60%", 20.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "bridge_acceleration": [90.0, 80.0]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([20.0, 20.0]));
}

#[test]
fn float_or_percent_keys_skip_merge_when_new_variant_index_is_missing() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "bridge_acceleration": ["40%", 20.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [2, 1],
        "bridge_acceleration": [90.0, "80%"]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([90.0, "80%"]));
}

#[test]
fn missing_current_float_or_percent_uses_registry_default_before_merge() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "bridge_acceleration": [150.0, "160%"]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(
        target.values()["bridge_acceleration"],
        json!(["50%", "50%"])
    );
}

#[test]
fn invalid_float_or_percent_values_or_lengths_return_invalid_input_without_mutation() {
    let cases = [
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "bridge_acceleration": [50.0]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "bridge_acceleration": ["fast"]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "bridge_acceleration": ["slow"]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "bridge_acceleration": [90.0]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "bridge_acceleration": [50.0]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "printer_extruder_id": [1, 2],
                "bridge_acceleration": [90.0, 80.0]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "bridge_acceleration": [50.0]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "printer_extruder_id": [1, 2],
                "bridge_acceleration": [90.0]
            }),
        ),
    ];

    for (target_value, source_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let source = options(source_value);

        let result = update(&mut target, &source, &["bridge_acceleration"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn representative_float_or_percent_option_names_merge() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "outer_wall_line_width": [0.3, "40%"],
        "line_width": [0.2, 0.6],
        "bridge_acceleration": ["20%", 70.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "outer_wall_line_width": [0.5, "35%"],
        "line_width": [0.1, 0.8],
        "bridge_acceleration": ["50%", 60.0]
    }));

    update(
        &mut target,
        &source,
        &["outer_wall_line_width", "line_width", "bridge_acceleration"],
    )
    .unwrap();

    assert_eq!(
        target.values()["outer_wall_line_width"],
        json!([0.3, "35%"])
    );
    assert_eq!(target.values()["line_width"], json!([0.1, 0.6]));
    assert_eq!(target.values()["bridge_acceleration"], json!(["20%", 60.0]));
}
