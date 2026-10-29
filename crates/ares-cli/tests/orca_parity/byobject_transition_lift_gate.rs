//! GCode.cpp:3326-3327; GCode.cpp:7432-7462; GCode.cpp:7483; GCodeWriter.cpp:626-648.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn orca_parity_byobject_transition_lift_gate_open() {
    check("gate-open");
}

#[test]
fn orca_parity_byobject_transition_lift_gate_band() {
    check("gate-band");
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
    let fixture = crate::runner::repo_root().join("tests/parity/byobject-transition-lift-gate");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let lift_above = manifest["cases"][label]["retract_lift_above"]
        .as_str()
        .unwrap();
    let lift_below = manifest["cases"][label]["retract_lift_below"]
        .as_str()
        .unwrap();
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("print_sequence", "by object"),
        ("spiral_mode", "0"),
        ("gcode_comments", "1"),
    ] {
        assert_eq!(settings[key], value);
    }
    assert_eq!(settings["z_hop"][0], "0.4");
    assert_eq!(settings["z_hop_types"][0], "Normal Lift");
    assert_eq!(settings["retraction_length"][0], "4");
    assert_eq!(settings["retract_lift_above"][0], lift_above);
    assert_eq!(settings["retract_lift_below"][0], lift_below);
    assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 1);
    let root = tempfile::Builder::new()
        .prefix(&format!("byobject-lift-gate-{label}-"))
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
            "input_sha256":sha(&bytes), "args":args,
            "source":"GCode.cpp:3326-3327; GCode.cpp:7432-7462; GCode.cpp:7483; GCodeWriter.cpp:626-648"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let native = std::str::from_utf8(&reference).unwrap();
    let lines: Vec<_> = native.lines().collect();
    let origin = lines
        .iter()
        .position(|line| line.ends_with("; move to origin position for next object"))
        .unwrap();
    if label == "gate-open" {
        // The retract deferred a hop (band disabled): branch 1 raises the
        // destination to `m_pos(2) + m_to_lift` and materializes the lift.
        assert_eq!(lines[origin - 2], "G1 Z4.4 F9000");
        assert_eq!(lines[origin - 1], "G1 Z4.8 F9000 ; normal lift Z");
        assert_eq!(
            lines[origin],
            "G1 X140 Y110 Z4.8 F9000 ; move to origin position for next object"
        );
    } else {
        // Z 4.0 (retract) and Z 4.4 (post safety lift) sit above the
        // upper boundary (3): `lazy_lift` never defers, so the transition
        // window carries no lift line and the travel keeps the finished
        // object's nominal Z. Intra-object hops inside the band keep their
        // own `normal lift Z` lines elsewhere.
        assert_eq!(lines[origin - 1], "G1 Z4.4 F9000");
        assert_eq!(
            lines[origin],
            "G1 X140 Y110 Z4 F9000 ; move to origin position for next object"
        );
        assert!(
            !lines[origin - 4..origin]
                .iter()
                .any(|line| line.contains("; normal lift Z"))
        );
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    let cli_bytes = fs::read(&output).unwrap();
    let api_comparison = golden::compare_ordered_bytes_generator_only(&reference, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "raw_api_cli_equal":api==cli_bytes,"cli_code":cli.status.code()
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI {root:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}
