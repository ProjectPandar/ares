//! Geometry-only witnesses for beta `Fill3DHoneycomb.cpp:336–436`.
//! The parent's full-stream assertion remains independent and unchanged.

#[path = "honeycomb_geometry/height_thickness.rs"]
mod height_thickness;
#[path = "honeycomb_geometry/planar_smoothing.rs"]
mod planar_smoothing;

use std::fs;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{ConfigOverrides, GenerationMetadata, OrcaRunner, PlaterSession, SlicerContext};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn geometry(gcode: &[u8]) -> (String, Vec<Value>) {
    let mut output = String::new();
    let mut context = Vec::new();
    let mut z = "";
    let mut role = "";
    for (index, line) in std::str::from_utf8(gcode).unwrap().lines().enumerate() {
        if let Some(value) = line.strip_prefix(";Z:") {
            z = value;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let code = line.split(';').next().unwrap();
        let mut tokens = code.split_whitespace();
        let Some(command) = tokens.next() else {
            continue;
        };
        let operands: Vec<_> = tokens.filter(|token| !token.starts_with('F')).collect();
        let motion = matches!(command, "G0" | "G1" | "G2" | "G3");
        let mode = matches!(
            command,
            "G20" | "G21" | "G90" | "G91" | "G92" | "M82" | "M83"
        );
        if !mode && !(motion && !operands.is_empty()) {
            continue;
        }
        output.push_str(command);
        for operand in operands {
            output.push(' ');
            output.push_str(operand);
        }
        output.push('\n');
        context.push(json!({"source_line": index + 1, "z": z, "role": role, "raw": line}));
    }
    (output, context)
}

fn run(label: &str, settings: Value, moved: bool) {
    if !crate::runner::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("native profile oracle");
    let presets = super::presets(&runner);
    let process_file = runner.work.join("stl-session-process.json");
    let mut process: Value = serde_json::from_slice(&fs::read(&process_file).unwrap()).unwrap();
    process
        .as_object_mut()
        .unwrap()
        .extend(settings.as_object().unwrap().clone());
    fs::write(&process_file, serde_json::to_vec_pretty(&process).unwrap()).unwrap();
    fs::write(runner.work.join("catalog.json"), &presets.catalog).unwrap();
    fs::write(runner.work.join("selection.json"), &presets.selection).unwrap();
    fs::write(
        runner.work.join("overrides.json"),
        serde_json::to_vec_pretty(&settings).unwrap(),
    )
    .unwrap();
    let overrides = ConfigOverrides::from_json(&serde_json::to_vec(&settings).unwrap()).unwrap();
    let input = fs::read(crate::runner::repo_root().join("tests/parity/cube10.stl")).unwrap();
    fs::write(runner.work.join("api-input.stl"), &input).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut session = runtime
        .block_on(PlaterSession::load_with_profiles(
            input.clone(),
            super::request(&presets, &overrides),
        ))
        .unwrap();
    let shift = if moved {
        session
            .translate_instance(0, 0, [20.0, -15.0, 3.0])
            .unwrap();
        [138.0, 123.0, 0.0]
    } else {
        [118.0, 138.0, 0.0]
    };
    let positioned = super::cube_at(shift);
    let native = super::native(&runner, label, &positioned);
    let api = runtime
        .block_on(ares_core::slice_plater_with_profiles(
            &session,
            super::request(&presets, &overrides),
            false,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            SlicerContext::new(0),
        ))
        .unwrap()
        .gcode;
    fs::write(runner.work.join("native.gcode"), &native).unwrap();
    fs::write(runner.work.join("api.gcode"), &api).unwrap();
    compare(
        &runner.work,
        &native,
        &api,
        json!({
            "scope": "ordered motion geometry excluding feedrates; not full-stream parity",
            "label": label,
            "input_sha256": sha(&input),
            "native_input_sha256": sha(&positioned),
            "settings": settings,
            "moved": moved,
        }),
    );
}

fn compare(root: &std::path::Path, native: &[u8], api: &[u8], mut report: Value) {
    let (native_geometry, native_context) = geometry(native);
    let (api_geometry, api_context) = geometry(api);
    fs::write(root.join("native.geometry.gcode"), &native_geometry).unwrap();
    fs::write(root.join("api.geometry.gcode"), &api_geometry).unwrap();
    fs::write(
        root.join("native.geometry-context.json"),
        serde_json::to_vec_pretty(&native_context).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("api.geometry-context.json"),
        serde_json::to_vec_pretty(&api_context).unwrap(),
    )
    .unwrap();
    let difference =
        crate::golden::first_difference(native_geometry.as_bytes(), api_geometry.as_bytes());
    let strict = crate::golden::compare_ordered_bytes_generator_only(native, api);
    report.as_object_mut().unwrap().extend(
        json!({
            "native_sha256": sha(native),
            "api_sha256": sha(api),
            "native_geometry_sha256": sha(native_geometry.as_bytes()),
            "api_geometry_sha256": sha(api_geometry.as_bytes()),
            "native_geometry_records": native_context.len(),
            "api_geometry_records": api_context.len(),
            "strict_comparison": strict,
            "geometry_difference": difference.as_ref().map(ToString::to_string),
        })
        .as_object()
        .unwrap()
        .clone(),
    );
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(
        !native_context.is_empty(),
        "{root:?}: Native geometry must be exercised"
    );
    assert!(
        native_context
            .iter()
            .any(|row| row["role"] == "Sparse infill"),
        "{root:?}: Native sparse infill must be exercised"
    );
    if let Some(difference) = difference {
        panic!("{root:?}: geometry-only mismatch: {difference}");
    }
}

#[test]
fn three_d_honeycomb_centered_plater_geometry_matches_native() {
    run("honeycomb-centered", json!({}), false);
}

#[test]
fn three_d_honeycomb_moved_plater_geometry_matches_native() {
    run("honeycomb-moved", json!({}), true);
}

#[test]
fn three_d_honeycomb_dense_plater_geometry_matches_native() {
    run(
        "honeycomb-dense",
        json!({"sparse_infill_density": "60%"}),
        false,
    );
}

#[test]
fn three_d_honeycomb_multiline_plater_geometry_matches_native() {
    run("honeycomb-multiline", json!({"fill_multiline": "3"}), false);
}

#[test]
fn three_d_honeycomb_complete_top_plater_geometry_matches_native() {
    run(
        "honeycomb-complete-top",
        json!({"infill_complete_top": "1"}),
        false,
    );
}

#[test]
fn three_d_honeycomb_open_top_plater_geometry_matches_native() {
    run(
        "honeycomb-open-top",
        json!({"infill_complete_top": "0"}),
        false,
    );
}

#[test]
fn three_d_honeycomb_smoothed_plater_geometry_matches_native() {
    run(
        "honeycomb-smoothed",
        json!({"sparse_infill_smooth_factor": "35%"}),
        false,
    );
}
