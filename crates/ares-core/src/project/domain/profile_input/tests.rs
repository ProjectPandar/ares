use std::io::Write as _;

use super::*;
use crate::ProjectSettings;
use crate::project::raw_settings::RawConfigValue;

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
  <object id="2" type="model"><components><component p:path="/3D/leaf.model" objectid="1" transform="1 0 0 0 1 0 0 0 1 0 0 0"/></components></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 10 20 30" printable="1" auto_drop="1"/></build>
</model>"#;

const LEAF_MODEL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <resources><object id="1" type="model"><mesh><vertices>
  <vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/>
 </vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object></resources>
 <build/>
</model>"#;

const MODEL_RELATIONSHIPS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/leaf.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

const MODEL_SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
 <object id="2"><part id="1" subtype="normal_part"><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance></plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 30" offset="0 0 0"/></assemble>
</config>"#;

const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;

fn archive(settings_json: &str) -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for (path, bytes) in [
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
            ("3D/root.model", ROOT_MODEL.as_bytes()),
            ("3D/_rels/root.model.rels", MODEL_RELATIONSHIPS.as_bytes()),
            ("3D/leaf.model", LEAF_MODEL.as_bytes()),
            ("Metadata/model_settings.config", MODEL_SETTINGS.as_bytes()),
            ("Metadata/slice_info.config", SLICE_INFO.as_bytes()),
            (
                "Metadata/filament_sequence.json",
                FILAMENT_SEQUENCE.as_bytes(),
            ),
            ("Metadata/project_settings.config", settings_json.as_bytes()),
        ] {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

fn composed(settings_json: &str) -> ProjectSettings {
    serde_json::from_str(settings_json).unwrap()
}

fn raw(settings_json: &str) -> ProjectSettingsRaw {
    serde_json::from_str(settings_json).unwrap()
}

fn text(value: Option<&RawConfigValue>, index: usize) -> String {
    match value.and_then(|value| value.element(index)) {
        Some(RawConfigValue::Text(text)) => text.clone(),
        Some(RawConfigValue::Number(number)) => number.to_string(),
        other => panic!("expected text at {index}, got {other:?}"),
    }
}

fn overlay(
    settings_json: &str,
    profile_settings_json: &str,
    profile_json: &str,
    filament_jsons: &[&str],
) -> Result<Project, SliceError> {
    let composed = composed(profile_settings_json);
    let profile_raw = raw(profile_json);
    let filament_raws = filament_jsons
        .iter()
        .map(|json| raw(json))
        .collect::<Vec<_>>();
    Project::from_3mf_with_profiles(
        &archive(settings_json),
        SelectedProfile {
            settings: &composed,
            profile_raw: &profile_raw,
            filament_raws: &filament_raws,
            validate_print: false,
        },
        &crate::ConfigOverrides::default(),
        None,
        &crate::plater::NO_EDITS,
    )
    .map(|(project, _)| project)
}

#[test]
fn overlay_preserves_embedded_keys_and_filament_tail() {
    // OrcaSlicer.cpp:2750-2812: keys the presets do not supply keep the
    // embedded value; filament slots beyond the selection keep theirs
    // (:3301 set_at writes only the loaded indices).
    let settings = r#"{
            "printer_settings_id": "Embedded Machine",
            "print_settings_id": "Embedded Process",
            "filament_settings_id": ["Embedded Filament", "Second Filament"],
            "filament_diameter": [3.0, 2.85],
            "layer_height": "0.32"
        }"#;
    let profile = r#"{
            "filament_diameter": [1.75],
            "layer_height": "0.2"
        }"#;
    let filament = r#"{"filament_diameter": [1.75]}"#;
    let selection = r#"{
            "printer_settings_id": "Embedded Machine",
            "print_settings_id": "Embedded Process",
            "filament_settings_id": ["Embedded Filament"]
        }"#;
    let project = overlay(settings, selection, profile, &[filament]).unwrap();
    let merged = project.documents().project_settings_raw.clone();

    assert_eq!(
        text(merged.get("filament_diameter"), 0),
        "1.75",
        "supplied slot takes the selected filament value"
    );
    assert_eq!(
        text(merged.get("filament_diameter"), 1),
        "2.85",
        "unselected slot keeps the embedded value"
    );
    assert_eq!(
        text(merged.get("layer_height"), 0),
        "0.2",
        "process key supplied by the preset is overlaid"
    );
    assert_eq!(
        project.settings().project.preset.printer_settings_id.0,
        "Embedded Machine"
    );
}

