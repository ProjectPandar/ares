use std::fs;

use ares_core::{
    ConfigOverrides, GenerationMetadata, Placement, ProfileSliceRequest, SlicerContext,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::profile_entry::{vendor, write_json};
use super::{OrcaRunner, stages};

#[path = "stock_validation/cases.rs"]
mod cases;
#[path = "stock_validation/project_value_out_of_range.rs"]
mod project_value_out_of_range;

async fn compare(vendor_name: &str, printer: &str, expected: &str, loaded_error: bool) {
    let Some(_) = crate::native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let root = super::repo_root();
    let profiles_root = root.join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&profiles_root, vendor_name).unwrap();
    let selected = crate::select_printer(&profiles, vendor_name, printer).unwrap();
    let catalog = json!({"vendors": [vendor(&profiles_root, "OrcaFilamentLibrary"), vendor(&profiles_root, vendor_name)], "imports": []});
    let selection = json!({"machine": selected.printer, "process": selected.process, "filaments": selected.filaments});
    let mut builder = tempfile::Builder::new();
    builder.prefix("stock-validation-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let input = fs::read(root.join("tests/parity/cube10.stl")).unwrap();
    fs::write(directory.join("cube10.stl"), &input).unwrap();
    write_json(&directory.join("catalog.json"), &catalog);
    write_json(&directory.join("selection.json"), &selection);
    let mut filament_paths = Vec::new();
    for (kind, filename, mut fields) in [
        (
            "machine",
            "machine.json".to_owned(),
            profiles.machine(printer).unwrap(),
        ),
        (
            "process",
            "process.json".to_owned(),
            profiles.process(&selected.process).unwrap(),
        ),
    ]
    .into_iter()
    .chain(selected.filaments.iter().enumerate().map(|(index, name)| {
        (
            "filament",
            format!("filament-{index}.json"),
            profiles.filament(name).unwrap(),
        )
    })) {
        fields.insert("type".to_owned(), Value::String(kind.to_owned()));
        fields.insert("from".to_owned(), Value::String("system".to_owned()));
        fields.insert("instantiation".to_owned(), Value::String("true".to_owned()));
        write_json(&directory.join(&filename), &Value::Object(fields));
        if kind == "filament" {
            filament_paths.push(directory.join(filename).to_str().unwrap().to_owned());
        }
    }
    let path = |name: &str| directory.join(name).to_str().unwrap().to_owned();
    let export = stages::run(
        runner.bin(),
        [
            "--load-settings".to_owned(),
            format!("{};{}", path("machine.json"), path("process.json")),
            "--load-filaments".to_owned(),
            filament_paths.join(";"),
            "--arrange".to_owned(),
            "1".to_owned(),
            "--export-3mf".to_owned(),
            path("native.3mf"),
            path("cube10.stl"),
        ],
        stages::Stage::Export,
        &directory,
    );
    let failure = if loaded_error {
        export.unwrap_err()
    } else {
        export.unwrap();
        stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                path("native-output"),
                path("native.3mf"),
            ],
            stages::Stage::Slice,
            &directory,
        )
        .unwrap_err()
    };
    let captured = failure.captured.as_ref().unwrap();
    let expected_code: i32 = if loaded_error { -18 } else { -51 };
    let expected_frame = if loaded_error {
        format!("Param values in 3mf/config error: \n{expected}\n")
    } else {
        format!("{expected}\n")
    };
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
    let validated = ares_core::validate_with_profiles(&input, request);
    let sliced = if loaded_error {
        ares_core::slice_with_profiles_validated(
            &input,
            request,
            GenerationMetadata::default(),
            SlicerContext::new(0),
        )
        .await
    } else {
        let project = fs::read(directory.join("native.3mf")).unwrap();
        crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::default(),
            None,
            &overrides,
            SlicerContext::new(0),
        )
        .await
    };
    if let Ok(bytes) = &sliced {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
    }
    let validate_message = validated
        .as_ref()
        .map(|_| "SUCCESS".to_owned())
        .unwrap_or_else(ToString::to_string);
    let slice_message = sliced
        .as_ref()
        .map(|_| "SUCCESS".to_owned())
        .unwrap_or_else(ToString::to_string);
    write_json(
        &directory.join("result.json"),
        &json!({
            "printer": printer, "vendor": vendor_name,
            "provenance": "RECONSTRUCTED_STOCK_SELECTION_NOT_ORIGINAL_PRESETS",
            "input_sha256": Sha256::digest(&input).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "native_code": captured.code, "native_status": captured.status,
            "native_stderr": String::from_utf8_lossy(&captured.stderr), "native_evidence": failure.evidence,
            "native_stage": format!("{:?}", failure.stage),
            "ares_validation": validate_message, "ares_slice": slice_message,
            "expected_payload": expected,
        }),
    );
    assert_eq!(
        captured.code,
        Some(if cfg!(unix) {
            expected_code.rem_euclid(256)
        } else {
            expected_code
        }),
        "artifacts {directory:?}"
    );
    assert_eq!(
        captured.stderr,
        expected_frame.as_bytes(),
        "artifacts {directory:?}"
    );
    let payload = expected.to_owned();
    assert_eq!(
        validate_message, payload,
        "validation artifacts {directory:?}"
    );
    assert_eq!(slice_message, payload, "slice artifacts {directory:?}");
    assert!(sliced.is_err(), "artifacts {directory:?}");
    assert!(
        !directory.join("native-output/plate_1.gcode").exists(),
        "artifacts {directory:?}"
    );
}
