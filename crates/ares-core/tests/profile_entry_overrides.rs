//! M854 user option overrides through the profile entry: overrides apply
//! last (GUI edit order, `Config.cpp:461 apply_only`) over the selected
//! profiles on both the STL and 3MF entries.

use ares_core::{ConfigOverrides, ProfileSliceRequest, slice_with_profiles};
use serde_json::{Value, json};

fn catalog_parts() -> (Value, Value) {
    let machine = json!({
        "type": "machine", "name": "Acme A1 0.4 nozzle", "from": "system",
        "instantiation": "true", "printer_model": "A1", "printer_variant": "0.4",
        "nozzle_diameter": [0.4], "printable_area": ["0x0", "256x0", "256x256", "0x256"]
    });
    let process = json!({
        "type": "process", "name": "0.20mm Standard @A1", "from": "system",
        "instantiation": "true", "layer_height": "0.2"
    });
    let filament = json!({
        "type": "filament", "name": "Acme PLA", "from": "system",
        "instantiation": "true", "filament_diameter": [1.75]
    });
    let vendor = json!({
        "id": "Acme",
        "index": {
            "name": "Acme",
            "machine_list": [{"sub_path": "machine.json"}],
            "process_list": [{"sub_path": "process.json"}],
            "filament_list": [{"sub_path": "filament.json"}]
        },
        "files": {
            "machine.json": machine,
            "process.json": process,
            "filament.json": filament
        }
    });
    let catalog = json!({"vendors": [vendor], "imports": []});
    let selection = json!({
        "machine": "Acme A1 0.4 nozzle",
        "process": "0.20mm Standard @A1",
        "filaments": ["Acme PLA"]
    });
    (catalog, selection)
}

fn stl() -> Vec<u8> {
    b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square".to_vec()
}

fn request<'a>(
    catalog: &'a str,
    selection: &'a str,
    overrides: &'a ConfigOverrides,
) -> ProfileSliceRequest<'a> {
    ProfileSliceRequest {
        filename: "cube.stl",
        catalog_json: catalog,
        selection_json: selection,
        plate: None,
        overrides,
        // Center the raw STL on the bed: the unconditional volume gate
        // rejects partially-outside instances (upstream --arrange equivalent).
        placement: ares_core::Placement::Center,
    }
}

#[tokio::test]
async fn composed_vendor_profile_preserves_omitted_filament_gcode_defaults() {
    let (_, selection) = catalog_parts();
    let catalog_json = include_str!("../../../tests/profiles/filament-gcode-slots/catalog.json");
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let settings =
        ares_core::profile_settings(stl(), request(catalog_json, &selection_json, &overrides))
            .await
            .unwrap();
    let settings = serde_json::to_value(settings).unwrap();
    write_artifact(
        "vendor-omitted.json",
        &serde_json::to_vec_pretty(&settings).unwrap(),
    );
    assert_eq!(settings["filament_end_gcode"], json!([" "]));
    assert_eq!(settings["filament_start_gcode"], json!([" "]));
}

#[tokio::test]
async fn imported_sparse_profile_pads_empty_filament_gcode_vectors() {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../tests/profiles/filament-gcode-slots/catalog.json"
    ))
    .unwrap();
    let process: Value = serde_json::from_str(include_str!(
        "../../../tests/profiles/filament-gcode-slots/cli-process.json"
    ))
    .unwrap();
    let filament: Value = serde_json::from_str(include_str!(
        "../../../tests/profiles/filament-gcode-slots/cli-filament.json"
    ))
    .unwrap();
    let imported_catalog = json!({
        "vendors": [],
        "imports": [{"profiles": [catalog["vendors"][0]["files"]["machine.json"], process, filament]}]
    });
    let (_, selection) = catalog_parts();
    let catalog_json = imported_catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let settings =
        ares_core::profile_settings(stl(), request(&catalog_json, &selection_json, &overrides))
            .await
            .unwrap();
    let settings = serde_json::to_value(settings).unwrap();
    write_artifact(
        "imported-empty.json",
        &serde_json::to_vec_pretty(&settings).unwrap(),
    );
    assert_eq!(settings["filament_end_gcode"], json!([""]));
    assert_eq!(settings["filament_start_gcode"], json!([""]));
}

