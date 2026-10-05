use super::*;

#[tokio::test]
async fn slice_stl_bytes_returns_gcode_bytes() {
    let output = slice_stl_bytes(square_ascii_stl(), r#"{"layer_height":0.2}"#)
        .await
        .unwrap();
    let gcode = String::from_utf8(output).unwrap();

    assert!(gcode.contains("; input_format = stl"));
    assert!(gcode.contains("; layer_height = 0.2"));
    assert!(gcode.ends_with("M2\n"));
}

#[tokio::test]
async fn slice_stl_bytes_rejects_invalid_options_json() {
    let error = slice_stl_bytes(square_ascii_stl(), "{").await.unwrap_err();

    assert!(error.starts_with("invalid options JSON:"));
}

#[tokio::test]
async fn slice_stl_bytes_rejects_invalid_model_bytes() {
    let error = slice_stl_bytes(Vec::new(), "{}").await.unwrap_err();

    assert_eq!(error, "slice input is empty");
}

fn square_ascii_stl() -> Vec<u8> {
    b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square"
        .to_vec()
}

#[test]
fn preset_catalog_handle_matches_one_shot_filtering() {
    let machine = r#"{"type":"machine","name":"M1","from":"system","instantiation":"true","nozzle_diameter":[0.4]}"#;
    let process = r#"{"type":"process","name":"0.2","from":"system","instantiation":"true","layer_height":"0.2"}"#;
    let filament = r#"{"type":"filament","name":"PLA","from":"system","instantiation":"true","filament_diameter":[1.75]}"#;
    let vendor = serde_json::json!({
        "index": {
            "name": "Acme",
            "machine_list": [{"sub_path": "machine.json"}],
            "process_list": [{"sub_path": "process.json"}],
            "filament_list": [{"sub_path": "filament.json"}]
        },
        "files": {
            "machine.json": serde_json::from_str::<serde_json::Value>(machine).unwrap(),
            "process.json": serde_json::from_str::<serde_json::Value>(process).unwrap(),
            "filament.json": serde_json::from_str::<serde_json::Value>(filament).unwrap()
        }
    });
    let import = serde_json::json!({
        "type": "filament", "name": "User PETG", "from": "User",
        "instantiation": "true", "version": "1.9.0.0",
        "filament_settings_id": "GFPU1"
    });

    let mut catalog = super::PresetCatalog::new();
    catalog
        .add_vendor_json(vendor.to_string().as_bytes())
        .unwrap();
    catalog
        .add_import_json(import.to_string().as_bytes(), "user_petg.json")
        .unwrap();
    let via_handle: serde_json::Value =
        serde_json::from_str(&catalog.filter_compatible_json("M1", None).unwrap()).unwrap();

    let one_shot_catalog = serde_json::json!({
        "vendors": [vendor],
        "imports": [
            {"profiles": [import], "bundle": null}
        ]
    });
    let via_one_shot: serde_json::Value = serde_json::from_str(
        &super::filter_compatible_presets_json(&one_shot_catalog.to_string(), "M1", None).unwrap(),
    )
    .unwrap();

    assert_eq!(via_handle, via_one_shot);
    assert!(
        via_handle["filaments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "User PETG")
    );
}
