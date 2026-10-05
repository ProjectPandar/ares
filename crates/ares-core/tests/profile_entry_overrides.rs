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
    }
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
