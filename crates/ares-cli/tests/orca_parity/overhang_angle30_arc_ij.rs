//! X1C printable-angle 30° public fixture for retained arc-center replay.
//!
//! Native and Ares have the same fitting vertices and local circle. At Z11.6,
//! origin=(128,128), extruder_offset=(0,2), folding those offsets changes the
//! mapped center Y by one ULP and prints J2.174 instead of Native's J2.175.
//! `GCode.cpp:8124-8128` maps `(unscale(p) + origin) - extruder_offset`;
//! `GCode.cpp:7085` maps center and start separately before subtracting them.
//! Scope: retained center mapping only, not cone geometry, fitting or formatting.

use crate::{OrcaRunner, golden, runner::stages};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn overhang_angle30_arc_ij() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let fixture =
        crate::runner::repo_root().join("tests/parity/overhang-angle30-arc-ij/witness.3mf");
    let bytes = fs::read(fixture).unwrap();
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(
        digest,
        "37ec57a0549347be48bbe9ef8d9ce9bfd1f676c6c8c1ef5d7da2737463258796"
    );
    let root = tempfile::Builder::new()
        .prefix("overhang-angle30-arc-ij-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    stages::run(
        runner.bin(),
        vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            root.join("native").to_str().unwrap().into(),
            root.join("input.3mf").to_str().unwrap().into(),
        ],
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    let result = serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join("native/result.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let native = fs::read(root.join("native/plate_1.gcode")).unwrap();
    assert!(
        String::from_utf8_lossy(&native)
            .lines()
            .any(|line| line == "G2 X113.993 Y124.287 I-2.468 J2.175 E.03566"),
        "Native witness arc {root:?}"
    );
    let output = root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "CLI admission {root:?}: {cli:?}");
    let cli_bytes = fs::read(output).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_or_else(|error| panic!("public API admission {root:?}: {error:?}"));
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    assert_eq!(api, cli_bytes, "public API/CLI raw equality {root:?}");
    golden::compare_ordered_bytes_generator_only(&native, &api)
        .unwrap_or_else(|error| panic!("API strict comparison {root:?}: {error}"));
    golden::compare_ordered_bytes_generator_only(&native, &cli_bytes)
        .unwrap_or_else(|error| panic!("CLI strict comparison {root:?}: {error}"));
}
