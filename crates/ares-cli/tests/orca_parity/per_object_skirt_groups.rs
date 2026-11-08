//! `Print.cpp:2784-2996` group membership and `GCode.cpp:4465-4484` emission.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn per_object_skirt_disjoint() {
    check("disjoint");
}

#[test]
fn per_object_skirt_overlap() {
    check("overlap");
}

#[test]
fn per_object_skirt_transitive() {
    check("transitive");
}

#[test]
fn per_object_skirt_copies() {
    check("copies");
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
    let fixture = crate::runner::repo_root().join("tests/parity/per-object-skirt");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("skirt_type", "perobject"),
        ("skirt_loops", "2"),
        ("skirt_height", "1"),
        ("skirt_distance", "6"),
        ("min_skirt_length", "0"),
        ("draft_shield", "disabled"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("brim_type", "no_brim"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
    ] {
        assert_eq!(settings[key], value);
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-skirt-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native-first")).unwrap();
    fs::create_dir(root.join("native-second")).unwrap();
    let slice_args = |outputdir: &str| {
        vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            root.join(outputdir).to_str().unwrap().into(),
            root.join("input.3mf").to_str().unwrap().into(),
        ]
    };
    let native_binary = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "args":slice_args("native-first"),"input_sha256":sha(&bytes),
            "stability_double_run":"native-second",
            "source":"Print.cpp:4231-4400; GCode.cpp:5991-6082"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(
        runner.bin(),
        slice_args("native-first"),
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    stages::run(
        runner.bin(),
        slice_args("native-second"),
        stages::Stage::Slice,
        &root,
    )
    .unwrap();
    // Stability premise: the Native producer must be deterministic on
    // this input under the generator-line normalization the acceptance
    // comparison uses.
    let normalize = |raw: &[u8]| {
        golden::normalize_one_generator_line(raw, golden::GeneratorKind::Orca)
            .unwrap_or_else(|error| panic!("Native generator line: {error}"))
    };
    let first = fs::read(root.join("native-first/plate_1.gcode")).unwrap();
    let second = fs::read(root.join("native-second/plate_1.gcode")).unwrap();
    assert_eq!(
        normalize(&first),
        normalize(&second),
        "Native slice is not deterministic"
    );
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native-first/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let reference = first;
    let text = std::str::from_utf8(&reference).unwrap();
    assert!(text.contains("; skirt_type = perobject\n"));
    assert!(text.contains("; skirt_loops = 2\n"));
    let blocks = text.split(";TYPE:Skirt\n").skip(1).collect::<Vec<_>>();
    // No frozen Native group-count premise: the beta oracle emits 3
    // per-instance groups on the copies fixture where this froze 1; the
    // group count is compared only as part of the whole-output comparison
    // below, never waived on its own.
    let bounds = blocks
        .iter()
        .map(|block| {
            let points = block
                .split(";TYPE:")
                .next()
                .unwrap()
                .lines()
                .filter_map(|line| {
                    if !line.starts_with("G1 ") || !line.ends_with("; skirt") {
                        return None;
                    }
                    let coordinate = |prefix| {
                        line.split_whitespace()
                            .find_map(|word: &str| word.strip_prefix(prefix)?.parse::<f64>().ok())
                    };
                    let e = coordinate('E')?;
                    (e > 0.0).then(|| (coordinate('X').unwrap(), coordinate('Y').unwrap()))
                })
                .collect::<Vec<_>>();
            assert!(!points.is_empty(), "real skirt extrusion {root:?}");
            [
                points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min),
                points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
                points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max),
                points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
            ]
        })
        .collect::<Vec<_>>();
    if label == "disjoint" {
        assert!(bounds[0][2] < bounds[1][0] || bounds[1][2] < bounds[0][0]);
    }
    fs::write(
        root.join("witness.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"input_sha256":sha(&bytes),"native_sha256":sha(&reference),
            "skirt_groups":blocks.len(),"positive_xy_extrusion_bounds":bounds
        }))
        .unwrap(),
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
        // Per-instance grouping premise: ares must emit exactly as many
        // ;TYPE:Skirt blocks as the Native reference produced in this run
        // (runtime cross-producer equality, not a frozen count).
        assert_eq!(
            std::str::from_utf8(gcode)
                .unwrap()
                .matches(";TYPE:Skirt\n")
                .count(),
            blocks.len(),
            "ares per-instance skirt grouping {root:?}"
        );
    }
    let api_comparison = api
        .as_ref()
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    let output = root.join("ares.cli.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = crate::runner::application::CliCommand::new(binary)
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
