//! Group-local shield lifetime/inner loop (`Print.cpp:575-586,2646-2696`; `GCode.cpp:4300-4452`).

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read},
    process::Command,
};

#[test]
fn per_object_draft_shield_disjoint_disabled() {
    check("disjoint-disabled", false, false);
}

#[test]
fn per_object_draft_shield_disjoint_all() {
    check("disjoint-all", true, false);
}

#[test]
fn per_object_draft_shield_disjoint_single() {
    check("disjoint-single", true, true);
}

#[test]
fn per_object_draft_shield_overlap_disabled() {
    check("overlap-disabled", false, false);
}

#[test]
fn per_object_draft_shield_overlap_all() {
    check("overlap-all", true, false);
}

#[test]
fn per_object_draft_shield_overlap_single() {
    check("overlap-single", true, true);
}

#[test]
fn per_object_draft_shield_transitive_disabled() {
    check("transitive-disabled", false, false);
}

#[test]
fn per_object_draft_shield_transitive_all() {
    check("transitive-all", true, false);
}

#[test]
fn per_object_draft_shield_transitive_single() {
    check("transitive-single", true, true);
}

#[test]
fn per_object_draft_shield_copies_disabled() {
    check("copies-disabled", false, false);
}

#[test]
fn per_object_draft_shield_copies_all() {
    check("copies-all", true, false);
}

#[test]
fn per_object_draft_shield_copies_single() {
    check("copies-single", true, true);
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str, enabled: bool, single: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/per-object-draft-shield");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let touching = manifest["cases"][label]["touching"].as_bool().unwrap();
    // Upstream groups skirts per object instance (Print.cpp:4176-4192) and only
    // merges instances whose expanded skirts intersect (Print.cpp:4252-4300).
    let items = {
        let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let mut model = String::new();
        archive
            .by_name("3D/3dmodel.model")
            .unwrap()
            .read_to_string(&mut model)
            .unwrap();
        model.matches("<item ").count()
    };
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("skirt_type", "perobject"),
        ("skirt_loops", "2"),
        ("skirt_height", "1"),
        ("skirt_distance", "6"),
        ("min_skirt_length", "0"),
        ("draft_shield", if enabled { "enabled" } else { "disabled" }),
        ("single_loop_draft_shield", if single { "1" } else { "0" }),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("brim_type", "no_brim"),
        ("enable_prime_tower", "0"),
        ("print_sequence", "by layer"),
    ] {
        assert_eq!(settings[key], value);
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("per-object-draft-shield-{label}-"))
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
            "source":"Print.cpp575-586,2646-2696; GCode.cpp4300-4452,4456-4484"
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
    assert_eq!(
        layers.len(),
        if enabled { 20 } else { 1 },
        "real shield lifetime {root:?}"
    );
    assert_eq!(layers.first().unwrap().z, 0.2);
    assert_eq!(layers.last().unwrap().z, if enabled { 4.0 } else { 0.2 });
    let first = &layers[0].groups;
    for (index, layer) in layers.iter().enumerate() {
        if touching {
            assert!(
                (1..items).contains(&layer.groups.len()),
                "touching instances merge into fewer than {items} groups {root:?}"
            );
        } else {
            assert_eq!(
                layer.groups.len(),
                items,
                "one shield per instance {root:?}"
            );
        }
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
        "enabled":enabled,"single":single,"groups_per_z":items,"items":items,"touching":touching,"skirt_z_count":layers.len(),"skirt_layers":layers
    })).unwrap()).unwrap();
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

#[derive(serde::Serialize)]
struct LayerMotion {
    z: f64,
    groups: Vec<GroupMotion>,
}

fn native_layers(text: &str) -> Vec<LayerMotion> {
    text.split(";LAYER_CHANGE\n")
        .skip(1)
        .filter_map(|chunk| {
            let mut groups = chunk
                .split(";TYPE:Skirt\n")
                .skip(1)
                .map(native_group)
                .collect::<Vec<_>>();
            if groups.is_empty() {
                return None;
            }
            groups.sort_by(|a, b| a.bounds[0].total_cmp(&b.bounds[0]));
            let z = chunk
                .lines()
                .find_map(|line| line.strip_prefix(";Z:")?.parse::<f64>().ok())
                .unwrap();
            Some(LayerMotion { z, groups })
        })
        .collect()
}

fn native_group(block: &str) -> GroupMotion {
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
            (e > 0.0).then(|| (coordinate('X').unwrap(), coordinate('Y').unwrap(), e))
        })
        .collect::<Vec<_>>();
    assert!(!points.is_empty(), "real shield XY extrusion");
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
