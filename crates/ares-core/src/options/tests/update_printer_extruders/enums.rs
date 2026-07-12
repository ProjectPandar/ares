use super::*;
use serde_json::json;

#[test]
fn selected_extruder_id_copies_enum_values_with_stride() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "z_hop_types": ["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"]
    }));

    update(&mut target, &printer_config, &["z_hop_types"], 2).unwrap();

    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Spiral Lift"])
    );
}

#[test]
fn all_extruders_copy_enum_values_in_printer_order() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Bowden", "Direct Drive"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [2, 1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "z_hop_types": ["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"]
    }));

    update(&mut target, &printer_config, &["z_hop_types"], 0).unwrap();

    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Spiral Lift", "Auto Lift", "Normal Lift"])
    );
}

#[test]
fn enum_get_at_falls_back_to_first_value_for_short_vectors() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "z_hop_types": ["Slope Lift"]
    }));

    update(&mut target, &printer_config, &["z_hop_types"], 0).unwrap();

    assert_eq!(
        target.values()["z_hop_types"],
        json!(["Slope Lift", "Slope Lift", "Slope Lift", "Slope Lift"])
    );
}

#[test]
fn nullable_enum_nil_entries_are_preserved() {
    let printer_config = printer(json!({
        "nozzle_diameter": [0.4, 0.6],
        "extruder_variant_list": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive", "Bowden"],
        "nozzle_volume_type": ["Standard", "Standard"]
    }));
    let mut target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "filament_z_hop_types": ["Auto Lift", "nil", "nil", "Spiral Lift"]
    }));

    update(&mut target, &printer_config, &["filament_z_hop_types"], 2).unwrap();

    assert_eq!(
        target.values()["filament_z_hop_types"],
        json!(["nil", "Spiral Lift"])
    );
}

#[test]
fn invalid_enum_inputs_do_not_partially_mutate() {
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
            "z_hop_types": ["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
            "nozzle_volume_type": []
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "z_hop_types": ["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
            "nozzle_volume_type": ["Standard", 0]
        }),
        json!({
            "printer_extruder_id": [1, 2],
            "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
            "z_hop_types": ["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
            "nozzle_volume_type": ["Standard", "nil"]
        }),
    ] {
        let mut target = options(value);
        let before = target.clone();

        let result = update(
            &mut target,
            &printer_config,
            &["z_hop_types", "nozzle_volume_type"],
            0,
        );

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}
