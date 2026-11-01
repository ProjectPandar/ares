//! `slice_with_profiles_processed` shares the `record_moves` path with
//! `slice_project_plate_processed` (`slice_source`): on a 3MF whose embedded
//! config already equals the selected profiles (a no-op overlay), both
//! variants must produce identical moves, and the processed G-code must be
//! byte-identical to the non-processed entry.

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

/// A 10 mm cube centered around (118, 138) in XY and grounded in Z.
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
        "nozzle_diameter": [0.4],
        "printable_area": ["0x0", "256x0", "256x256", "0x256"]
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

/// The embedded config carries exactly the selected profiles' own values plus
/// the synthesized identity keys, so the selected-profile overlay is a no-op
/// for the resolved typed settings.
fn embedded_settings() -> String {
    let (catalog, selection) = catalog_parts();
    let mut embedded = json!({});
    for file in catalog["vendors"][0]["files"].as_object().unwrap().values() {
        for (key, value) in file.as_object().unwrap() {
            if key != "type" && key != "name" && key != "from" && key != "instantiation" {
                embedded[key] = value.clone();
            }
        }
    }
    embedded["printer_settings_id"] = selection["machine"].clone();
    embedded["print_settings_id"] = selection["process"].clone();
    embedded["filament_settings_id"] = json!([selection["filaments"][0]]);
    embedded["filament_ids"] = json!([""]);
    embedded["print_compatible_printers"] = json!([]);
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
    overrides: &'a ares_core::ConfigOverrides,
) -> ares_core::ProfileSliceRequest<'a> {
    ares_core::ProfileSliceRequest {
        filename: "cube.3mf",
        catalog_json: catalog,
        selection_json: selection,
        plate: None,
        overrides,
        placement: ares_core::Placement::None,
    }
}

#[tokio::test]
async fn processed_profile_entry_matches_plain_and_project_processed() {
    let (catalog, selection) = catalog_parts();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let input = project_3mf();
    let overrides = ares_core::ConfigOverrides::default();

    let plain = ares_core::slice_with_profiles(
        &input,
        request(&catalog_json, &selection_json, &overrides),
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let processed = ares_core::slice_with_profiles_processed(
        &input,
        request(&catalog_json, &selection_json, &overrides),
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let project_processed = ares_core::slice_project_plate_processed(
        &input,
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();

    // record_moves must not change the exported G-code, and the processed
    // entry shares the slice_project_plate_processed record_moves path: the
    // overlay is a no-op here, so both processed results record identical
    // moves. (The G-code config block may differ where the overlay
    // synthesizes dynamic group keys, so only moves are compared.)
    assert_eq!(plain, processed.gcode, "processed G-code must be unchanged");
    assert!(!processed.result.moves.is_empty(), "cube moves recorded");
    assert_eq!(processed.result, project_processed.result);
}
