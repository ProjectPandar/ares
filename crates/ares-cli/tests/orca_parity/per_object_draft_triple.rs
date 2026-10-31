//! Divergent draft compositions (`Print.cpp:2703-2778,2794-2819`; `GCode.cpp:4300-4540`).

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn per_object_draft_triple_far_triple_all() {
    check("far-triple-all", 2, 2, false);
}

#[test]
fn per_object_draft_triple_far_triple_single() {
    check("far-triple-single", 2, 2, true);
}

#[test]
fn per_object_draft_triple_near_triple_all() {
    check("near-triple-all", 1, 2, false);
}

#[test]
fn per_object_draft_triple_near_triple_single() {
    check("near-triple-single", 1, 2, true);
}

#[test]
fn per_object_draft_triple_far_all_no_brim_2() {
    check("far-all-no-brim-2", 2, 1, false);
}

#[test]
fn per_object_draft_triple_far_all_no_brim_3() {
    check("far-all-no-brim-3", 2, 1, false);
}

#[test]
fn per_object_draft_triple_far_single_no_brim_2() {
    check("far-single-no-brim-2", 2, 1, true);
}

#[test]
fn per_object_draft_triple_far_single_no_brim_3() {
    check("far-single-no-brim-3", 2, 1, true);
}

#[test]
fn per_object_draft_triple_near_all_no_brim_2() {
    check("near-all-no-brim-2", 1, 1, false);
}

#[test]
fn per_object_draft_triple_near_all_no_brim_3() {
    check("near-all-no-brim-3", 1, 1, false);
}

#[test]
fn per_object_draft_triple_near_single_no_brim_2() {
    check("near-single-no-brim-2", 1, 1, true);
}

#[test]
fn per_object_draft_triple_near_single_no_brim_3() {
    check("near-single-no-brim-3", 1, 1, true);
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str, expected_groups: usize, brim_owners: usize, single: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/per-object-draft-triple");
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
        ("min_skirt_length", "20"),
        ("draft_shield", "enabled"),
        ("single_loop_draft_shield", if single { "1" } else { "0" }),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("brim_type", "outer_only"),
        ("brim_width", "8"),
        ("combine_brims", "0"),
        ("brim_object_gap", "0"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
    ] {
        assert_eq!(settings[key], value);
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-draft-triple-{label}-"))
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
            "source":"Print.cpp2703-2778,2794-2819; GCode.cpp4300-4540,5334-5335"
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
    assert!(text.lines().any(|line| line.starts_with("M83")));
    assert!(!text.lines().any(|line| line.starts_with("M82")));
    let layers = native_layers(text);
    assert_eq!(layers.len(), 20, "real shield lifetime {root:?}");
    assert_eq!(layers.first().unwrap().z, 0.2);
    assert_eq!(layers.last().unwrap().z, 4.0);
    assert_eq!(
        layers[0].brim.len(),
        brim_owners,
        "independent first-layer brims {root:?}"
    );
    assert!(layers.iter().skip(1).all(|layer| layer.brim.is_empty()));
    let first = &layers[0].groups;
    // Fact for these fixed Native20mm inputs, not a generic clipped-E threshold.
    assert!(first.iter().all(|group| group.relative_e_mm > 20.0));
    if single {
        assert!(
            layers
                .iter()
                .skip(1)
                .flat_map(|layer| &layer.groups)
                .all(|group| group.relative_e_mm < 20.0),
            "no per-layer budget top-up {root:?}"
        );
    }
    for (index, layer) in layers.iter().enumerate() {
        assert_eq!(
            layer.groups.len(),
            expected_groups,
            "once per group/Z {root:?}"
        );
        if index > 0 && single {
            for (group, initial) in layer.groups.iter().zip(first) {
                assert!(
                    group.bounds[0] > initial.bounds[0] && group.bounds[2] < initial.bounds[2],
                    "own innermost loop {root:?}"
                );
                assert!(group.relative_e_mm < initial.relative_e_mm);
            }
        }
    }
    fs::write(root.join("witness.json"), serde_json::to_vec_pretty(&json!({
        "case":label,"input_sha256":sha(&bytes),"native_sha256":sha(&reference),
        "minimum":20,"brim_owners":brim_owners,"single":single,"groups_per_z":expected_groups,"skirt_z_count":layers.len(),"skirt_layers":layers
    })).unwrap()).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
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

#[derive(serde::Serialize)]
struct LayerMotion {
    z: f64,
    groups: Vec<GroupMotion>,
    brim: Vec<GroupMotion>,
}

fn native_layers(text: &str) -> Vec<LayerMotion> {
    text.split(";LAYER_CHANGE\n")
        .skip(1)
        .filter_map(|chunk| {
            let mut groups = native_groups(chunk, "Skirt");
            if groups.is_empty() {
                return None;
            }
            groups.sort_by(|a, b| a.bounds[0].total_cmp(&b.bounds[0]));
            let z = chunk
                .lines()
                .find_map(|line| line.strip_prefix(";Z:")?.parse::<f64>().ok())
                .unwrap();
            Some(LayerMotion {
                z,
                groups,
                brim: native_groups(chunk, "Brim"),
            })
        })
        .collect()
}

fn native_groups(text: &str, role: &str) -> Vec<GroupMotion> {
    text.split(&format!(";TYPE:{role}\n"))
        .skip(1)
        .map(|block| native_group(block, role))
        .collect()
}

fn native_group(block: &str, role: &str) -> GroupMotion {
    let suffix = format!("; {}", role.to_lowercase());
    let points = block
        .split(";TYPE:")
        .next()
        .unwrap()
        .lines()
        .filter_map(|line| {
            if !line.starts_with("G1 ") || !line.ends_with(&suffix) {
                return None;
            }
            let coordinate = |prefix| {
                line.split_whitespace()
                    .find_map(|word: &str| word.strip_prefix(prefix)?.parse::<f64>().ok())
            };
            let e = coordinate('E')?;
            (e > 0.0).then(|| (coordinate('X').unwrap(), coordinate('Y').unwrap(), e))
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
