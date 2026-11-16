//! O6: instances without plate identity and the upstream "no suitable
//! objects" geometric gate (`OrcaSlicer.cpp:137,5710-5724`,
//! `Model.cpp:692-702,3423-3469`, `BuildVolume.hpp:91-106`).

#[path = "../../../tests/support/slice_project_plate.rs"]
mod direct_project;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use direct_project::slice_project_plate;
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

/// A 10 mm cube mesh with vertices spanning [-5,5] per axis.
const ROOT_MODEL_OFF_BED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <metadata name="OrcaSlicer">2.4.2</metadata>
 <resources>
  <object id="2" type="model"><mesh><vertices>
   <vertex x="-5" y="-5" z="0"/><vertex x="5" y="-5" z="0"/><vertex x="-5" y="5" z="0"/><vertex x="5" y="5" z="0"/>
   <vertex x="-5" y="-5" z="10"/><vertex x="5" y="-5" z="10"/><vertex x="-5" y="5" z="10"/><vertex x="5" y="5" z="10"/>
  </vertices><triangles>
   <triangle v1="0" v2="1" v3="2"/><triangle v1="1" v2="3" v3="2"/>
   <triangle v1="4" v2="6" v3="5"/><triangle v1="5" v2="6" v3="7"/>
   <triangle v1="0" v2="4" v3="1"/><triangle v1="1" v2="4" v3="5"/>
   <triangle v1="2" v2="3" v3="6"/><triangle v1="3" v2="7" v3="6"/>
   <triangle v1="0" v2="2" v3="4"/><triangle v1="2" v2="6" v3="4"/>
   <triangle v1="1" v2="5" v3="3"/><triangle v1="3" v2="5" v3="7"/>
  </triangles></mesh></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 5 -56 0" printable="1" auto_drop="1"/></build>
</model>"#;

/// Same mesh but the instance is inside a 255×255 bed (bed center-ish).
const ROOT_MODEL_IN_BED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">
 <metadata name="OrcaSlicer">2.4.2</metadata>
 <resources>
  <object id="2" type="model"><mesh><vertices>
   <vertex x="-5" y="-5" z="0"/><vertex x="5" y="-5" z="0"/><vertex x="-5" y="5" z="0"/><vertex x="5" y="5" z="0"/>
   <vertex x="-5" y="-5" z="10"/><vertex x="5" y="-5" z="10"/><vertex x="-5" y="5" z="10"/><vertex x="5" y="5" z="10"/>
  </vertices><triangles>
   <triangle v1="0" v2="1" v3="2"/><triangle v1="1" v2="3" v3="2"/>
   <triangle v1="4" v2="6" v3="5"/><triangle v1="5" v2="6" v3="7"/>
   <triangle v1="0" v2="4" v3="1"/><triangle v1="1" v2="4" v3="5"/>
   <triangle v1="2" v2="3" v3="6"/><triangle v1="3" v2="7" v3="6"/>
   <triangle v1="0" v2="2" v3="4"/><triangle v1="2" v2="6" v3="4"/>
   <triangle v1="1" v2="5" v3="3"/><triangle v1="3" v2="5" v3="7"/>
  </triangles></mesh></object>
 </resources>
 <build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 122 122 0" printable="1" auto_drop="1"/></build>
</model>"#;

/// Plate metadata with NO model_instance entries (the O6 shape).
const MODEL_SETTINGS_EMPTY_PLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
 <object id="2">
   <part id="1" subtype="normal_part"><metadata key="name" value="cube"/><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate>
   <metadata key="plater_id" value="1"/>
   <metadata key="gcode_file" value=""/>
 </plate>
 <assemble></assemble>
</config>"#;

/// Normal plate metadata with the instance properly assigned.
const MODEL_SETTINGS_WITH_INSTANCE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
 <object id="2">
   <part id="1" subtype="normal_part"><metadata key="name" value="cube"/><metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/></part></object>
 <plate>
   <metadata key="plater_id" value="1"/>
   <model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="133"/></model_instance>
 </plate>
 <assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble>
</config>"#;

const SLICE_INFO: &str =
    r#"<config><header><header_item key="OrcaSlicer-Version" value="2.4.2"/></header></config>"#;
const FILAMENT_SEQUENCE: &str =
    r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[]}}"#;

fn settings() -> String {
    json!({
        "printer_settings_id": "Kobra 3",
        "print_settings_id": "Std",
        "filament_settings_id": ["PLA"],
        "nozzle_diameter": [0.4],
        "printable_area": ["0x0", "255x0", "255x255", "0x255"],
        "printable_height": 260,
        "use_relative_e_distances": false,
        "layer_change_gcode": "G92 E0"
    })
    .to_string()
}

