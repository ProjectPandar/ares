//! Legacy rule sources must never leak into the exported G-code config
//! block: upstream `handle_legacy` renames or drops them at load
//! (Config.cpp:573-587, PrintConfig.cpp:8033+), so a process preset
//! carrying the legacy `wall_infill_order` resolves into `wall_sequence`
//! (plus `is_infill_first` for infill-first orders) and the legacy key
//! itself never appears. Guards the f5c869df raw admission: legacy keys
//! ride the raw view for the typed resolution but are not config-export
//! keys.

use ares_core::{
    ConfigOverrides, Placement, ProfileSliceRequest, SlicerContext, slice_with_profiles,
};
use serde_json::json;

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
   <vertex x="120" y="120" z="0"/><vertex x="130" y="120" z="0"/><vertex x="120" y="130" z="0"/><vertex x="130" y="130" z="0"/>
   <vertex x="120" y="120" z="10"/><vertex x="130" y="120" z="10"/><vertex x="120" y="130" z="10"/><vertex x="130" y="130" z="10"/>
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
 <object id="2">
  <metadata key="name" value="cube10.stl"/>
  <metadata key="extruder" value="1"/>
  <part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble>
</config>"#;

const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;

const EMBEDDED_SETTINGS: &str = r#"{
 "printer_settings_id": "Acme A1 0.4 nozzle",
 "print_settings_id": "0.20mm Standard @A1",
 "filament_settings_id": ["Acme PLA"],
 "nozzle_diameter": [0.4],
 "printable_area": ["0x0", "256x0", "256x256", "0x256"],
 "printable_height": 256,
 "layer_height": "0.2",
 "filament_diameter": [1.75]
}"#;

fn catalog() -> String {
    let machine = json!({
        "type": "machine", "name": "Acme A1 0.4 nozzle", "from": "system",
        "instantiation": "true", "printer_model": "A1", "printer_variant": "0.4",
        "nozzle_diameter": [0.4], "printable_area": ["0x0", "256x0", "256x256", "0x256"]
    });
    // The process chain carries the legacy wall order (like the real LH
    // vendor's fdm_process_common.json).
    let base = json!({
        "type": "process", "name": "fdm_process_common", "from": "system",
        "instantiation": "false", "wall_infill_order": "inner wall/outer wall/infill"
    });
    let process = json!({
        "type": "process", "name": "0.20mm Standard @A1", "from": "system",
        "instantiation": "true", "inherits": "fdm_process_common", "layer_height": "0.2"
    });
    let filament = json!({
        "type": "filament", "name": "Acme PLA", "from": "system",
        "instantiation": "true", "filament_diameter": [1.75]
    });
    json!({
        "vendors": [{
            "id": "Acme",
            "index": {
                "name": "Acme",
                "machine_list": [{"sub_path": "machine.json"}],
                "process_list": [{"sub_path": "base.json"}, {"sub_path": "process.json"}],
                "filament_list": [{"sub_path": "filament.json"}]
            },
            "files": {
                "machine.json": machine, "base.json": base, "process.json": process,
                "filament.json": filament
            }
        }],
        "imports": []
    })
    .to_string()
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
                EMBEDDED_SETTINGS.as_bytes(),
            ),
        ] {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

#[tokio::test]
async fn legacy_wall_infill_order_never_reaches_config_block() {
    let gcode = slice_with_profiles(
        project_3mf(),
        ProfileSliceRequest {
            filename: "cube.stl",
            catalog_json: &catalog(),
            selection_json: &json!({
                "machine": "Acme A1 0.4 nozzle",
                "process": "0.20mm Standard @A1",
                "filaments": ["Acme PLA"]
            })
            .to_string(),
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::Center,
        },
        ares_core::GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    let text = String::from_utf8(gcode).unwrap();
    assert!(
        !text
            .lines()
            .any(|line| line.starts_with("; wall_infill_order")),
        "legacy key leaked into the config block"
    );
    // The renamed canonical order does export.
    assert!(
        text.lines()
            .any(|line| line == "; wall_sequence = inner wall/outer wall")
    );
}
