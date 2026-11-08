use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::{
    golden,
    presets::VendorProfiles,
    runner::{OrcaRunner, stages},
};

#[path = "dirty_columns.rs"]
mod dirty_columns;

pub(super) async fn verify(fields: Value, expected_error: Option<&str>, expected_dirty: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        eprintln!("C1 live reference unprovisioned; dedicated acceptance requires Native");
        return;
    };
    let profiles = VendorProfiles::load(
        &crate::runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 S1 Plus 0.4 nozzle")
            .unwrap();
    let model = crate::runner::repo_root().join("tests/parity/cube08.stl");
    let exported = crate::export_selection_case(&runner, &profiles, &selection, &model).unwrap();
    let directory = super::super::evidence_directory("c1-threshold-");
    let input = directory.join("input.3mf");
    fs::write(&input, &exported.project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let effective = output.join("effective.3mf");
    let mut args = Vec::new();
    for (key, value) in fields.as_object().unwrap() {
        args.push(format!("--{}", key.replace('_', "-")));
        args.push(match value {
            Value::String(value) => value.clone(),
            Value::Array(values) => values
                .iter()
                .map(|value| value.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(","),
            _ => unreachable!(),
        });
    }
    args.extend([
        "--slice".to_owned(),
        "0".to_owned(),
        "--export-3mf".to_owned(),
        "effective.3mf".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        input.to_str().unwrap().to_owned(),
    ]);
    let native = stages::run(runner.bin(), args, stages::Stage::Slice, &directory);
    let overrides = ConfigOverrides::from_json(fields.to_string().as_bytes()).unwrap();
    let validation = ares_core::validate_project_plate(&exported.project, None, &overrides);
    let sliced = crate::runner::application::slice_project_plate_validated(
        &exported.project,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    )
    .await;
    if let Ok(bytes) = &sliced {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
    }
    let cli_directory = directory.join("cli");
    fs::create_dir(&cli_directory).unwrap();
    let cli = (expected_error.is_some() || !expected_dirty.is_empty()).then(|| {
        if expected_dirty.is_empty() {
            crate::runner::application::CliCommand::new(assert_cmd::cargo::cargo_bin!("ares"))
                .args([
                    "slice",
                    "--overrides",
                    &fields.to_string(),
                    "-o",
                    cli_directory.join("output.gcode").to_str().unwrap(),
                    input.to_str().unwrap(),
                ])
                .output()
                .unwrap()
        } else {
            crate::runner::application::cli_command()
                .args([
                    "slice",
                    "--overrides",
                    &fields.to_string(),
                    "-o",
                    cli_directory.join("output.gcode").to_str().unwrap(),
                    input.to_str().unwrap(),
                ])
                .output()
                .unwrap()
        }
    });
    let reference = fs::read(output.join("plate_1.gcode"));
    let comparison = match (&reference, &sliced) {
        (Ok(native), Ok(ares)) => Some(golden::compare_ordered_bytes_generator_only(native, ares)),
        _ => None,
    };
    let application = fs::read(&effective).map(|project| {
        fields
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                crate::runner::application::verify_value(&project, key, value)
                    .map_err(|error| error.to_string())
            })
            .collect::<Vec<_>>()
    });
    let native_evidence = match &native {
        Ok(()) => json!({"status":"ACCEPTED"}),
        Err(error) => json!({
            "status":"ERROR", "stage":format!("{:?}",error.stage),
            "kind":format!("{:?}",error.kind), "evidence":error.evidence,
            "capture":error.captured.as_ref().map(|capture| json!({
                "code":capture.code,"stderr":String::from_utf8_lossy(&capture.stderr),
                "stdout":String::from_utf8_lossy(&capture.stdout),
            })),
        }),
    };
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "upstream":"345efec64c7b95a05ce5cf96df0dde66a158835c",
            "input_sha256":super::super::sha256(&exported.project),
            "model_sha256":super::super::sha256(&fs::read(&model).unwrap()),
            "prediction_mismatch":native.is_err() != expected_error.is_some(),
            "overrides":fields,"expected_error":expected_error,"native":native_evidence,
            "native_binary":std::env::var("ARES_ORCA_NATIVE_BIN").ok(),
            "resources":std::env::var("ARES_ORCA_RESOURCES_DIR").ok(),
            "cli":cli.as_ref().map(|output|json!({"code":output.status.code(),"stderr":String::from_utf8_lossy(&output.stderr),"stdout":String::from_utf8_lossy(&output.stdout)})),
            "validation":validation.as_ref().map_err(ToString::to_string),
            "sliced":sliced.as_ref().map(Vec::len).map_err(ToString::to_string),
            "application":application.as_ref().map_err(ToString::to_string),
            "native_output_sha256":reference.as_ref().ok().map(|bytes|super::super::sha256(bytes)),
            "ares_output_sha256":sliced.as_ref().ok().map(|bytes|super::super::sha256(bytes)),
            "comparison":comparison,
        }))
        .unwrap(),
    )
    .unwrap();
    if let Some(expected) = expected_dirty.first() {
        assert!(native.is_ok(), "Native generation; {directory:?}");
        validation.as_ref().unwrap();
        dirty_columns::verify(
            expected,
            reference.as_ref().unwrap(),
            sliced.as_ref().unwrap(),
            cli.as_ref().unwrap(),
            &directory,
        );
    }
    match expected_error {
        Some(expected) => {
            let error = native.unwrap_err();
            assert_eq!(error.kind, stages::FailureKind::Process, "{directory:?}");
            let capture = error.captured.unwrap();
            assert_eq!(
                capture.code,
                Some(if cfg!(unix) { 254 } else { -2 }),
                "{directory:?}"
            );
            assert_eq!(
                capture.stderr,
                format!("Params in command line error: \n{expected}\nsetup params error\n")
                    .as_bytes(),
                "{directory:?}"
            );
            let cli = cli.unwrap();
            assert_eq!(cli.status.code(), capture.code, "CLI exit; {directory:?}");
            assert_eq!(cli.stderr, capture.stderr, "CLI diagnostic; {directory:?}");
            assert_eq!(cli.stdout, capture.stdout, "CLI stdout; {directory:?}");
            assert!(
                !output.join("result.json").exists(),
                "Native setup result.json; {directory:?}"
            );
            assert!(
                !cli_directory.join("result.json").exists(),
                "CLI setup result.json; {directory:?}"
            );
            assert!(
                !cli_directory.join("output.gcode").exists(),
                "{directory:?}"
            );
            assert!(
                matches!(validation, Err(SliceError::InvalidInput(ref message)) if message == expected),
                "{validation:?}; {directory:?}"
            );
            assert!(
                matches!(sliced, Err(SliceError::InvalidInput(ref message)) if message == expected),
                "{sliced:?}; {directory:?}"
            );
            assert!(
                reference.is_err(),
                "unexpected Native output; {directory:?}"
            );
            assert!(!directory.join("ares.gcode").exists(), "{directory:?}");
        }
        None => match native {
            Err(error) => {
                assert!(
                    fields == json!({"layer_height":"0.001"})
                        || fields == json!({"initial_layer_print_height":"0.001"}),
                    "unexpected rejection outside the approved layer-height ruling; {error}; {directory:?}"
                );
                assert_eq!(error.kind, stages::FailureKind::Process, "{directory:?}");
                let capture = error.captured.unwrap();
                assert_eq!(
                    capture.code,
                    Some(if cfg!(unix) { 205 } else { -51 }),
                    "{directory:?}"
                );
                let stderr = std::str::from_utf8(&capture.stderr).unwrap();
                let message = stderr.strip_suffix('\n').unwrap();
                assert_eq!(message.lines().count(), 1, "{directory:?}");
                assert!(
                    matches!(validation, Err(SliceError::InvalidInput(ref actual)) if actual == message),
                    "{validation:?}; {directory:?}"
                );
                assert!(
                    matches!(sliced, Err(SliceError::InvalidInput(ref actual)) if actual == message),
                    "{sliced:?}; {directory:?}"
                );
                assert!(
                    reference.is_err(),
                    "unexpected Native output; {directory:?}"
                );
                assert!(!directory.join("ares.gcode").exists(), "{directory:?}");
            }
            Ok(()) => {
                validation.unwrap_or_else(|error| panic!("{error}; {directory:?}"));
                for applied in application.unwrap() {
                    applied.unwrap_or_else(|error| panic!("{error}; {directory:?}"));
                }
                comparison
                    .expect("both complete outputs required")
                    .unwrap_or_else(|error| panic!("{error}; {directory:?}"));
            }
        },
    }
}
