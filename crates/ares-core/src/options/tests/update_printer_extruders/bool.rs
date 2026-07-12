use super::*;
use serde_json::json;

#[test]
fn selected_extruder_id_copies_bool_values_with_stride() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "enable_pressure_advance": [true, false, false, true]
    }));

    update(
        &mut target,
        &printer_config,
        &["enable_pressure_advance"],
        2,
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true])
    );
}

#[test]
fn all_extruders_copy_bool_values_in_printer_order() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Bowden", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [2, 1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "enable_pressure_advance": [true, false, false, true]
    }));

    update(
        &mut target,
        &printer_config,
        &["enable_pressure_advance"],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true, true, false])
    );
}

#[test]
fn bool_get_at_falls_back_to_first_value_for_short_vectors() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "enable_pressure_advance": [true]
    }));

    update(
        &mut target,
        &printer_config,
        &["enable_pressure_advance"],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([true, true, true, true])
    );
}

#[test]
fn nullable_bool_nil_entries_are_preserved() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_retract_when_changing_layer": [true, "nil", "nil", false]
    }));

    update(
        &mut target,
        &printer_config,
        &["filament_retract_when_changing_layer"],
        2,
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_retract_when_changing_layer"],
        json!(["nil", false])
    );
}

#[test]
fn invalid_bool_inputs_do_not_partially_mutate() {
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
            "activate_air_filtration": [false, true, false, true],
            "enable_pressure_advance": []
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "activate_air_filtration": [false, true, false, true],
            "enable_pressure_advance": [true, 0, false, true]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "activate_air_filtration": [false, true, false, true],
            "enable_pressure_advance": [true, 1, false, true]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "activate_air_filtration": [false, true, false, true],
            "enable_pressure_advance": [true, "0", false, true]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "activate_air_filtration": [false, true, false, true],
            "enable_pressure_advance": [true, "1", false, true]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "activate_air_filtration": [false, true, false, true],
            "enable_pressure_advance": [true, "nil", false, true]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "activate_air_filtration": [false, true, false, true],
            "filament_retract_when_changing_layer": [true, "bad", false, true]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update(
            &mut target,
            &printer_config,
            &[
                "activate_air_filtration",
                "enable_pressure_advance",
                "filament_retract_when_changing_layer",
            ],
            0,
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn bool_updates_while_scalar_enum_keys_remain_skipped() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Bowden", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [2, 1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "enable_pressure_advance": [true, false, false, true],
        "curr_bed_type": ["Textured PEI Plate", "Cool Plate", "Textured PEI Plate", "Cool Plate"]
    }));
    let enum_before = target.values()["curr_bed_type"].clone();

    update(
        &mut target,
        &printer_config,
        &["enable_pressure_advance", "curr_bed_type"],
        0,
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true, true, false])
    );
    assert_eq!(target.values()["curr_bed_type"], enum_before);
}
