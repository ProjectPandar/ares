use super::*;
use serde_json::json;

#[test]
fn multiple_filament_float_or_percent_copy_follows_filament_map() {
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
        "sparse_infill_line_width": [0.42, "20%"],
        "bridge_acceleration": ["50%", 1200.0]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["sparse_infill_line_width", "bridge_acceleration"],
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["20%", 0.42])
    );
    assert_eq!(
        target.values()["bridge_acceleration"],
        json!([1200.0, "50%"])
    );
}

#[test]
fn multiple_filament_float_or_percent_out_of_range_and_empty_vectors_leave_zero_slots() {
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
        "sparse_infill_line_width": ["10%"],
        "bridge_acceleration": []
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["sparse_infill_line_width", "bridge_acceleration"],
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["10%", 0])
    );
    assert_eq!(target.values()["bridge_acceleration"], json!([0, 0]));
}

#[test]
fn multiple_filament_invalid_float_or_percent_inputs_do_not_partially_mutate() {
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
            "sparse_infill_line_width": ["10%", "bad"]
        }),
        json!({
            "filament_self_index": [1, 2],
            "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "filament_max_volumetric_speed": [8.5, 12.25],
            "sparse_infill_line_width": ["10%", "nil"]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update_multiple_filaments(
            &mut target,
            &printer_config,
            &["filament_max_volumetric_speed", "sparse_infill_line_width"],
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn multiple_filament_bool_and_enum_keys_update_after_enum_milestone() {
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
        "z_hop_types": ["Auto Lift", "Slope Lift"],
        "sparse_infill_line_width": [0.42, "20%"]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &[
            "enable_pressure_advance",
            "z_hop_types",
            "sparse_infill_line_width",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true])
    );
    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Auto Lift"])
    );
    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["20%", 0.42])
    );
}
