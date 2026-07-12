use super::*;
use serde_json::json;

#[test]
fn multiple_filament_bool_copy_follows_filament_map() {
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
        "enable_pressure_advance": [true, false],
        "retract_when_changing_layer": [false, true]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["enable_pressure_advance", "retract_when_changing_layer"],
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true])
    );
    assert_eq!(
        target.values()["retract_when_changing_layer"],
        json!([true, false])
    );
}

#[test]
fn multiple_filament_bool_out_of_range_and_empty_vectors_leave_false_slots() {
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
        "enable_pressure_advance": [true],
        "retract_when_changing_layer": []
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["enable_pressure_advance", "retract_when_changing_layer"],
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([true, false])
    );
    assert_eq!(
        target.values()["retract_when_changing_layer"],
        json!([false, false])
    );
}

#[test]
fn multiple_filament_nullable_bool_nil_entries_are_preserved() {
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
        "filament_retract_when_changing_layer": [true, "nil"]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_retract_when_changing_layer"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_retract_when_changing_layer"],
        json!(["nil", true])
    );
}

#[test]
fn multiple_filament_invalid_bool_inputs_do_not_partially_mutate() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "filament_map": [1, 2],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    for value in [
        json!({
            "filament_self_index": [1, 2],
            "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "filament_max_volumetric_speed": [8.5, 12.25],
            "enable_pressure_advance": [true, 1]
        }),
        json!({
            "filament_self_index": [1, 2],
            "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "filament_max_volumetric_speed": [8.5, 12.25],
            "enable_pressure_advance": [true, "nil"]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update_multiple_filaments(
            &mut target,
            &printer_config,
            &["filament_max_volumetric_speed", "enable_pressure_advance"],
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn multiple_filament_enum_keys_update_after_enum_milestone() {
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
        "z_hop_types": ["Auto Lift", "Slope Lift"],
        "enable_pressure_advance": [true, false]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["z_hop_types", "enable_pressure_advance"],
    )
    .unwrap();

    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Auto Lift"])
    );
    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true])
    );
}
