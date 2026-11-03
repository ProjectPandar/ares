//! Brim.cpp:55-62,109-118,462-513: post-EFC region polygon consumer.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn brim_efc_outer_efc0_outline0() {
    check("outer-efc0-outline0");
}

#[test]
fn brim_efc_outer_efc0_outline1() {
    check("outer-efc0-outline1");
}

#[test]
fn brim_efc_outer_efc0p3_outline0() {
    check("outer-efc0p3-outline0");
}

#[test]
fn brim_efc_outer_efc0p3_outline1() {
    check("outer-efc0p3-outline1");
}

#[test]
fn brim_efc_inner_efc0_outline0() {
    check("inner-efc0-outline0");
}

#[test]
fn brim_efc_inner_efc0_outline1() {
    check("inner-efc0-outline1");
}

#[test]
fn brim_efc_inner_efc0p3_outline0() {
    check("inner-efc0p3-outline0");
}

#[test]
fn brim_efc_inner_efc0p3_outline1() {
    check("inner-efc0p3-outline1");
}

#[test]
fn brim_efc_both_efc0_outline0() {
    check("both-efc0-outline0");
}

#[test]
fn brim_efc_both_efc0_outline1() {
    check("both-efc0-outline1");
}

#[test]
fn brim_efc_both_efc0p3_outline0() {
    check("both-efc0p3-outline0");
}

#[test]
fn brim_efc_both_efc0p3_outline1() {
    check("both-efc0p3-outline1");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/brim-consumes-efc-region-polygons");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let settings = crate::runner::application::settings(&bytes).unwrap();
    let case = &manifest["cases"][label];
    assert_eq!(settings["elefant_foot_compensation"], case["efc"]);
    assert_eq!(settings["brim_use_efc_outline"], case["outline"]);
    assert_eq!(settings["wall_sequence"], "outer wall/inner wall");
    let root = tempfile::Builder::new()
        .prefix(&format!("brim-efc-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    let native_binary = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "args":args,"input_sha256":sha(&bytes),
            "source":"Brim.cpp55-62,109-118,462-513; GCode.cpp4456-4540,5334-5335"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    for key in [
        "brim_type",
        "brim_use_efc_outline",
        "elefant_foot_compensation",
    ] {
        assert!(text.contains(&format!("; {key} = {}\n", settings[key].as_str().unwrap())));
    }
    assert!(text.lines().any(|line| line.starts_with("M83")));
    assert!(!text.lines().any(|line| line.starts_with("M82")));
    let first_brim = std::str::from_utf8(&reference)
        .unwrap()
        .split(";TYPE:Brim\n")
        .nth(1)
        .unwrap();
    let motion = native_group(first_brim, "Brim");
    fs::write(
        root.join("witness.json"),
        serde_json::to_vec_pretty(&json!({"case":label,"native_brim":motion})).unwrap(),
    )
    .unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let api_comparison = api
        .as_ref()
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    let output = root.join("ares.cli.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|e| format!("CLI: {e}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
            "api_comparison":api_comparison,"cli_code":cli.status.code(),
            "cli_comparison":cli_comparison,"cli_binary_sha256":sha(&fs::read(binary).unwrap()),
            "strict_parity_required":true
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(api.is_ok(), "public API admission RED {root:?}: {api:?}");
    assert!(cli.status.success(), "CLI admission RED {root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    cli_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    assert_eq!(
        api.unwrap(),
        cli_bytes.unwrap(),
        "API/CLI raw equality {root:?}"
    );
}

#[derive(serde::Serialize)]
struct GroupMotion {
    bounds: [f64; 4],
    positive_xy_segments: usize,
    relative_e_mm: f64,
}

fn native_group(block: &str, role: &str) -> GroupMotion {
    let points = block
        .split(";TYPE:")
        .next()
        .unwrap()
        .lines()
        .filter_map(|line| {
            if !line.starts_with("G1 ") {
                return None;
            }
            let coordinate = |prefix| {
                line.split_whitespace()
                    .find_map(|word: &str| word.strip_prefix(prefix)?.parse::<f64>().ok())
            };
            let (x, y, e) = (coordinate('X')?, coordinate('Y')?, coordinate('E')?);
            (e > 0.0).then_some((x, y, e))
        })
        .collect::<Vec<_>>();
    assert!(!points.is_empty(), "real {role} XY extrusion");
    GroupMotion {
        bounds: [
            points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min),
            points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
            points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max),
            points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
        ],
        positive_xy_segments: points.len(),
        relative_e_mm: points.iter().map(|p| p.2).sum(),
    }
}
