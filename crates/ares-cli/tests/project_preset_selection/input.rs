use std::io::{Cursor, Write};

use serde_json::{Value, json};

const ROOT_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
 <metadata name="Application">OrcaSlicer-2.5.0</metadata>
 <resources><object id="2" type="model"><mesh><vertices>
  <vertex x="113" y="133" z="0"/><vertex x="123" y="133" z="0"/><vertex x="113" y="143" z="0"/><vertex x="123" y="143" z="0"/>
  <vertex x="113" y="133" z="10"/><vertex x="123" y="133" z="10"/><vertex x="113" y="143" z="10"/><vertex x="123" y="143" z="10"/>
 </vertices><triangles>
  <triangle v1="0" v2="1" v3="2"/><triangle v1="1" v2="3" v3="2"/>
  <triangle v1="4" v2="6" v3="5"/><triangle v1="5" v2="6" v3="7"/>
  <triangle v1="0" v2="4" v3="1"/><triangle v1="1" v2="4" v3="5"/>
  <triangle v1="2" v2="3" v3="6"/><triangle v1="3" v2="7" v3="6"/>
  <triangle v1="0" v2="2" v3="4"/><triangle v1="2" v2="6" v3="4"/>
  <triangle v1="1" v2="5" v3="3"/><triangle v1="3" v2="5" v3="7"/>
 </triangles></mesh></object></resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1"/></build>
</model>"#;

const MODEL_SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
 <object id="2"><metadata key="name" value="Synthetic cube"/><metadata key="wall_loops" value="4"/><part id="1" subtype="normal_part"><metadata key="extruder" value="1"/><metadata key="sparse_infill_density" value="23%"/><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble>
</config>"#;

pub fn settings(original: &str, layer: &str) -> Value {
    json!({
        "print_settings_id": original,
        "printer_settings_id": "Missing Printer",
        "filament_settings_id": ["Missing Filament"],
        "filament_ids": ["fixture-filament"],
        "filament_colour": ["#AABBCC"],
        "filament_diameter": ["1.75"],
        "layer_height": layer,
        "nozzle_diameter": ["0.6"],
        "printable_area": ["0x0", "200x0", "200x200", "0x200"]
    })
}

pub fn process(name: &str, layer: &str, origin: &str) -> Value {
    json!({"type": "process", "name": name, "from": origin, "instantiation": "true", "version": "02.05.00.00", "layer_height": layer})
}

pub fn catalog(system: &[Value], users: &[Value]) -> Value {
    let mut index = json!({
        "name": "Acme", "version": "02.05.00.00", "machine_model_list": [],
        "machine_list": [], "process_list": [], "filament_list": []
    });
    let mut files = serde_json::Map::new();
    for (ordinal, profile) in system.iter().enumerate() {
        let kind = profile["type"].as_str().unwrap();
        let filename = format!("{kind}/{ordinal}.json");
        index[format!("{kind}_list")]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": profile["name"], "sub_path": filename}));
        files.insert(filename, profile.clone());
    }
    let vendors = if system.is_empty() {
        json!([])
    } else {
        json!([{"id": "Acme", "index": index, "files": files}])
    };
    json!({"vendors": vendors, "imports": [{"profiles": users}]})
}

pub fn project(settings: &Value) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut cursor);
        let settings = settings.to_string();
        for (name, content) in [
            (
                "[Content_Types].xml",
                r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/><Default Extension="png" ContentType="image/png"/></Types>"#,
            ),
            (
                "_rels/.rels",
                r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/root.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#,
            ),
            ("3D/root.model", ROOT_MODEL),
            ("Metadata/model_settings.config", MODEL_SETTINGS),
            (
                "Metadata/slice_info.config",
                r#"<config><header><header_item key="OrcaSlicer-Version" value="2.5.0"/></header></config>"#,
            ),
            (
                "Metadata/filament_sequence.json",
                r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#,
            ),
            ("Metadata/project_settings.config", settings.as_str()),
        ] {
            writer
                .start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(content.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
    }
    cursor.into_inner()
}
