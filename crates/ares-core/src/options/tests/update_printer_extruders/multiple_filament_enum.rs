use super::*;
use serde_json::json;

#[test]
fn multiple_filament_enum_copy_follows_filament_map() {
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
        "retract_lift_enforce": ["All Surfaces", "Top Only"]
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["z_hop_types", "retract_lift_enforce"],
    )
    .unwrap();

    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Auto Lift"])
    );
    assert_eq!(
        target.values()["retract_lift_enforce"],
        json!(["Top Only", "All Surfaces"])
    );
}

#[test]
fn multiple_filament_enum_out_of_range_and_empty_vectors_leave_empty_slots() {
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
        "z_hop_types": ["Auto Lift"],
        "retract_lift_enforce": []
    }));

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &["z_hop_types", "retract_lift_enforce"],
    )
    .unwrap();

    assert_eq!(target.values()["z_hop_types"], json!(["Auto Lift", ""]));
    assert_eq!(target.values()["retract_lift_enforce"], json!(["", ""]));
}

#[test]
fn multiple_filament_nullable_enum_nil_entries_are_preserved() {
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
        "filament_z_hop_types": ["Auto Lift", "nil"]
    }));

    update_multiple_filaments(&mut target, &printer_config, &["filament_z_hop_types"]).unwrap();

    assert_eq!(
        target.values()["filament_z_hop_types"],
        json!(["nil", "Auto Lift"])
    );
}

#[test]
fn multiple_filament_invalid_enum_inputs_do_not_partially_mutate() {
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
            "z_hop_types": ["Auto Lift", 1]
        }),
        json!({
            "filament_self_index": [1, 2],
            "filament_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "filament_max_volumetric_speed": [8.5, 12.25],
            "z_hop_types": ["Auto Lift", "nil"]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update_multiple_filaments(
            &mut target,
            &printer_config,
            &["filament_max_volumetric_speed", "z_hop_types"],
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn multiple_filament_unknown_missing_and_scalar_keys_still_skip_after_enum() {
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
        "wall_sequence": "inner wall/outer wall",
        "z_hop_types": ["Auto Lift", "Slope Lift"]
    }));
    let scalar_before = target.values()["wall_sequence"].clone();

    update_multiple_filaments(
        &mut target,
        &printer_config,
        &[
            "unknown_key",
            "missing_defined_key",
            "wall_sequence",
            "z_hop_types",
        ],
    )
    .unwrap();

    assert_eq!(target.values()["wall_sequence"], scalar_before);
    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Auto Lift"])
    );
}
