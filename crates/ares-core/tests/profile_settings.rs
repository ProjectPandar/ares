//! `profile_settings`: the effective config a selected-profile slice uses,
//! exposed for the web Process panel (upstream `PresetBundle::full_config` /
//! the GUI Tab's edited preset view). The returned view is the same merged
//! raw the slice pipeline consumes — no second synthesis.

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest, profile_settings};
use serde_json::{Value, json};

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
 <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
 <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
 <Default Extension="png" ContentType="image/png"/>
</Types>"#;

const ROOT_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/root.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

const ROOT_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <metadata name="OrcaSlicer">2.4.2</metadata>
 <resources>
  <object id="2" type="model"><mesh><vertices>
   <vertex x="113" y="133" z="0"/><vertex x="123" y="133" z="0"/><vertex x="113" y="143" z="0"/><vertex x="123" y="143" z="0"/>
   <vertex x="113" y="133" z="10"/><vertex x="123" y="133" z="10"/><vertex x="113" y="143" z="10"/><vertex x="123" y="143" z="10"/>
  </vertices><triangles>
   <triangle v1="0" v2="1" v3="2"/><triangle v1="1" v2="3" v3="2"/>
   <triangle v1="4" v2="6" v3="5"/><triangle v1="5" v2="6" v3="7"/>
   <triangle v1="0" v2="4" v3="1"/><triangle v1="1" v2="4" v3="5"/>
   <triangle v1="2" v2="3" v3="6"/><triangle v1="3" v2="7" v3="6"/>
   <triangle v1="0" v2="2" v3="4"/><triangle v1="2" v2="6" v3="4"/>
   <triangle v1="1" v2="5" v3="3"/><triangle v1="3" v2="5" v3="7"/>
  </triangles></mesh></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build>
</model>"#;

const MODEL_SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
 <object id="2"><part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble>
</config>"#;

const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;

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

fn embedded_settings() -> String {
    let (_catalog, _selection) = catalog_parts();
    let mut embedded = json!({
        "printer_settings_id": "Old Machine",
        "print_settings_id": "Old Process",
        "filament_settings_id": ["Old Filament"],
        "layer_height": "0.28",
        "filament_diameter": [3.0],
        "nozzle_diameter": [0.6],
        "printable_area": ["0x0", "200x0", "200x200", "0x200"]
    });
    // Embed the machine preset's own area values so only the preset-supplied
    // keys differ from the selection overlay below.
    embedded["printable_area"] = json!(["0x0", "256x0", "256x256", "0x256"]);
    embedded.to_string()
}

fn project_3mf() -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        use std::io::Write as _;
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for (path, bytes) in [
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
            ("3D/root.model", ROOT_MODEL.as_bytes()),
            ("Metadata/model_settings.config", MODEL_SETTINGS.as_bytes()),
            ("Metadata/slice_info.config", SLICE_INFO.as_bytes()),
            (
                "Metadata/filament_sequence.json",
                FILAMENT_SEQUENCE.as_bytes(),
            ),
            (
                "Metadata/project_settings.config",
                embedded_settings().as_bytes(),
            ),
        ] {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

fn request<'a>(
    catalog: &'a str,
    selection: &'a str,
    overrides: &'a ConfigOverrides,
) -> ProfileSliceRequest<'a> {
    ProfileSliceRequest {
        filename: "cube.3mf",
        catalog_json: catalog,
        selection_json: selection,
        plate: None,
        overrides,
        placement: Placement::None,
    }
}

#[tokio::test]
async fn profile_settings_returns_the_merged_view_slicing_uses() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::default();
    let input = project_3mf();

    let settings = profile_settings(&input, request(&catalog_json, &selection_json, &overrides))
        .await
        .unwrap();
    let view: Value = serde_json::to_value(&settings).unwrap();

    // Preset-supplied keys win over the embedded config.
    assert_eq!(view["layer_height"], json!("0.2"), "{view}");
    assert_eq!(view["filament_diameter"], json!([1.75]), "{view}");
    assert_eq!(view["nozzle_diameter"], json!([0.4]), "{view}");
    // Identity keys carry the selection names.
    assert_eq!(view["printer_settings_id"], json!("Acme A1 0.4 nozzle"));
    assert_eq!(view["print_settings_id"], json!("0.20mm Standard @A1"));
    assert_eq!(view["filament_settings_id"], json!(["Acme PLA"]));
}

#[tokio::test]
async fn profile_settings_applies_user_overrides_last() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let overrides = ConfigOverrides::from_json(br#"{"layer_height":"0.3"}"#).unwrap();
    let settings = profile_settings(
        &project_3mf(),
        request(&catalog_json, &selection_json, &overrides),
    )
    .await
    .unwrap();
    let view: Value = serde_json::to_value(&settings).unwrap();
    assert_eq!(view["layer_height"], json!("0.3"), "{view}");
}
