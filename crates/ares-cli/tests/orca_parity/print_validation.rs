//! Real stock inputs rejected by `Print.cpp:1695-1749::Print::validate`.
use std::fs;

#[path = "print_validation/applied_map_controls.rs"]
mod applied_map_controls;
#[path = "print_validation/bed_compatibility.rs"]
mod bed_compatibility;
#[path = "print_validation/bed_compatibility_controls.rs"]
mod bed_compatibility_controls;
#[path = "print_validation/bed_compatibility_native_controls.rs"]
mod bed_compatibility_native_controls;
#[path = "print_validation/bridge_width.rs"]
mod bridge_width;
#[path = "print_validation/bridge_width_controls.rs"]
mod bridge_width_controls;
#[path = "print_validation/c1_thresholds.rs"]
mod c1_thresholds;
#[path = "print_validation/entries.rs"]
mod entries;
#[path = "print_validation/layer_nozzle.rs"]
mod layer_nozzle;
#[path = "print_validation/layer_nozzle_controls.rs"]
mod layer_nozzle_controls;
#[path = "print_validation/maximum_width.rs"]
mod maximum_width;
#[path = "print_validation/maximum_width_controls.rs"]
mod maximum_width_controls;
#[path = "print_validation/minimum_width.rs"]
mod minimum_width;
#[path = "print_validation/minimum_width_controls.rs"]
mod minimum_width_controls;
#[path = "print_validation/positive.rs"]
mod positive;
#[path = "print_validation/rejections.rs"]
pub(crate) mod rejections;
#[path = "print_validation/tiq2_corner.rs"]
pub(crate) mod tiq2_corner;

use ares_core::{ConfigOverrides, SliceError};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const RELATIVE_INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/layer-reset-kobra-plus-relative.3mf");
const RELATIVE_SHA256: &str = "0919c79bc4f84dd0fdaffd237645a0e91124fa12cf07a24da6376d4eeeca9093";
const RELATIVE_ERROR: &str = "Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add \"G92 E0\" to layer_gcode.";

const ABSOLUTE_INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/layer-reset-dremel-3d40-absolute.3mf");
const ABSOLUTE_SHA256: &str = "9bede932e95c6e2a1c558667e233c75c3d467ac0f8ed3c3628c03be1e7a9ae01";
const ABSOLUTE_ERROR: &str = "\"G92 E0\" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing.";

fn evidence_directory(prefix: &str) -> std::path::PathBuf {
    let mut builder = tempfile::Builder::new();
    builder.prefix(prefix);
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    };
    directory.unwrap().keep()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn relative_stock_project_rejects_missing_active_layer_reset() {
    verify_stock_rejection(
        "stock/Anycubic/Anycubic Kobra Plus 0.4 nozzle",
        RELATIVE_INPUT,
        RELATIVE_SHA256,
        RELATIVE_ERROR,
    );
}

#[test]
fn absolute_stock_project_rejects_before_layer_reset() {
    verify_stock_rejection(
        "stock/Dremel/Dremel 3D40 0.4 nozzle",
        ABSOLUTE_INPUT,
        ABSOLUTE_SHA256,
        ABSOLUTE_ERROR,
    );
}

fn verify_stock_rejection(label: &str, project: &[u8], expected_hash: &str, expected_error: &str) {
    assert_eq!(sha256(project), expected_hash);
    let runner = OrcaRunner::from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("o1-stock-")
        .tempdir()
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    fs::write(&input, project).unwrap();
    let oracle = runner.as_ref().map(|runner| {
        let output = directory.join("orca");
        fs::create_dir(&output).unwrap();
        let error = stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                input.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &directory,
        )
        .unwrap_err();
        let captured = error.captured.as_ref().unwrap();
        let stderr = String::from_utf8(captured.stderr.clone()).unwrap();
        let expected_code = if cfg!(unix) { 205 } else { -51 };
        assert_eq!(captured.code, Some(expected_code), "{error}");
        assert_eq!(
            stderr.lines().collect::<Vec<_>>(),
            [expected_error],
            "{error}"
        );
        assert!(!output.join("plate_1.gcode").exists());
        json!({
            "code": captured.code,
            "error": expected_error,
            "evidence": error.evidence,
        })
    });
    let actual = ares_core::validate_project_plate(project, None, &ConfigOverrides::default());
    let outcome = match &actual {
        Ok(()) => json!({"status": "VALIDATED"}),
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "printer": label,
            "input_sha256": expected_hash,
            "expected_error": expected_error,
            "oracle": oracle,
            "ares": outcome,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(actual, Err(SliceError::Validate { ref message, .. }) if message == expected_error),
        "O1 must reject the original stock input; evidence: {}",
        directory.display()
    );
    assert!(!directory.join("ares.gcode").exists());
}
