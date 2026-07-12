use super::*;
use serde_json::json;

#[test]
fn multiple_filament_float_and_percent_copy_follows_filament_map() {
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
        "filament_max_volumetric_speed": [8.5, 12.25],
        "filament_retract_before_wipe": [35.0, 75.0]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &[
            "filament_max_volumetric_speed",
            "filament_retract_before_wipe",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_max_volumetric_speed"],
        json!([12.25, 8.5])
    );
    assert_eq!(
        target.values()["filament_retract_before_wipe"],
        json!([75.0, 35.0])
    );
}

#[test]
fn multiple_filament_numeric_out_of_range_and_empty_vectors_leave_zero_slots() {
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
        "filament_max_volumetric_speed": [8.5],
        "filament_retract_before_wipe": []
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &[
            "filament_max_volumetric_speed",
            "filament_retract_before_wipe",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_max_volumetric_speed"],
        json!([8.5, 0])
    );
    assert_eq!(
        target.values()["filament_retract_before_wipe"],
        json!([0, 0])
    );
}

#[test]
fn multiple_filament_nullable_numeric_nil_entries_are_preserved() {
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
        "filament_retraction_length": [0.8, "nil"]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["filament_retraction_length"],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_retraction_length"],
        json!(["nil", 0.8])
    );
}

#[test]
fn multiple_filament_invalid_numeric_inputs_do_not_partially_mutate() {
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
            "filament_retract_before_wipe": [35.0, "bad"]
        }),
        json!({
            "filament_self_index": [1, 2],
            "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "filament_max_volumetric_speed": [8.5, "nil"],
            "filament_retract_before_wipe": [35.0, 75.0]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update_multiple_filaments(
            &mut target,
            &printer_config,
            &[
                "filament_max_volumetric_speed",
                "filament_retract_before_wipe",
            ],
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn multiple_filament_float_or_percent_bool_and_enum_update_after_enum_milestone() {
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
        "sparse_infill_line_width": ["10%", "20%"],
        "enable_pressure_advance": [true, false],
        "z_hop_types": ["Auto Lift", "Slope Lift"],
        "filament_max_volumetric_speed": [8.5, 12.25]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &[
            "sparse_infill_line_width",
            "enable_pressure_advance",
            "z_hop_types",
            "filament_max_volumetric_speed",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["sparse_infill_line_width"],
        json!(["20%", "10%"])
    );
    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([false, true])
    );
    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Auto Lift"])
    );
    assert_eq!(
        target.values()["filament_max_volumetric_speed"],
        json!([12.25, 8.5])
    );
}
