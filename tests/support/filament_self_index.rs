use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
    path::Path,
};

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

pub fn inputs() -> (Vec<u8>, String, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OrcaSlicer/resources/profiles");
    let vendors: Vec<Value> = ["OrcaFilamentLibrary", "BBL", "Snapmaker"]
        .into_iter()
        .map(|id| {
            let index: Value =
                serde_json::from_slice(&std::fs::read(root.join(format!("{id}.json"))).unwrap())
                    .unwrap();
            let mut files = BTreeMap::new();
            for kind in [
                "machine_model_list",
                "machine_list",
                "process_list",
                "filament_list",
            ] {
                for entry in index[kind].as_array().into_iter().flatten() {
                    let path = entry["sub_path"].as_str().unwrap();
                    let value: Value =
                        serde_json::from_slice(&std::fs::read(root.join(id).join(path)).unwrap())
                            .unwrap();
                    files.insert(path.to_owned(), value);
                }
            }
            json!({"id":id,"index":index,"files":files})
        })
        .collect();
    let catalog = json!({"vendors":vendors,"imports":[]}).to_string();
    let selection = json!({
        "machine":"Snapmaker U1 (0.4 nozzle)",
        "process":"0.20 Standard @Snapmaker U1 (0.4 nozzle)",
        "filaments": vec!["Generic PLA @U1 0.4 nozzle"; 4]
    })
    .to_string();
    let settings = json!({
        "printer_settings_id":"Bambu Lab H2D 0.4 nozzle",
        "printer_model":"H2D",
        "nozzle_diameter":["0.4","0.4"],
        "filament_settings_id":vec!["Synthetic PLA"; 4],
        "filament_diameter":vec!["1.75"; 4],
        "filament_colour":vec!["#F2754E"; 4],
        "filament_self_index":["1","1","2","2","3","3","4","4"],
        "filament_extruder_variant":[
            "Direct Drive Standard","Direct Drive High Flow",
            "Direct Drive Standard","Direct Drive High Flow",
            "Direct Drive Standard","Direct Drive High Flow",
            "Direct Drive Standard","Direct Drive High Flow"
        ]
    })
    .to_string();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", ROOT_RELATIONSHIPS),
        ("3D/root.model", ROOT_MODEL),
        ("Metadata/model_settings.config", MODEL_SETTINGS),
        ("Metadata/slice_info.config", SLICE_INFO),
        ("Metadata/filament_sequence.json", FILAMENT_SEQUENCE),
        ("Metadata/project_settings.config", &settings),
    ] {
        writer
            .start_file(path, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(bytes.as_bytes()).unwrap();
    }
    (writer.finish().unwrap().into_inner(), catalog, selection)
}