#[tokio::test]
async fn vendor_profile_omitted_filament_gcodes_keep_registered_space_defaults() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let settings =
        ares_core::profile_settings(stl(), request(&catalog_json, &selection_json, &overrides))
            .await
            .unwrap();
    let settings = serde_json::to_value(settings).unwrap();
    assert_eq!(settings["filament_end_gcode"], json!([" "]));
    assert_eq!(settings["filament_start_gcode"], json!([" "]));
}

#[tokio::test]
async fn vendor_profile_empty_filament_gcode_vectors_become_one_empty_slot() {
    let (mut catalog, selection) = catalog_parts();
    let filament = &mut catalog["vendors"][0]["files"]["filament.json"];
    filament["filament_end_gcode"] = json!([]);
    filament["filament_start_gcode"] = json!([]);
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let settings =
        ares_core::profile_settings(stl(), request(&catalog_json, &selection_json, &overrides))
            .await
            .unwrap();
    let settings = serde_json::to_value(settings).unwrap();
    assert_eq!(settings["filament_end_gcode"], json!([""]));
    assert_eq!(settings["filament_start_gcode"], json!([""]));
}

fn write_artifact(name: &str, bytes: &[u8]) {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    std::fs::create_dir_all(root).unwrap();
    let path = root.join(format!(
        "filament-gcode-slots-{}-{name}",
        std::process::id()
    ));
    std::fs::write(&path, bytes).unwrap();
    eprintln!("artifact: {}", path.display());
}

#[tokio::test]
async fn overrides_win_over_preset_values() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let plain = slice_with_profiles(
        stl(),
        request(&catalog_json, &selection_json, &ConfigOverrides::default()),
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let overridden = slice_with_profiles(
        stl(),
        request(
            &catalog_json,
            &selection_json,
            &ConfigOverrides::from_json(br#"{"layer_height":"0.3"}"#).unwrap(),
        ),
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();

    let plain = String::from_utf8(plain).unwrap();
    let overridden = String::from_utf8(overridden).unwrap();
    assert!(plain.contains("; layer_height = 0.2"), "{plain}");
    assert!(
        overridden.contains("; layer_height = 0.3"),
        "override must win over the preset value: {overridden}"
    );
    assert_ne!(plain, overridden, "the slice itself must change");
}

#[tokio::test]
async fn typed_sparse_cli_empty_filament_gcodes_complete_slicing() {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../tests/profiles/filament-gcode-slots/catalog.json"
    ))
    .unwrap();
    let machine = catalog["vendors"][0]["files"]["machine.json"].to_string();
    let process = include_str!("../../../tests/profiles/filament-gcode-slots/cli-process.json");
    let filament = include_str!("../../../tests/profiles/filament-gcode-slots/cli-filament.json");
    write_artifact(
        "runtime-inputs.json",
        &serde_json::to_vec_pretty(&json!({
            "machine": serde_json::from_str::<Value>(&machine).unwrap(),
            "process": serde_json::from_str::<Value>(process).unwrap(),
            "filament": serde_json::from_str::<Value>(filament).unwrap(),
            "mesh": include_str!("../../../tests/profiles/filament-gcode-slots/sparse-runtime.stl")
        }))
        .unwrap(),
    );
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let gcode = ares_core::slice_with_load_settings(
        include_bytes!("../../../tests/profiles/filament-gcode-slots/sparse-runtime.stl").to_vec(),
        ares_core::LoadSettingsSliceRequest {
            orient: false,
            filename: "cube.stl",
            catalog_json: None,
            uptodate: false,
            settings: &[
                ares_core::LoadSettingsFile {
                    label: "machine.json",
                    json: &machine,
                },
                ares_core::LoadSettingsFile {
                    label: "process.json",
                    json: process,
                },
            ],
            filaments: &[ares_core::LoadSettingsFile {
                label: "filament.json",
                json: filament,
            }],
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: ares_core::Placement::Center,
            sliced_plate: &mut sliced_plate,
        },
        ares_core::GenerationMetadata::deterministic(2026, 11, 13, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    write_artifact("runtime-empty.gcode", &gcode);
    let gcode = String::from_utf8(gcode).unwrap();
    for expected in [
        "; filament_start_gcode = \"\"",
        "; filament_end_gcode = \"\"",
        "; single_extruder_multi_material = 1",
    ] {
        assert!(
            gcode.lines().any(|line| line == expected),
            "missing configuration line: {expected}"
        );
    }
}