#[test]
fn overlay_rejects_more_selected_filaments_than_embedded() {
    let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["Only One"]
        }"#;
    let selection = r#"{
            "filament_settings_id": ["A", "B"]
        }"#;
    let error = overlay(settings, selection, "{}", &["{}", "{}"]).unwrap_err();
    assert!(
        error.to_string().contains("selection has 2 filaments"),
        "{error}"
    );
}

#[test]
fn overlay_print_compatible_printers_follows_process_load() {
    // OrcaSlicer.cpp:2980-2984: the loaded process's compatible_printers
    // replaces print_compatible_printers even when it is absent or empty;
    // with no process loaded the embedded list is retained — the file
    // loop's arms run per loaded file only (:2009-2106).
    let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["F"],
            "print_compatible_printers": ["Embedded Machine"]
        }"#;
    let loaded = overlay(
        settings,
        "{\"printer_settings_id\":\"M\",\"print_settings_id\":\"P\",\"filament_settings_id\":[\"F\"]}",
        "{}",
        &["{}"],
    )
    .unwrap();
    let merged = loaded.documents().project_settings_raw.clone();
    assert!(merged.get("print_compatible_printers").unwrap().len() == 0);
    assert!(
        loaded
            .settings()
            .project
            .preset
            .print_compatible_printers
            .0
            .is_empty()
    );
    let not_loaded = overlay(
        settings,
        "{\"filament_settings_id\":[\"F\"]}",
        "{}",
        &["{}"],
    )
    .unwrap();
    let merged = not_loaded.documents().project_settings_raw.clone();
    assert_eq!(
        text(merged.get("print_compatible_printers"), 0),
        "Embedded Machine"
    );
}

#[test]
fn single_preset_mapping_disables_prime_tower_and_records_the_key() {
    // OrcaSlicer.cpp:2275,3680-3699.
    let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["Same", "Same"],
            "enable_prime_tower": "1"
        }"#;
    let project = overlay(
        settings,
        "{\"filament_settings_id\":[\"Same\",\"Same\"]}",
        "{}",
        &["{}", "{}"],
    )
    .unwrap();
    let merged = project.documents().project_settings_raw.clone();
    assert_eq!(text(merged.get("enable_prime_tower"), 0), "0");
    assert_eq!(
        text(merged.get("different_settings_to_system"), 0),
        "enable_prime_tower"
    );
}

#[test]
fn smooth_timelapse_keeps_the_prime_tower() {
    let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["Same"],
            "enable_prime_tower": "1",
            "enable_timelapse": "1",
            "timelapse_type": "1"
        }"#;
    let project = overlay(
        settings,
        "{\"filament_settings_id\":[\"Same\"]}",
        "{}",
        &["{}"],
    )
    .unwrap();
    let merged = project.documents().project_settings_raw.clone();
    assert_eq!(text(merged.get("enable_prime_tower"), 0), "1");
    assert_eq!(text(merged.get("different_settings_to_system"), 0), "");
}

#[test]
fn distinct_filaments_keep_the_prime_tower() {
    let settings = r#"{
            "printer_settings_id": "M", "print_settings_id": "P",
            "filament_settings_id": ["A", "B"],
            "enable_prime_tower": "1"
        }"#;
    let project = overlay(
        settings,
        "{\"filament_settings_id\":[\"A\",\"B\"]}",
        "{}",
        &["{}", "{}"],
    )
    .unwrap();
    let merged = project.documents().project_settings_raw.clone();
    assert_eq!(text(merged.get("enable_prime_tower"), 0), "1");
    assert_eq!(text(merged.get("different_settings_to_system"), 0), "");
}
