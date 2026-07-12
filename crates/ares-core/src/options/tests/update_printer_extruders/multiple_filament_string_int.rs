use super::*;
use serde_json::json;

#[test]
fn multiple_filament_single_extruder_guard_does_not_mutate() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4],
        "filament_map": [1]
    }));
    let mut target = options(json!({
        "filament_self_index": [1],
        "filament_extruder_variant": ["Direct Drive Standard"],
        "filament_ramming_parameters": ["ram"]
    }));
    let before = target.clone();

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_self_index", "filament_ramming_parameters"],
    )
    .unwrap();

    assert_eq!(target, before);
}

#[test]
fn multiple_filament_missing_prerequisites_skip_when_guard_passes() {
    for printer_config in [
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "extruder_type": ["Direct Drive", "Bowden"],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "filament_map": [1, 2],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "filament_map": [1, 2],
            "extruder_type": ["Direct Drive", "Bowden"]
        })),
    ] {
        let mut target = options(json!({
            "filament_self_index": [1, 2],
            "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "filament_ramming_parameters": ["direct", "bowden"]
        }));
        let before = target.clone();

        update_multiple_filaments(
            &mut target,
            &printer_config,
            &["filament_self_index", "filament_ramming_parameters"],
        )
        .unwrap();

        assert_eq!(target, before);
    }
}

#[test]
fn multiple_filament_string_and_int_copy_follows_filament_map() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "filament_map": [2, 1],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "filament_self_index": [2, 1],
        "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_ramming_parameters": ["direct", "bowden"],
        "filament_map": [11, 22]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_ramming_parameters", "filament_map"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_ramming_parameters"],
        json!(["bowden", "direct"])
    );
    assert_eq!(target.values()["filament_map"], json!([22, 11]));
}

#[test]
fn multiple_filament_negative_lookup_falls_back_to_matching_id_then_zero() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden High Flow"],
        "filament_map": [2, 2],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "High Flow"]
    }));
    let mut target = options(json!({
        "filament_self_index": [9, 2],
        "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_ramming_parameters": ["fallback-zero", "fallback-id"],
        "filament_map": [10, 20]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_ramming_parameters", "filament_map"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_ramming_parameters"],
        json!(["fallback-zero", "fallback-id"])
    );
    assert_eq!(target.values()["filament_map"], json!([10, 20]));
}

#[test]
fn multiple_filament_out_of_range_variant_indices_leave_default_slots() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "filament_map": [1, 2],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "filament_self_index": [1, 2],
        "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_ramming_parameters": ["direct"],
        "filament_map": [7]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_ramming_parameters", "filament_map"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_ramming_parameters"],
        json!(["direct", ""])
    );
    assert_eq!(target.values()["filament_map"], json!([7, 0]));
}

#[test]
fn multiple_filament_empty_source_vectors_leave_default_slots() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "filament_map": [1, 2],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "filament_self_index": [1, 2],
        "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_ramming_parameters": [],
        "filament_map": []
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_ramming_parameters", "filament_map"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_ramming_parameters"],
        json!(["", ""])
    );
    assert_eq!(target.values()["filament_map"], json!([0, 0]));
}

#[test]
fn multiple_filament_invalid_inputs_do_not_partially_mutate() {
    let base_printer = json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "filament_map": [1, 2],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    });
    for (printer_config, target_value) in [
        (
            printer(json!({
                "nozzle_diameter": [0.4, 0.6],
                "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
                "filament_map": [0, 2],
                "extruder_type": ["Direct Drive", "Bowden"],
                "nozzle_volume_type": ["Standard", "Standard"]
            })),
            json!({
                "filament_self_index": [1, 2],
                "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "filament_ramming_parameters": ["direct", "bowden"],
                "filament_map": [1, 2]
            }),
        ),
        (
            printer(base_printer.clone()),
            json!({
                "filament_self_index": [1, 2],
                "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "filament_ramming_parameters": ["direct", 7],
                "filament_map": [1, 2]
            }),
        ),
        (
            printer(base_printer.clone()),
            json!({
                "filament_self_index": [1, 2],
                "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
                "filament_ramming_parameters": ["direct", "bowden"],
                "filament_map": [1, i64::from(i32::MAX) + 1]
            }),
        ),
    ] {
        let mut target = options(target_value);
        let before = target.clone();

        let result = update_multiple_filaments(
            &mut target,
            &printer_config,
            &["filament_ramming_parameters", "filament_map"],
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn multiple_filament_unknown_missing_and_unsupported_keys_are_skipped() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "filament_map": [1, 2],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "filament_self_index": [1, 2],
        "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_ramming_parameters": ["direct", "bowden"],
        "filament_flow_ratio": [0.9, 1.1]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &[
            "unknown",
            "missing",
            "filament_flow_ratio",
            "filament_ramming_parameters",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_ramming_parameters"],
        json!(["direct", "bowden"])
    );
    assert_eq!(target.values()["filament_flow_ratio"], json!([0.9, 1.1]));
}
