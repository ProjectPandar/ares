//! Outer brim occupancy/ownership (`Print.cpp:2794-2918`; `GCode.cpp:4486-4540,5334-5335`).

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn per_object_skirt_brim_far_control() {
    check("far-control", 2, false);
}
#[test]
fn per_object_skirt_brim_far() {
    check("far-brim", 2, true);
}
#[test]
fn per_object_skirt_brim_near_control() {
    check("near-control", 2, false);
}
#[test]
fn per_object_skirt_brim_near_merges() {
    check("near-brim", 1, true);
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str, expected_groups: usize, has_brim: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/per-object-skirt-brim");
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
        ("brim_type", if has_brim { "outer_only" } else { "no_brim" }),
        ("brim_width", if has_brim { "8" } else { "0" }),
        ("combine_brims", "0"),
        ("brim_object_gap", "0"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
    ] {
        assert_eq!(settings[key], value);
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-skirt-brim-{label}-"))
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
            "source":"Print.cpp2794-2918; GCode.cpp4486-4540,5334-5335"
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
    assert!(text.contains("; skirt_type = perobject\n"));
    assert!(text.contains("; skirt_loops = 2\n"));
    let blocks = text.split(";TYPE:Skirt\n").skip(1).collect::<Vec<_>>();
    assert_eq!(blocks.len(), expected_groups, "Native groups {root:?}");
    let bounds = positive_bounds(text, "Skirt", "; skirt");
    let brim_blocks = text.split(";TYPE:Brim\n").skip(1).count();
    assert_eq!(
        brim_blocks,
        if has_brim { 2 } else { 0 },
        "independent brim emission {root:?}"
    );
    let brim_bounds = positive_bounds(text, "Brim", "; brim");
    if has_brim {
        assert!(text.find(";TYPE:Skirt\n").unwrap() < text.find(";TYPE:Brim\n").unwrap());
    }
    if label.starts_with("far-") {
        assert!(bounds[0][2] < bounds[1][0] || bounds[1][2] < bounds[0][0]);
    }
    fs::write(
        root.join("witness.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"input_sha256":sha(&bytes),"native_sha256":sha(&reference),
            "skirt_groups":blocks.len(),"positive_xy_extrusion_bounds":bounds,
            "brim_blocks":brim_blocks,"brim_positive_xy_bounds":brim_bounds
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

fn positive_bounds(text: &str, role: &str, suffix: &str) -> Vec<[f64; 4]> {
    text.split(&format!(";TYPE:{role}\n"))
        .skip(1)
        .map(|block| {
            let points = block
                .split(";TYPE:")
                .next()
                .unwrap()
                .lines()
                .filter_map(|line| {
                    if !line.starts_with("G1 ") || !line.ends_with(suffix) {
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
            assert!(!points.is_empty(), "real {role} XY extrusion");
            [
                points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min),
                points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
                points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max),
                points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
            ]
        })
        .collect()
}
