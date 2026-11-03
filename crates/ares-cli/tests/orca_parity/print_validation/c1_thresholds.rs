//! PrintConfig.cpp:10340-10457: independent public rejection and complete-byte gates.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::{
    golden,
    presets::VendorProfiles,
    runner::{OrcaRunner, stages},
};

macro_rules! threshold_case {
    ($name:ident, $key:literal, $value:literal, $expected:expr) => {
        #[tokio::test]
        async fn $name() {
            let value = if $key.starts_with("filament_") {
                json!([$value])
            } else {
                json!($value)
            };
            verify(json!({$key: value}), $expected).await;
        }
    };
}

threshold_case!(
    c1_layer_height_zero,
    "layer_height",
    "0",
    Some("layer_height: invalid value 0.000000")
);
threshold_case!(c1_layer_height_positive, "layer_height", "0.001", None);
threshold_case!(
    c1_initial_height_zero,
    "initial_layer_print_height",
    "0",
    Some("initial_layer_print_height: invalid value 0.000000")
);
threshold_case!(
    c1_initial_height_positive,
    "initial_layer_print_height",
    "0.001",
    None
);
threshold_case!(
    c1_bridge_flow_zero,
    "bridge_flow",
    "0",
    Some("bridge_flow: invalid value 0.000000")
);
threshold_case!(c1_bridge_flow_positive, "bridge_flow", "0.001", None);
threshold_case!(
    c1_internal_bridge_flow_zero,
    "internal_bridge_flow",
    "0",
    Some("internal_bridge_flow: invalid value 0.000000")
);
threshold_case!(
    c1_internal_bridge_flow_positive,
    "internal_bridge_flow",
    "0.001",
    None
);
threshold_case!(
    c1_clearance_radius_zero,
    "extruder_clearance_radius",
    "0",
    Some("extruder_clearance_radius: invalid value 0.000000")
);
threshold_case!(
    c1_clearance_radius_positive,
    "extruder_clearance_radius",
    "0.001",
    None
);
threshold_case!(
    c1_clearance_rod_zero,
    "extruder_clearance_height_to_rod",
    "0",
    Some("extruder_clearance_height_to_rod: invalid value 0.000000")
);
threshold_case!(
    c1_clearance_rod_positive,
    "extruder_clearance_height_to_rod",
    "0.001",
    None
);
threshold_case!(
    c1_clearance_lid_zero,
    "extruder_clearance_height_to_lid",
    "0",
    Some("extruder_clearance_height_to_lid: invalid value 0.000000")
);
threshold_case!(
    c1_clearance_lid_positive,
    "extruder_clearance_height_to_lid",
    "0.001",
    None
);
threshold_case!(
    c1_nozzle_height_zero,
    "nozzle_height",
    "0",
    Some("nozzle_height: invalid value 0.000000")
);
threshold_case!(c1_nozzle_height_positive, "nozzle_height", "0.001", None);
threshold_case!(
    c1_filament_flow_zero,
    "filament_flow_ratio",
    "0",
    Some("filament_flow_ratio: invalid value 0")
);
threshold_case!(
    c1_filament_flow_positive,
    "filament_flow_ratio",
    "0.001",
    None
);
threshold_case!(
    c1_filament_diameter_schema_zero,
    "filament_diameter",
    "0",
    Some("filament_diameter: invalid value 0")
);
threshold_case!(
    c1_filament_diameter_below,
    "filament_diameter",
    "0.999",
    Some("filament_diameter: invalid value 0.999")
);
threshold_case!(
    c1_filament_diameter_threshold,
    "filament_diameter",
    "1",
    None
);
threshold_case!(
    c1_filament_diameter_above,
    "filament_diameter",
    "1.001",
    None
);

#[tokio::test]
async fn c1_unchanged_control() {
    verify(json!({}), None).await;
}

async fn verify(fields: Value, expected_error: Option<&str>) {
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
    let directory = super::evidence_directory("c1-threshold-");
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
    let cli = expected_error.map(|_| {
        std::process::Command::new(assert_cmd::cargo::cargo_bin!("ares"))
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
            "upstream":"8500fcdccaa10b5099ac20d252af3a7c560046f1",
            "input_sha256":super::sha256(&exported.project),
            "model_sha256":super::sha256(&fs::read(&model).unwrap()),
            "prediction_mismatch":native.is_err() != expected_error.is_some(),
            "overrides":fields,"expected_error":expected_error,"native":native_evidence,
            "native_binary":std::env::var("ARES_ORCA_NATIVE_BIN").ok(),
            "resources":std::env::var("ARES_ORCA_RESOURCES_DIR").ok(),
            "cli":cli.as_ref().map(|output|json!({"code":output.status.code(),"stderr":String::from_utf8_lossy(&output.stderr),"stdout":String::from_utf8_lossy(&output.stdout)})),
            "validation":validation.as_ref().map_err(ToString::to_string),
            "sliced":sliced.as_ref().map(Vec::len).map_err(ToString::to_string),
            "application":application.as_ref().map_err(ToString::to_string),
            "native_output_sha256":reference.as_ref().ok().map(|bytes|super::sha256(bytes)),
            "ares_output_sha256":sliced.as_ref().ok().map(|bytes|super::sha256(bytes)),
            "comparison":comparison,
        }))
        .unwrap(),
    )
    .unwrap();
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
