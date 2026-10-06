//! `Flow.cpp:147-164`: processing errors are not hidden Print validation.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

const INPUT: &[u8] = include_bytes!("../../../../tests/parity/validation/o4-ender3s1plus-0.2.3mf");
const HASH: &str = "edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110";
const INVALID_SPACING: &str =
    "Invalid spacing supplied to Flow::with_spacing(), check your layer height and extrusion width";

#[test]
fn s1plus_pure_processing_returns_spacing_invalid_argument() {
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hash, HASH);
    let directory = tempfile::Builder::new()
        .prefix("flow-s1plus-red-")
        .tempdir()
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), INPUT).unwrap();
    fs::write(
        directory.join("invocation.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": hash,
            "api": "slice_project_plate_processed",
            "overrides": {},
            "expected_source_type": "Slic3r::InvalidArgument",
            "expected_error": INVALID_SPACING,
        }))
        .unwrap(),
    )
    .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = runtime.block_on(ares_core::slice_project_plate_processed(
        INPUT,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    ));
    let outcome = match &result {
        Ok(processed) => {
            fs::write(directory.join("ares.gcode"), &processed.gcode).unwrap();
            json!({"status": "GENERATED", "bytes": processed.gcode.len()})
        }
        Err(error) => json!({"status": "ERROR", "error": error.to_string()}),
    };
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&outcome).unwrap(),
    )
    .unwrap();
    assert!(
        matches!(result, Err(SliceError::InvalidInput(ref message)) if message == INVALID_SPACING),
        "pure processing must return the source Flow exception; evidence: {}",
        directory.display()
    );
    assert!(!directory.join("ares.gcode").exists());
}
