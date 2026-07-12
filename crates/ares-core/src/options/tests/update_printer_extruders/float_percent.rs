use super::*;
use serde_json::json;

#[test]
fn selected_extruder_id_copies_float_and_percent_values_with_stride() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_flow_ratio": [0.91, 0.92, 1.01, 1.02],
        "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
        "elefant_foot_layers_density": [11.0, 12.0, 21.0, 22.0]
    }));

    update(
        &mut target,
        &printer_config,
        &[
            "filament_flow_ratio",
            "fan_max_speed",
            "elefant_foot_layers_density",
        ],
        2,
    )
    .unwrap();

    assert_eq!(target.values()["filament_flow_ratio"], json!([1.01, 1.02]));
    assert_eq!(target.values()["fan_max_speed"], json!([90.0, 91.0]));
    assert_eq!(
        target.values()["elefant_foot_layers_density"],
        json!([21.0, 22.0])
    );
}

#[test]
fn all_extruders_copy_float_and_percent_values_in_printer_order() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Bowden", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [2, 1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_flow_ratio": [0.91, 0.92, 1.01, 1.02],
        "fan_min_speed": [20.0, 21.0, 30.0, 31.0],
        "prime_tower_infill_gap": [150.0, 151.0, 160.0, 161.0]
    }));

    update(
        &mut target,
        &printer_config,
        &[
            "filament_flow_ratio",
            "fan_min_speed",
            "prime_tower_infill_gap",
        ],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([1.01, 1.02, 0.91, 0.92])
    );
    assert_eq!(
        target.values()["fan_min_speed"],
        json!([30.0, 31.0, 20.0, 21.0])
    );
    assert_eq!(
        target.values()["prime_tower_infill_gap"],
        json!([160.0, 161.0, 150.0, 151.0])
    );
}

#[test]
fn numeric_source_get_at_falls_back_to_first_value_for_short_vectors() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [75.0],
        "prime_tower_infill_gap": [125.0]
    }));

    update(
        &mut target,
        &printer_config,
        &["fan_max_speed", "prime_tower_infill_gap"],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["fan_max_speed"],
        json!([75.0, 75.0, 75.0, 75.0])
    );
    assert_eq!(
        target.values()["prime_tower_infill_gap"],
        json!([125.0, 125.0, 125.0, 125.0])
    );
}

#[test]
fn selected_extruder_id_copies_float_or_percent_values_with_stride() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "sparse_infill_line_width": ["10%", 0.42, 0.64, "120%"]
    }));

    update(
        &mut target,
        &printer_config,
        &["sparse_infill_line_width"],
        2,
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!([0.64, "120%"])
    );
}

#[test]
fn all_extruders_copy_float_or_percent_values_in_printer_order() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Bowden", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [2, 1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "sparse_infill_line_width": [0.4, "105%", "80%", 0.65]
    }));

    update(
        &mut target,
        &printer_config,
        &["sparse_infill_line_width"],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["80%", 0.65, 0.4, "105%"])
    );
}

#[test]
fn float_or_percent_get_at_falls_back_to_first_value_for_short_vectors() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "sparse_infill_line_width": ["115%"]
    }));

    update(
        &mut target,
        &printer_config,
        &["sparse_infill_line_width"],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["115%", "115%", "115%", "115%"])
    );
}

#[test]
fn float_or_percent_numeric_strings_become_numbers_and_percent_strings_stay_percent() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "sparse_infill_line_width": ["0.44", "110%", "0.66", "125%"]
    }));

    update(
        &mut target,
        &printer_config,
        &["sparse_infill_line_width"],
        2,
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!([0.66, "125%"])
    );
}

#[test]
fn nullable_numeric_nil_entries_are_preserved() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_flow_ratio": [0.91, "nil", "nil", 1.02]
    }));

    update(&mut target, &printer_config, &["filament_flow_ratio"], 0).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.91, "nil", "nil", 1.02])
    );
}

#[test]
fn invalid_numeric_inputs_do_not_partially_mutate() {
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
            "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
            "filament_flow_ratio": []
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
            "filament_flow_ratio": [0.91, "bad", 1.01, 1.02]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update(
            &mut target,
            &printer_config,
            &["fan_max_speed", "filament_flow_ratio"],
            0,
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn invalid_float_or_percent_inputs_do_not_partially_mutate() {
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
            "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
            "sparse_infill_line_width": []
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
            "sparse_infill_line_width": ["10%", "bad", 0.6, "120%"]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
            "sparse_infill_line_width": ["10%", "nil", 0.6, "120%"]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "fan_max_speed": [80.0, 81.0, 90.0, 91.0],
            "sparse_infill_line_width": ["10%", "nan%", 0.6, "120%"]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update(
            &mut target,
            &printer_config,
            &["fan_max_speed", "sparse_infill_line_width"],
            0,
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn float_or_percent_and_bool_update_while_scalar_enum_keys_remain_skipped() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "sparse_infill_line_width": ["10%", "20%", "30%", "40%"],
        "enable_pressure_advance": [true, false, true, false],
        "curr_bed_type": ["Textured PEI Plate", "Cool Plate", "Textured PEI Plate", "Cool Plate"]
    }));
    let enum_before = target.values()["curr_bed_type"].clone();

    update(
        &mut target,
        &printer_config,
        &[
            "sparse_infill_line_width",
            "enable_pressure_advance",
            "curr_bed_type",
        ],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["10%", "20%", "30%", "40%"])
    );
    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([true, false, true, false])
    );
    assert_eq!(target.values()["curr_bed_type"], enum_before);
}
