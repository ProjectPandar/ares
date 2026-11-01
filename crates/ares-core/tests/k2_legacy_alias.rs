//! K2: legacy feature-filament aliases coexisting with their new keys.
//! Upstream renames the legacy key in place (`PrintConfigDef::handle_legacy`,
//! PrintConfig.cpp:8053-8080) and `set_deserialize_raw` overwrites on the
//! second assignment (Config.cpp:573-587) — there is no duplicate concept;
//! with `nlohmann::json`'s sorted-map iteration (Config.cpp:822,874+) the
//! lexicographically greater source key is applied last and wins. Five real
//! Creality process presets ship both keys (K2 ×3, Ender-5 Max ×2); the
//! native loads and slices them (verified, exit 0). Precedence probe
//! (differing values through the real chain): legacy 3 beats new 2
//! (`internal_solid_filament_id` < `solid_infill_filament`), new 5 beats
//! legacy 4 (`sparse_infill_filament` < `sparse_infill_filament_id`),
//! legacy 6 beats new 7 (`outer_wall_filament_id` < `wall_filament`).

use ares_core::{
    ConfigOverrides, Placement, ProfileSliceRequest, SlicerContext, slice_with_profiles,
};
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
   <vertex x="170" y="170" z="0"/><vertex x="180" y="170" z="0"/><vertex x="170" y="180" z="0"/><vertex x="180" y="180" z="0"/>
   <vertex x="170" y="170" z="10"/><vertex x="180" y="170" z="10"/><vertex x="170" y="180" z="10"/><vertex x="180" y="180" z="10"/>
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

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../tests/parity/validation/e5max-legacy/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// The real Creality Ender-5 Max 0.4 chains; `process.json` carries all
/// three legacy/new alias pairs (legacy "1", new "0").
fn catalog(process_override: Option<&[(&str, &str)]>) -> String {
    let dir = format!(
        "{}/../../tests/parity/validation/e5max-legacy",
        env!("CARGO_MANIFEST_DIR")
    );
    let load = |stem: &str| -> Value {
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/{stem}.json")).unwrap())
            .unwrap()
    };
    let mut machine_list = Vec::new();
    let mut process_list = Vec::new();
    let mut filament_list = Vec::new();
    let mut files = serde_json::Map::new();
    let add = |stem: &str, list: &mut Vec<Value>, files: &mut serde_json::Map<String, Value>| {
        files.insert(format!("{stem}.json"), load(stem));
        list.push(json!({"sub_path": format!("{stem}.json")}));
    };
    add("machine", &mut machine_list, &mut files);
    add("machine-common", &mut machine_list, &mut files);
    add("machine-fdm", &mut machine_list, &mut files);
    add("process", &mut process_list, &mut files);
    add("process-klipper", &mut process_list, &mut files);
    add("process-creality", &mut process_list, &mut files);
    add("process-common", &mut process_list, &mut files);
    add("filament", &mut filament_list, &mut files);
    add("filament-base", &mut filament_list, &mut files);
    if let Some(overrides) = process_override {
        let process = files.get_mut("process.json").unwrap();
        for (key, value) in overrides {
            process[*key] = json!(value);
        }
    }
    json!({
        "vendors": [{
            "id": "Creality",
            "index": {
                "name": "Creality",
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

const MACHINE: &str = "Creality Ender-5 Max 0.4 nozzle";
const PROCESS: &str = "0.20mm Standard @Creality Ender-5 Max 0.4mm nozzle";
const FILAMENT: &str = "Creality Hyper PLA @Ender-5Max-all";

fn selection() -> String {
    json!({"machine": MACHINE, "process": PROCESS, "filaments": [FILAMENT]}).to_string()
}

/// A 3MF whose embedded settings are the native 2.4.2 export of the same
/// selection (post-alias resolution, 643 keys).
fn project_3mf() -> Vec<u8> {
    let embedded = fixture("embedded-settings.json");
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
            ("Metadata/project_settings.config", embedded.as_bytes()),
        ] {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

/// The real both-keys process must load and slice; the alias pair resolves
/// to the value the native 2.4.2 carries in its exported config for the
/// same selection (legacy "1" maps to canonical "0"). Full byte-parity
/// is not asserted here: this Creality klipper process also triggers an
/// unrelated pre-existing Ares gap (missing per-feature
/// `SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY` lines) that reproduces on
/// the pure embedded path without any alias involvement.
#[tokio::test]
async fn both_keys_process_loads_and_slices_like_native() {
    let gcode = slice_with_profiles(
        project_3mf(),
        ProfileSliceRequest {
            filename: "cube.stl",
            catalog_json: &catalog(None),
            selection_json: &selection(),
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
    let resolved = text
        .lines()
        .find(|line| line.starts_with("; internal_solid_filament_id"))
        .unwrap_or_default();
    assert_eq!(resolved, "; internal_solid_filament_id = 0", "{text:.600}");
}

/// Differing values, native-verified precedence: `solid_infill_filament` is
/// lexicographically greater than `internal_solid_filament_id`, so the
/// legacy value applies last and wins. The resolved value is observable in
/// the emitted G-code config block.
#[tokio::test]
async fn legacy_alias_wins_when_lexicographically_later() {
    let overrides = [
        ("solid_infill_filament", "3"),
        ("internal_solid_filament_id", "2"),
        ("wall_filament", "6"),
        ("outer_wall_filament_id", "7"),
    ];
    let gcode = slice_with_profiles(
        project_3mf(),
        ProfileSliceRequest {
            filename: "cube.stl",
            catalog_json: &catalog(Some(&overrides)),
            selection_json: &selection(),
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
    for (key, value) in [
        ("internal_solid_filament_id", "3"),
        ("outer_wall_filament_id", "6"),
    ] {
        let resolved = text
            .lines()
            .find(|line| line.starts_with(&format!("; {key}")))
            .unwrap_or_default();
        assert_eq!(resolved, format!("; {key} = {value}"));
    }
}

/// The reverse order: `sparse_infill_filament` sorts before
/// `sparse_infill_filament_id`, so the new key applies last and wins.
#[tokio::test]
async fn new_key_wins_when_lexicographically_later() {
    let overrides = [
        ("sparse_infill_filament", "4"),
        ("sparse_infill_filament_id", "5"),
    ];
    let gcode = slice_with_profiles(
        project_3mf(),
        ProfileSliceRequest {
            filename: "cube.stl",
            catalog_json: &catalog(Some(&overrides)),
            selection_json: &selection(),
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
    let resolved = text
        .lines()
        .find(|line| line.starts_with("; sparse_infill_filament_id"))
        .unwrap_or_default();
    assert_eq!(resolved, "; sparse_infill_filament_id = 5");
}
