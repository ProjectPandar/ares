//! OrcaSlicer.cpp:3647-3653 / PrintConfig.cpp:10391-10394, original RH3D inputs.
use std::fs;

use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SliceError, SlicerContext,
};
use serde_json::{Value, json};

use super::{OrcaRunner, stages, vendor, write_json};

const FILAMENT: &str = "Generic PLA @E3NG v1.2S";
const ERROR: &str =
    "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe";

macro_rules! original_case {
    ($test:ident, $nozzle:literal) => {
        #[tokio::test]
        async fn $test() {
            original_profile($nozzle).await;
        }
    };
}

original_case!(orca_parity_o2_original_rh3d_02_loaded_config, "0.2");
original_case!(orca_parity_o2_original_rh3d_03_loaded_config, "0.3");
original_case!(orca_parity_o2_original_rh3d_04_loaded_config, "0.4");
original_case!(orca_parity_o2_original_rh3d_05_loaded_config, "0.5");
original_case!(orca_parity_o2_original_rh3d_06_loaded_config, "0.6");

async fn original_profile(nozzle: &str) {
    let root = super::super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "RH3D").unwrap();
    let machine = format!("E3NG v1.2S - {nozzle} nozzle");
    let machine_fields = profiles.machine(&machine).unwrap();
    let process = machine_fields["default_print_profile"].as_str().unwrap();
    let catalog = json!({"vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "RH3D")], "imports": []});
    let selection = json!({"machine": machine, "process": process, "filaments": [FILAMENT]});
    let directory = tempfile::Builder::new()
        .prefix(&format!("o2-rh3d-{nozzle}-"))
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    for (kind, fields) in [
        ("machine", machine_fields.clone()),
        ("process", profiles.process(process).unwrap()),
        ("filament", profiles.filament(FILAMENT).unwrap()),
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
            format!("Param values in 3mf/config error: \n{ERROR}\n")
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
    let placed = ares_core::profile_placed_project(&input, request).await;
    assert!(
        placed.is_ok(),
        "placed scene must allow editing invalid config; evidence {directory:?}"
    );
    let validated = ares_core::validate_with_profiles(&input, request);
    let sliced = ares_core::slice_with_profiles_validated(
        &input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(gcode) = &sliced {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    write_json(
        &directory.join("result.json"),
        &json!({
            "native": native,
            "loaded": loaded.as_ref().map(|view| json!(view)).map_err(ToString::to_string),
            "validated": validated.as_ref().map_err(ToString::to_string),
            "sliced": sliced.as_ref().map(|_| "GCODE").map_err(ToString::to_string),
        }),
    );
    let loaded = loaded.unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap()["default_filament_profile"],
        json!([FILAMENT])
    );
    assert!(
        matches!(validated, Err(SliceError::InvalidValuesInProject(text)) if text == ERROR),
        "evidence {directory:?}"
    );
    assert!(
        matches!(sliced, Err(SliceError::InvalidValuesInProject(text)) if text == ERROR),
        "evidence {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}
