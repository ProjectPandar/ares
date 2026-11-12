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
fn semm_selection_keeps_one_slot() {
    assert_eq!(
        catalog()
            .select_compatible_filaments(("SEMM control", None), &["First PLA".to_owned()], &[])
            .unwrap(),
        ["First PLA"]
    );
}
