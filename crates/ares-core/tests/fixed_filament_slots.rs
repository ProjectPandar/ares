//! Public preset-selection seam for `PresetBundle::min_filament_count` and
//! `update_filament_count` (v2.5.0 40ecc66f7, PresetBundle.cpp:5419-5436).

use ares_core::PresetCatalogHandle;
use serde_json::json;

fn catalog() -> PresetCatalogHandle {
    let mut catalog = PresetCatalogHandle::new();
    for preset in [
        json!({
            "type": "machine", "name": "U1 fixed-slot witness", "from": "user",
            "printer_model": "Snapmaker U1", "printer_technology": "FFF",
            "nozzle_diameter": ["0.4", "0.4", "0.4", "0.4"],
            "single_extruder_multi_material": "0"
        }),
        json!({
            "type": "machine", "name": "SEMM control", "from": "user",
            "printer_technology": "FFF", "nozzle_diameter": ["0.4"],
            "single_extruder_multi_material": "1"
        }),
        json!({"type": "filament", "name": "First PLA", "from": "user", "filament_type": ["PLA"]}),
        json!({"type": "filament", "name": "Last PETG", "from": "user", "filament_type": ["PETG"]}),
    ] {
        let mut preset = preset;
        preset["instantiation"] = json!("true");
        preset["version"] = json!("2.5.0.0");
        let key = if preset["type"] == "machine" {
            "printer_settings_id"
        } else {
            "filament_settings_id"
        };
        preset[key] = preset["name"].clone();
        catalog
            .add_import(preset.to_string().as_bytes(), "witness.json")
            .unwrap();
    }
    catalog
}

#[test]
fn u1_selection_pads_the_last_preset_to_four_slots() {
    let catalog = catalog();
    let selected = vec!["First PLA".to_owned(), "Last PETG".to_owned()];
    let actual = catalog
        .select_compatible_filaments(("U1 fixed-slot witness", None), &selected, &[])
        .unwrap();
    assert_eq!(actual, ["First PLA", "Last PETG", "Last PETG", "Last PETG"]);
    let root = std::env::var_os("ARES_FIXED_FILAMENT_ARTIFACT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("target/fixed-filament-slots"));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("u1-selection.json"),
        serde_json::to_vec_pretty(&json!({"input": selected, "output": actual})).unwrap(),
    )
    .unwrap();
}

#[test]
fn u1_flush_layout_grows_per_nozzle_and_preserves_existing_values() {
    let state = catalog()
        .normalize_filament_slots(
            "U1 fixed-slot witness",
            serde_json::from_value(json!({
                "filaments": ["Last PETG"],
                "flush_multiplier": [1.25],
                "flush_volumes_vector": [100.0, 120.0],
                "flush_volumes_matrix": [7.0],
                "filament_colour": ["#123456"],
                "filament_multi_colour": [""],
                "filament_colour_type": ["1"],
                "filament_map": [2],
                "filament_nozzle_map": [1],
                "filament_volume_map": [0]
            }))
            .unwrap(),
        )
        .unwrap();
    assert_eq!(state.filaments, vec!["Last PETG"; 4]);
    assert_eq!(state.flush_multiplier, [1.25, 1.0, 1.0, 1.0]);
    assert_eq!(
        state.flush_volumes_vector,
        [100.0, 120.0, 100.0, 120.0, 100.0, 120.0, 100.0, 120.0]
    );
    assert_eq!(state.flush_volumes_matrix.len(), 64);
    assert_eq!(state.flush_volumes_matrix[0], 7.0);
    assert_eq!(state.flush_volumes_matrix[1], 220.0);
    assert_eq!(state.flush_volumes_matrix[16], 0.0);
    assert_eq!(state.flush_volumes_matrix[17], 220.0);
    let actual = serde_json::to_value(&state).unwrap();
    if let Some(root) = std::env::var_os("ARES_FIXED_FILAMENT_ARTIFACT_ROOT") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("rust-slot-state.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(actual["filament_colour"], json!(vec!["#123456"; 4]));
    assert_eq!(actual["filament_multi_colour"], json!(vec!["#123456"; 4]));
    assert_eq!(actual["filament_map"], json!([2, 1, 1, 1]));
    assert_eq!(actual["filament_nozzle_map"], json!([1, 0, 0, 0]));
    assert_eq!(actual["filament_volume_map"], json!([0, 0, 0, 0]));
}

#[test]
fn semm_selection_keeps_one_slot() {
    assert_eq!(
        catalog()
            .select_compatible_filaments(("SEMM control", None), &["First PLA".to_owned()], &[])
            .unwrap(),
        ["First PLA"]
    );
}
