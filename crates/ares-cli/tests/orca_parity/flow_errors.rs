//! `Flow.cpp:147-164`: processing errors are not hidden Print validation.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/minimum-width-ender3s1plus-0.2.3mf");
const HASH: &str = "edb48192a609ede526e8e09e805b845ff99494c0d2d2b31a8655ca9b0ac89110";
const INVALID_SPACING: &str =
    "Invalid spacing supplied to Flow::with_spacing(), check your layer height and extrusion width";

#[test]
fn s1plus_pure_processing_returns_spacing_invalid_argument() {
    verify_error("flow-s1plus-red-", b"{}", INVALID_SPACING);
}

#[test]
fn narrow_width_returns_negative_spacing() {
    verify_error(
        "flow-negative-spacing-",
        br#"{"inner_wall_line_width":"0.01"}"#,
        "Flow::spacing() produced negative spacing. Did you set some extrusion width too small?",
    );
}

#[test]
fn zero_bridge_ratio_keeps_preflight_first_error() {
    verify_error(
        "flow-negative-volume-",
        br#"{"bridge_flow":"0","thick_bridges":"0"}"#,
        "invalid Orca option bridge_flow",
    );
}

#[test]
fn positive_spacing_and_flow_generate_complete_gcode() {
    let overrides = ConfigOverrides::from_json(
        br#"{
        "line_width":"0.4", "inner_wall_line_width":"0.4", "outer_wall_line_width":"0.4",
        "sparse_infill_line_width":"0.4", "internal_solid_infill_line_width":"0.4",
        "top_surface_line_width":"0.4", "skin_infill_line_width":"0.4",
        "skeleton_infill_line_width":"0.4", "bridge_flow":"1", "thick_bridges":"0"
    }"#,
    )
    .unwrap();
    let directory = tempfile::Builder::new()
        .prefix("flow-positive-")
        .tempdir()
        .unwrap()
        .keep();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let processed = runtime
        .block_on(ares_core::slice_project_plate_processed(
            INPUT,
            GenerationMetadata::default(),
            None,
            &overrides,
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &processed.gcode).unwrap();
    assert!(!processed.gcode.is_empty());
    // Beta `GCode.cpp:4882-4935` tail order for non-BBL machines: the config
    // block is written after the executable block and ends with
    // `; CONFIG_BLOCK_END\n\n` (:4902); the filament stats follow via
    // `update_print_stats_and_format_filament_stats` (:4906-4918), then the
    // `; total filament used [g] = ` / `; total filament cost = ` /
    // `; total layers count = ` lines and the estimated-printing-time
    // placeholder (:4919-4931) that the processor expands into the two
    // `; estimated ... printing time (normal mode) = ` lines. The file ends
    // with the two `file.write("\n")` calls at :4933-4934.
    let marker = b"; CONFIG_BLOCK_END\n\n";
    let tail = &processed.gcode[processed
        .gcode
        .windows(marker.len())
        .rposition(|window| window == marker)
        .expect("config block end marker")
        + marker.len()..];
    let lines: Vec<&[u8]> = tail.split_inclusive(|byte| *byte == b'\n').collect();
    let prefixes = [
        "; filament used [mm] = ",
        "; filament used [cm3] = ",
        "; filament used [g] = ",
        "; filament cost = ",
        "; total filament used [g] = ",
        "; total filament cost = ",
        "; total layers count = ",
        "; estimated printing time (normal mode) = ",
        "; estimated first layer printing time (normal mode) = ",
    ];
    assert_eq!(lines.len(), prefixes.len() + 2);
    for (line, prefix) in lines.iter().zip(prefixes) {
        assert!(line.starts_with(prefix.as_bytes()));
    }
    assert_eq!(lines[prefixes.len()], b"\n");
    assert_eq!(lines[prefixes.len() + 1], b"\n");
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": HASH, "status": "GENERATED", "bytes": processed.gcode.len()
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn validated_s1plus_keeps_minimum_width_first_error() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = runtime.block_on(ares_core::slice_project_plate_validated(
        INPUT,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    ));
    assert!(
        matches!(result, Err(SliceError::Validate { ref message, .. }) if message == "Too small line width")
    );
}

fn verify_error(prefix: &str, overrides_json: &[u8], expected: &str) {
    let overrides = ConfigOverrides::from_json(overrides_json).unwrap();
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hash, HASH);
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), INPUT).unwrap();
    fs::write(
        directory.join("invocation.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": hash,
            "api": "slice_project_plate_processed",
            "overrides": serde_json::from_slice::<serde_json::Value>(overrides_json).unwrap(),
            "expected_error": expected,
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
        &overrides,
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
        matches!(result, Err(SliceError::InvalidInput(ref message)) if message == expected),
        "pure processing must return the source Flow exception; evidence: {}",
        directory.display()
    );
    assert!(!directory.join("ares.gcode").exists());
}
