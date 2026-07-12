use crate::SliceError;
use serde_json::json;

use super::{options, update};

#[test]
fn bool_keys_copy_source_values_when_no_old_value_is_true() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "activate_air_filtration": [false, false]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "activate_air_filtration": [true, false]
    }));

    update(&mut target, &source, &["activate_air_filtration"]).unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, false])
    );
}

#[test]
fn bool_keys_preserve_old_same_variant_true_values() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "activate_air_filtration": [true, false]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "activate_air_filtration": [false, true]
    }));

    update(&mut target, &source, &["activate_air_filtration"]).unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, true])
    );
}

#[test]
fn bool_keys_consider_duplicate_old_variant_indices() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard", "Bowden Standard"],
        "activate_air_filtration": [false, true, true]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "activate_air_filtration": [false, false]
    }));

    update(&mut target, &source, &["activate_air_filtration"]).unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, true])
    );
}

#[test]
fn bool_keys_fallback_to_all_old_indices_when_variant_missing() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Legacy A", "Legacy B"],
        "activate_air_filtration": [false, true]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "activate_air_filtration": [false, false]
    }));

    update(&mut target, &source, &["activate_air_filtration"]).unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, true])
    );
}

#[test]
fn bool_keys_skip_merge_when_new_variant_index_is_missing() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "activate_air_filtration": [true, true]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [2, 1],
        "activate_air_filtration": [false, false]
    }));

    update(&mut target, &source, &["activate_air_filtration"]).unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([false, false])
    );
}

#[test]
fn missing_current_bool_uses_registry_default_before_merge() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "activate_air_filtration": [true, false]
    }));

    update(&mut target, &source, &["activate_air_filtration"]).unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, false])
    );
}

#[test]
fn invalid_bool_values_or_lengths_return_invalid_input_without_mutation() {
    let cases = [
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "activate_air_filtration": [false]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "activate_air_filtration": ["true"]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "activate_air_filtration": ["false"]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "activate_air_filtration": [true]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "activate_air_filtration": [false]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "printer_extruder_id": [1, 2],
                "activate_air_filtration": [true, false]
            }),
        ),
        (
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "activate_air_filtration": [false]
            }),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "printer_extruder_id": [1, 2],
                "activate_air_filtration": [true]
            }),
        ),
    ];

    for (target_value, source_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let source = options(source_value);

        let result = update(&mut target, &source, &["activate_air_filtration"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn representative_bool_option_names_merge() {
    let mut target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "activate_air_filtration": [true, false],
        "enable_pressure_advance": [false, true],
        "filament_is_support": [true, false],
        "wipe": [false, true]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "activate_air_filtration": [false, false],
        "enable_pressure_advance": [false, false],
        "filament_is_support": [false, false],
        "wipe": [false, false]
    }));

    update(
        &mut target,
        &source,
        &[
            "activate_air_filtration",
            "enable_pressure_advance",
            "filament_is_support",
            "wipe",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, false])
    );
    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true])
    );
    assert_eq!(target.values()["filament_is_support"], json!([true, false]));
    assert_eq!(target.values()["wipe"], json!([false, true]));
}