fn project_3mf(model: &str, settings_config: &str) -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        use std::io::Write as _;
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for (path, bytes) in [
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
            ("3D/root.model", model.as_bytes()),
            ("Metadata/model_settings.config", settings_config.as_bytes()),
            ("Metadata/slice_info.config", SLICE_INFO.as_bytes()),
            (
                "Metadata/filament_sequence.json",
                FILAMENT_SEQUENCE.as_bytes(),
            ),
            ("Metadata/project_settings.config", settings().as_bytes()),
        ] {
            writer.start_file(path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    buffer.into_inner()
}

/// Matrix O6: plate metadata is empty, the build instance is outside the
/// bed. Native exits -50; Ares must return the upstream message, not a
/// load-time "no loaded plate identity" error.
#[tokio::test]
async fn off_bed_instance_with_empty_plate_returns_upstream_error() {
    let error = slice_project_plate(
        project_3mf(ROOT_MODEL_OFF_BED, MODEL_SETTINGS_EMPTY_PLATE),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume."),
        "{error}"
    );
}

/// The same empty plate metadata but the instance IS inside the bed.
/// Native SIGSEGVs here (upstream bug); Ares slices normally (known
/// deviation, documented in the spec and commit message).
#[tokio::test]
async fn in_bed_instance_with_empty_plate_slices_normally() {
    let gcode = slice_project_plate(
        project_3mf(ROOT_MODEL_IN_BED, MODEL_SETTINGS_EMPTY_PLATE),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert!(!gcode.is_empty(), "in-bed case must produce output");
}

/// A normal 3MF (instance assigned to a plate, inside the bed) must
/// continue to slice — no regression from the Option change.
#[tokio::test]
async fn normal_instance_with_plate_identity_slices_normally() {
    let gcode = slice_project_plate(
        project_3mf(ROOT_MODEL_IN_BED, MODEL_SETTINGS_WITH_INSTANCE),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert!(!gcode.is_empty(), "normal case must produce output");
}

/// The matrix real fixture (exported from the Kobra 3 0.4 preset):
/// same shape as the synthetic off-bed case.
#[tokio::test]
async fn kobra3_matrix_fixture_returns_upstream_error() {
    let fixture = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/parity/validation/print-volume-kobra3-off-bed.3mf"
    ))
    .unwrap();
    let error = slice_project_plate(
        fixture,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume."),
        "{error}"
    );
}

/// Circle bed (72-corner discretized, like the Rolohaun Delta Flyer):
/// a model outside the circle must be rejected with the upstream -50 text.
#[tokio::test]
async fn circle_bed_model_outside_is_rejected() {
    // Build a 72-corner circle bed of radius 55 centered at origin.
    let mut area = Vec::new();
    for i in 0..72 {
        let angle = i as f64 * std::f64::consts::TAU / 72.0;
        area.push(format!(
            "{:.5}x{:.5}",
            55.0 * angle.cos(),
            55.0 * angle.sin()
        ));
    }
    let mut s: serde_json::Value = serde_json::from_str(&settings()).unwrap();
    s["printable_area"] = json!(area);
    let s = s.to_string();
    // The object at (150, 150) is outside the radius-55 circle.
    let error = slice_project_plate(
        project_3mf_custom(ROOT_MODEL_IN_BED, MODEL_SETTINGS_WITH_INSTANCE, &s),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume."),
        "{error}"
    );
}

/// Circle bed: a model inside the circle slices normally.
#[tokio::test]
async fn circle_bed_model_inside_slices_normally() {
    let mut area = Vec::new();
    for i in 0..72 {
        let angle = i as f64 * std::f64::consts::TAU / 72.0;
        area.push(format!(
            "{:.5}x{:.5}",
            55.0 * angle.cos(),
            55.0 * angle.sin()
        ));
    }
    let mut s: serde_json::Value = serde_json::from_str(&settings()).unwrap();
    s["printable_area"] = json!(area);
    let s = s.to_string();
    // The object at (0, 0) is inside the radius-55 circle.
    let gcode = slice_project_plate(
        project_3mf_custom(
            &ROOT_MODEL_IN_BED.replace("122 122", "0 0"),
            MODEL_SETTINGS_WITH_INSTANCE,
            &s,
        ),
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    assert!(!gcode.is_empty());
}

fn project_3mf_custom(model: &str, settings_config: &str, settings_json: &str) -> Vec<u8> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        use std::io::Write as _;
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default();
        for (path, bytes) in [
            ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
            ("_rels/.rels", ROOT_RELATIONSHIPS.as_bytes()),
            ("3D/root.model", model.as_bytes()),
            ("Metadata/model_settings.config", settings_config.as_bytes()),
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

/// Stock-backed Model/BuildVolume rejection, not a bbox-corner approximation.
/// CLI:5717-5723 -> Model.cpp:692-702/2370-2383/3423-3469 ->
/// BuildVolume.cpp:377-410. Receipt: stock-circle-print-volume spec/ARD.
#[tokio::test]
async fn circle_bed_cylinder_matches_native_no_suitable_rejection() {
    let input =
        include_bytes!("../../../tests/parity/validation/print-volume-stock-circle-cylinder.3mf");
    let error = slice_project_plate(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume."
    );
}
