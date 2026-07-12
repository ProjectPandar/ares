use super::*;
use serde_json::json;

#[test]
fn non_different_single_extruder_guard_does_not_mutate() {
    let printer_config = printer(json!({ "nozzle_diameter": [0.4] }));
    let mut target = options(json!({
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["Direct Drive Standard"],
        "print_extruder_id": [1, 1],
        "print_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard"]
    }));
    let before = target.clone();

    update(
        &mut target,
        &printer_config,
        &["print_extruder_id", "print_extruder_variant"],
        0,
    )
    .unwrap();

    assert_eq!(target, before);
}

#[test]
fn missing_printer_enum_vectors_skip_when_guard_passes() {
    for printer_config in [
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "extruder_type": ["Direct Drive", "Bowden"]
        })),
    ] {
        let mut target = options(json!({
            "print_extruder_id": [1, 1],
            "print_extruder_variant": ["Direct Drive Standard", "Direct Drive Standard"]
        }));
        let before = target.clone();

        update(
            &mut target,
            &printer_config,
            &["print_extruder_id", "print_extruder_variant"],
            0,
        )
        .unwrap();

        assert_eq!(target, before);
    }
}

#[test]
fn selected_extruder_id_copies_string_and_int_values_with_stride() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "print_extruder_id": [1, 1, 2, 2],
        "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
    }));

    update(
        &mut target,
        &printer_config,
        &["print_extruder_id", "print_extruder_variant"],
        2,
    )
    .unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([2, 2]));
    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["Bowden Standard", "BB"])
    );
}

#[test]
fn all_extruders_copy_string_and_int_values_in_printer_order() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Bowden", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [2, 1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "print_extruder_id": [1, 1, 2, 2],
        "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
    }));

    update(
        &mut target,
        &printer_config,
        &["print_extruder_id", "print_extruder_variant"],
        0,
    )
    .unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([2, 2, 1, 1]));
    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["Bowden Standard", "BB", "Direct Drive Standard", "AA"])
    );
}

#[test]
fn all_extruders_negative_lookup_falls_back_to_zero() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "High Flow"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "print_extruder_id": [1, 1, 2, 2],
        "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
    }));

    update(
        &mut target,
        &printer_config,
        &["print_extruder_id", "print_extruder_variant"],
        0,
    )
    .unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([1, 1, 1, 1]));
    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["Direct Drive Standard", "AA", "Direct Drive Standard", "AA"])
    );
}

#[test]
fn source_get_at_falls_back_to_first_value_for_short_vectors() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "print_extruder_id": [9],
        "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
    }));

    update(&mut target, &printer_config, &["print_extruder_id"], 0).unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([9, 9, 9, 9]));
}

#[test]
fn unknown_missing_and_unsupported_keys_are_skipped() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "print_extruder_id": [1, 1, 2, 2],
        "curr_bed_type": ["Textured PEI Plate", "Cool Plate"]
    }));

    update(
        &mut target,
        &printer_config,
        &["unknown_key", "missing_key", "curr_bed_type"],
        0,
    )
    .unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([1, 1, 2, 2]));
    assert_eq!(
        target.values()["curr_bed_type"],
        json!(["Textured PEI Plate", "Cool Plate"])
    );
}

#[test]
fn invalid_inputs_do_not_partially_mutate() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    for value in [
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "print_extruder_id": [1, 1, 2, 2],
            "print_extruder_variant": ["Direct Drive Standard", 7, "Bowden Standard", "BB"]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "print_extruder_id": [],
            "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "print_extruder_id": [1, i64::from(i32::MAX) + 1, 2, 2],
            "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update(
            &mut target,
            &printer_config,
            &["print_extruder_variant", "print_extruder_id"],
            0,
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn invalid_printer_enums_do_not_mutate() {
    for printer_config in [
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "extruder_type": ["Direct Drive", 7],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
        printer(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
            "extruder_type": ["Direct Drive", "Cartesian"],
            "nozzle_volume_type": ["Standard", "Standard"]
        })),
    ] {
        let mut target = options(json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "print_extruder_id": [1, 1, 2, 2],
            "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
        }));
        let before = target.clone();

        let result = update(&mut target, &printer_config, &["print_extruder_id"], 0);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn selected_extruder_negative_lookup_returns_invalid_input_without_mutation() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "High Flow"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "print_extruder_id": [1, 1, 2, 2],
        "print_extruder_variant": ["Direct Drive Standard", "AA", "Bowden Standard", "BB"]
    }));
    let before = target.clone();

    let result = update(&mut target, &printer_config, &["print_extruder_id"], 2);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}
