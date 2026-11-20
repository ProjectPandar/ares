//! ConfigBase::load_from_json accepts RH3D's original scalar coStrings value.
#[path = "preset_vector_loading/nil_completion.rs"]
mod nil_completion;

use std::fs;

use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SliceError, SlicerContext,
};
use serde_json::{Value, json};

use super::{OrcaRunner, stages, vendor, write_json};

const MACHINE: &str = "E3NG v1.2S - 0.4 nozzle";
const PROCESS: &str = "0.20mm Standard @E3NG v1.2S";
const ERROR: &str =
    "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe";

#[tokio::test]
async fn rh3d_original_scalar_vector_profile_loads_then_rejects_double_reset() {
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "RH3D").unwrap();
    let catalog = json!({
        "vendors": [
            vendor(&profiles_root, "OrcaFilamentLibrary"),
            vendor(&profiles_root, "RH3D"),
        ],
        "imports": [],
    });
    // The stock machine's parent chain carries this coStrings option as a scalar.
    let machine = profiles.machine(MACHINE).unwrap();
    let filament = machine["default_filament_profile"]
        .as_str()
        .unwrap()
        .to_owned();
    let selection = json!({"machine": MACHINE, "process": PROCESS, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("rh3d-vector-load-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    for (kind, fields) in [
        ("machine", machine),
        ("process", profiles.process(PROCESS).unwrap()),
        ("filament", profiles.filament(&filament).unwrap()),
    ] {
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let native = OrcaRunner::from_env().unwrap().map(|runner| {
        let presets =
            super::native_user_presets::resolved(&directory, &profiles_root, &catalog, &selection);
        let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
        let failure = stages::run(
            runner.bin(),
            [
                "--datadir".to_owned(),
                presets.datadir.to_str().unwrap().to_owned(),
                "--load-settings".to_owned(),
                super::super::join_paths(presets.settings.iter()),
                "--load-filaments".to_owned(),
                super::super::join_paths(presets.filaments.iter()),
                "--arrange".to_owned(),
                "1".to_owned(),
                "--export-3mf".to_owned(),
                path("native.3mf"),
                path("cube10.stl"),
            ],
            stages::Stage::Export,
            &directory,
        )
        .unwrap_err();
        let captured = failure.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) { 238 } else { -18 }));
        assert_eq!(
            std::str::from_utf8(&captured.stderr).unwrap(),
            format!("Param values in 3mf/config error: \n{ERROR}\n"),
        );
        assert!(!directory.join("native.3mf").exists());
        json!({"code": captured.code, "error": ERROR, "evidence": failure.evidence})
    });
    let catalog_json = serde_json::to_string(&catalog).unwrap();
    let selection_json = serde_json::to_string(&selection).unwrap();
    let overrides = ConfigOverrides::default();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let loaded = ares_core::profile_settings(&input, request).await;
    let load = match &loaded {
        Ok(view) => json!({"status": "LOADED", "settings": view}),
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };
    let sliced = ares_core::slice_with_profiles_validated(
        &input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await;
    let slice = match &sliced {
        Ok(gcode) => {
            fs::write(directory.join("ares.gcode"), gcode).unwrap();
            json!({"status": "GCODE"})
        }
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };
    write_json(
        &directory.join("result.json"),
        &json!({"native": native, "load": load, "slice": slice}),
    );
    assert!(loaded.is_ok(), "{load}; evidence {directory:?}");
    assert_eq!(
        load["settings"]["default_filament_profile"],
        json!([filament])
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidValuesInProject(text)) if text == ERROR),
        "{slice}; evidence {directory:?}",
    );
    assert!(!directory.join("ares.gcode").exists());
}
