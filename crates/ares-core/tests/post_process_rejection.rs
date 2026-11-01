//! O5: `post_process` rejection at the CLI-equivalent boundary
//! (`OrcaSlicer.cpp:131,1682-1690`). The check fires only for 3MF inputs —
//! the embedded `post_process` array being non-empty (including the
//! single-empty-string form `[""]` from Prusa `process_common_mk3.json`)
//! is rejected with the exact upstream message. STL inputs are never
//! checked (verified against the native 2.4.2 binary).

use ares_core::{
    ConfigOverrides, Placement, ProfileSliceRequest, slice_project_plate_validated,
    slice_with_profiles_validated,
};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// 3MF archive builder (same structure as the profile_settings tests).
// ---------------------------------------------------------------------------

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

fn base_settings(post_process: Option<Value>) -> String {
    let mut settings = json!({
        "printer_settings_id": "Acme A1 0.4 nozzle",
        "print_settings_id": "0.20mm Standard @A1",
        "filament_settings_id": ["Acme PLA"],
        "nozzle_diameter": [0.4],
        "printable_area": ["0x0", "256x0", "256x256", "0x256"],
        "layer_change_gcode": "G92 E0"
    });
    if let Some(value) = post_process {
        settings["post_process"] = value;
    }
    settings.to_string()
}

fn project_3mf(post_process: Option<Value>) -> Vec<u8> {
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
                base_settings(post_process).as_bytes(),
            ),
        ] {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

fn stl() -> Vec<u8> {
    b"solid square\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.2\nvertex 0 1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.2\nvertex 1 0 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.2\nvertex 0 -1 0.2\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.2\nvertex -1 0 0.2\nendloop\nendfacet\nendsolid square".to_vec()
}

// ---------------------------------------------------------------------------
// Catalog for the profile-path cases.
// ---------------------------------------------------------------------------

fn catalog_parts() -> (Value, Value) {
    let machine = json!({
        "type": "machine", "name": "Acme A1 0.4 nozzle", "from": "system",
        "instantiation": "true", "nozzle_diameter": [0.4],
        "printable_area": ["0x0", "256x0", "256x256", "0x256"],
        "use_relative_e_distances": false
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

// ---------------------------------------------------------------------------
// RED cases.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn embedded_post_process_with_empty_string_is_rejected() {
    // Matrix S13: Prusa process_common_mk3 writes "post_process": [""],
    // a single-element array whose sole value is an empty string. Upstream
    // checks size() > 0 (`:1686`), so [""] is rejected exactly like ["script"].
    let error = slice_project_plate_validated(
        project_3mf(Some(json!([""]))),
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("post_process is not supported under CLI."),
        "{error}"
    );
}

#[tokio::test]
async fn embedded_post_process_with_script_is_rejected() {
    let error = slice_project_plate_validated(
        project_3mf(Some(json!(["my_script.sh"]))),
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("post_process is not supported under CLI."),
        "{error}"
    );
}

#[tokio::test]
async fn embedded_post_process_empty_array_is_allowed() {
    // size() == 0 → upstream does not reject.
    slice_project_plate_validated(
        project_3mf(Some(json!([]))),
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn absent_post_process_is_allowed() {
    slice_project_plate_validated(
        project_3mf(None),
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn stl_with_profile_post_process_is_not_rejected() {
    // Native 2.4.2 verified: STL + --load-settings (process containing
    // post_process) slices successfully — the check is only in the
    // is_bbl_3mf branch (`:1682`).
    let (mut catalog, selection) = catalog_parts();
    catalog["vendors"][0]["files"]["process.json"]["post_process"] = json!([""]);
    let overrides = ConfigOverrides::default();
    slice_with_profiles_validated(
        stl(),
        ProfileSliceRequest {
            filename: "square.stl",
            catalog_json: &catalog.to_string(),
            selection_json: &selection.to_string(),
            plate: None,
            overrides: &overrides,
            // Center the raw STL on the bed for the volume gate.
            placement: Placement::Center,
        },
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn three_mf_with_profile_checks_embedded_not_overlay() {
    // The embedded 3MF has post_process: [""]; the profile overlay does not
    // supply the key. Upstream checks the embedded config (`:1683` reads
    // the `config` returned by read_from_file), so the embedded value wins.
    let (catalog, selection) = catalog_parts();
    let overrides = ConfigOverrides::default();
    let error = slice_with_profiles_validated(
        project_3mf(Some(json!([""]))),
        ProfileSliceRequest {
            filename: "cube.3mf",
            catalog_json: &catalog.to_string(),
            selection_json: &selection.to_string(),
            plate: None,
            overrides: &overrides,
            placement: Placement::None,
        },
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("post_process is not supported under CLI."),
        "{error}"
    );
}
