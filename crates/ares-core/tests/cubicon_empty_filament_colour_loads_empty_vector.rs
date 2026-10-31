//! O8: Cubicon xCeler presets carry `default_filament_colour: ""` — a bare
//! empty string for a coStrings vector. Upstream's preset and 3MF loaders
//! both deserialize that into an EMPTY vector
//! (`ConfigOptionStrings::deserialize`, Config.hpp:1153-1159 →
//! `unescape_strings_cstyle` Config.cpp:146-149), and the per-filament
//! merge then throws `ConfigOptionVector::set_at(): Assigning from an empty
//! vector` (OrcaSlicer.cpp:3301, Config.hpp:432-441); the native CLI aborts
//! uncaught (matrix SIGABRT 134). Ares returns the same message as
//! `InvalidInput` (accepted deviation).

use ares_core::{Placement, ProfileSliceRequest, SlicerContext, slice_with_profiles};
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
 <object id="2"><part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble>
</config>"#;

const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;

const EMBEDDED_SETTINGS: &str = r#"{
 "printer_settings_id": "Cubicon xCeler-I 0.4 nozzle",
 "print_settings_id": "cubicon default @Cubicon xCeler-I 0.4 nozzle",
 "filament_settings_id": ["Cubicon PLA @Cubicon xCeler-I 0.4 nozzle"],
 "nozzle_diameter": [0.4],
 "printable_area": ["0x0", "250x0", "250x250", "0x250"],
 "printable_height": 250,
 "layer_height": "0.2",
 "filament_diameter": [1.75]
}"#;

/// The real Cubicon vendor files (machine/process/filament chains for all
/// three xCeler printers); the filament @base carries the bare empty
/// `default_filament_colour` that upstream deserializes to an empty vector.
fn cubicon_catalog() -> String {
    let dir = format!(
        "{}/../../tests/parity/validation/empty-filament-colour-cubicon",
        env!("CARGO_MANIFEST_DIR")
    );
    let machines = [
        "machine-xceler-I",
        "machine-xceler-Mini",
        "machine-xceler-Plus",
    ];
    let processes = [
        "process-default-I",
        "process-default-Mini",
        "process-default-Plus",
    ];
    let filaments = ["filament-pla-I", "filament-pla-Mini", "filament-pla-Plus"];
    let mut machine_list = Vec::new();
    let mut process_list = Vec::new();
    let mut filament_list = Vec::new();
    let mut files = serde_json::Map::new();
    let add = |stem: &str, list: &mut Vec<Value>, files: &mut serde_json::Map<String, Value>| {
        let path = format!("{dir}/{stem}.json");
        let value: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        list.push(json!({"sub_path": format!("{stem}.json")}));
        files.insert(format!("{stem}.json"), value);
    };
    for stem in machines {
        add(stem, &mut machine_list, &mut files);
    }
    add("machine-fdm-common", &mut machine_list, &mut files);
    for stem in processes {
        add(stem, &mut process_list, &mut files);
    }
    for stem in [
        "process-common-base",
        "process-common-klipper",
        "process-common",
    ] {
        add(stem, &mut process_list, &mut files);
    }
    for stem in filaments {
        add(stem, &mut filament_list, &mut files);
    }
    for stem in [
        "filament-pla-base",
        "filament-fdm-pla",
        "filament-fdm-common",
    ] {
        add(stem, &mut filament_list, &mut files);
    }
    json!({
        "vendors": [{
            "index": {
                "name": "Cubicon",
                "machine_list": machine_list,
                "process_list": process_list,
                "filament_list": filament_list
            },
            "files": Value::Object(files)
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

/// One xCeler printer's full real chain selects its machine, process, and
/// Cubicon PLA filament; the filament @base's bare empty
/// `default_filament_colour` must surface as the upstream set_at rejection.
async fn slice_xceler(printer: &str, filament_printer: &str) -> Result<Vec<u8>, String> {
    let catalog = cubicon_catalog();
    let selection = json!({
        "machine": format!("Cubicon xCeler-{printer} 0.4 nozzle"),
        "process": format!("cubicon default @Cubicon xCeler-{printer} 0.4 nozzle"),
        "filaments": [format!("Cubicon PLA @Cubicon xCeler-{filament_printer} 0.4 nozzle")]
    })
    .to_string();
    slice_with_profiles(
        project_3mf(),
        ProfileSliceRequest {
            filename: "cube.stl",
            catalog_json: &catalog,
            selection_json: &selection,
            plate: None,
            overrides: &ares_core::ConfigOverrides::default(),
            placement: Placement::Center,
        },
        ares_core::GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .map_err(|error| error.to_string())
}

#[tokio::test]
async fn xceler_i_empty_filament_vector_rejected() {
    let error = slice_xceler("I", "I").await.unwrap_err();
    assert_eq!(
        error.trim(),
        "ConfigOptionVector::set_at(): Assigning from an empty vector",
        "{error}"
    );
}

#[tokio::test]
async fn xceler_mini_empty_filament_vector_rejected() {
    let error = slice_xceler("Mini", "Mini").await.unwrap_err();
    assert_eq!(
        error.trim(),
        "ConfigOptionVector::set_at(): Assigning from an empty vector",
        "{error}"
    );
}

#[tokio::test]
async fn xceler_plus_empty_filament_vector_rejected() {
    let error = slice_xceler("Plus", "Plus").await.unwrap_err();
    assert_eq!(
        error.trim(),
        "ConfigOptionVector::set_at(): Assigning from an empty vector",
        "{error}"
    );
}
