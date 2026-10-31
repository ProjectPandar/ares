//! O2 observable contracts: Config.cpp loading precedes variant selection / legacy process.
use std::fs;

use super::{OrcaRunner, stages, vendor, write_json};
use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SliceError, SliceOptions,
    SlicerContext,
};
use serde_json::{Value, json};

const ERROR: &str =
    "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe";

#[tokio::test]
async fn legacy_stl_firmware_wipe() {
    let directory = tempfile::Builder::new()
        .prefix("o2-legacy-stl-")
        .tempdir()
        .unwrap()
        .keep();
    let input = fs::read(super::super::repo_root().join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    let options_json =
        json!({"use_firmware_retraction": true, "wipe": true, "use_relative_e_distances": false});
    write_json(&directory.join("options.json"), &options_json);
    let native = native_rejection(
        &directory,
        vec![
            "--use-firmware-retraction".to_owned(),
            "--wipe=1".to_owned(),
            "--use-relative-e-distances=0".to_owned(),
        ],
        true,
    );
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let actual = ares_core::slice_stl_validated(&input, options).await;
    record_actual(&directory, native, &actual);
    assert!(
        matches!(actual, Err(SliceError::InvalidValuesInProject(text)) if text == ERROR),
        "evidence {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}

#[tokio::test]
async fn any_wipe_before_variant_selection() {
    let profiles_root = super::super::repo_root().join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, "RH3D").unwrap();
    let machine = "E3NG v1.2S - 0.4 nozzle";
    let process = "0.20mm Standard @E3NG v1.2S";
    let filament = "Generic PLA @E3NG v1.2S";
    let edits = json!({
        "wipe": ["0", "1"],
        "extruder_variant_list": ["Direct Drive Standard,Bowden Standard"],
        "printer_extruder_id": ["1", "1"],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "print_extruder_id": ["1", "1"],
        "print_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "extruder_type": ["Direct Drive"],
        "nozzle_volume_type": ["Standard"],
        "filament_extruder_variant": ["Direct Drive Standard"]
    });
    let overrides = ConfigOverrides::from_json(&serde_json::to_vec(&edits).unwrap()).unwrap();
    let catalog = json!({"vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, "RH3D")], "imports": []});
    let selection = json!({"machine": machine, "process": process, "filaments": [filament]});
    let directory = tempfile::Builder::new()
        .prefix("o2-any-wipe-")
        .tempdir()
        .unwrap()
        .keep();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    write_json(&directory.join("overrides.json"), &edits);
    let input = fs::read(super::super::repo_root().join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    let mut machine_fields = profiles.machine(machine).unwrap();
    machine_fields.extend(edits.as_object().unwrap().clone());
    for (kind, fields) in [
        ("machine", machine_fields),
        ("process", profiles.process(process).unwrap()),
        ("filament", profiles.filament(filament).unwrap()),
    ] {
        write_json(
            &directory.join(format!("{kind}.json")),
            &Value::Object(fields),
        );
    }
    let path = |file: &str| directory.join(file).to_str().unwrap().to_owned();
    let native = native_rejection(
        &directory,
        vec![
            "--load-settings".to_owned(),
            format!("{};{}", path("machine.json"), path("process.json")),
            "--load-filaments".to_owned(),
            path("filament.json"),
        ],
        false,
    );
    let catalog_json = serde_json::to_string(&catalog).unwrap();
    let selection_json = serde_json::to_string(&selection).unwrap();
    let request = ProfileSliceRequest {
        filename: "cube10.stl",
        catalog_json: &catalog_json,
        selection_json: &selection_json,
        plate: None,
        overrides: &overrides,
        placement: Placement::None,
    };
    let loaded =
        serde_json::to_value(ares_core::profile_settings(&input, request).await.unwrap()).unwrap();
    assert_eq!(loaded["wipe"], edits["wipe"]);
    assert_eq!(loaded["nozzle_diameter"], json!(["0.4"]));
    write_json(&directory.join("loaded.json"), &loaded);
    let actual = ares_core::slice_with_profiles_validated(
        &input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await;
    record_actual(&directory, native, &actual);
    assert!(
        matches!(actual, Err(SliceError::InvalidValuesInProject(text)) if text == ERROR),
        "evidence {directory:?}"
    );
    assert!(!directory.join("ares.gcode").exists());
}

fn native_rejection(
    directory: &std::path::Path,
    mut args: Vec<String>,
    cli: bool,
) -> Option<Value> {
    OrcaRunner::from_env().unwrap().map(|runner| {
        args.extend([
            "--arrange".to_owned(),
            "1".to_owned(),
            "--export-3mf".to_owned(),
            directory.join("native.3mf").to_str().unwrap().to_owned(),
            directory.join("cube10.stl").to_str().unwrap().to_owned(),
        ]);
        let failure =
            stages::run(runner.bin(), args, stages::Stage::Export, directory).unwrap_err();
        let captured = failure.captured.as_ref().unwrap();
        let code = if cli { -2 } else { -18 };
        assert_eq!(
            captured.code,
            Some(if cfg!(unix) { 256 + code } else { code })
        );
        let stderr = if cli {
            format!("Params in command line error: \n{ERROR}\nsetup params error\n")
        } else {
            format!("Param values in 3mf/config error: \n{ERROR}\n")
        };
        assert_eq!(std::str::from_utf8(&captured.stderr).unwrap(), stderr);
        assert!(!directory.join("native.3mf").exists());
        json!({"code": captured.code, "error": ERROR, "evidence": failure.evidence})
    })
}

fn record_actual(
    directory: &std::path::Path,
    native: Option<Value>,
    actual: &Result<Vec<u8>, SliceError>,
) {
    let actual = match actual {
        Ok(gcode) => {
            fs::write(directory.join("ares.gcode"), gcode).unwrap();
            json!({"status": "GCODE", "bytes": gcode.len()})
        }
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };
    write_json(
        &directory.join("result.json"),
        &json!({"native": native, "ares": actual}),
    );
}
