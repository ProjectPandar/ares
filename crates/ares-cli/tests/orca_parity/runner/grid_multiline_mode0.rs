//! FillRectilinear.cpp:3048-3162,3344-3384,3422-3446; Fill.cpp:1489-1497.
//! Authored STL inputs; Native streams/exported projects remain runtime artifacts.
#[path = "grid_surface_collections.rs"]
mod surface_collections;

use std::{fs, process::Command};

use serde_json::json;

use super::{CaseInputs, OrcaRunner, ParityCase, application, repo_root};
use crate::{presets::VendorProfiles, runner};

#[path = "grid_multiline_mode0_inputs.rs"]
mod inputs;
use crate::option_coverage::multiline_union::observation;

#[derive(Clone, Copy)]
struct Case {
    count: &'static str,
    angle: &'static str,
    density: &'static str,
    hole: bool,
    combined: bool,
}

macro_rules! control {
    ($name:ident, $label:literal, $count:literal, $angle:literal, $density:literal, $hole:literal, $combined:literal) => {
        #[test]
        fn $name() {
            compare(
                $label,
                Case {
                    count: $count,
                    angle: $angle,
                    density: $density,
                    hole: $hole,
                    combined: $combined,
                },
            );
        }
    };
}

control!(
    grid_mode0_count1_control,
    "G1",
    "1",
    "45",
    "30%",
    false,
    false
);
control!(grid_mode0_count2, "G2", "2", "45", "30%", false, false);
control!(grid_mode0_count3, "G3", "3", "45", "30%", false, false);
control!(grid_mode0_count10, "G10", "10", "45", "30%", false, false);
control!(grid_mode0_angle0, "G2-A0", "2", "0", "30%", false, false);
control!(
    grid_mode0_density20,
    "G2-D20",
    "2",
    "45",
    "20%",
    false,
    false
);
control!(
    grid_mode0_hole_count1_control,
    "H1",
    "1",
    "45",
    "30%",
    true,
    false
);
control!(grid_mode0_hole_count2, "H2", "2", "45", "30%", true, false);
control!(
    grid_mode0_combined_two_layers,
    "G2-C2",
    "2",
    "45",
    "30%",
    false,
    true
);

fn compare(label: &str, case: Case) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let baseline = inputs::reference(
        &runner,
        &format!("{label}/count1"),
        Case { count: "1", ..case },
    );
    observation::require_sparse(&observation::observe(&baseline.reference));
    if case.hole {
        observation::require_hole_clipping(&observation::observe(&baseline.reference));
    }
    let baseline_root = evaluate(&baseline);
    require_equal(&baseline_root);
    if case.count == "1" {
        return;
    }
    let active = inputs::reference(&runner, label, case);
    let native = observation::observe(&active.reference);
    observation::require_sparse(&native);
    if case.hole {
        observation::require_hole_clipping(&native);
    }
    if case.combined {
        assert!(
            has_combined_sparse(&active.reference),
            "Native must extrude combined sparse layers"
        );
    }
    assert_ne!(
        sparse_commands(&baseline.reference),
        sparse_commands(&active.reference),
        "{label}: multiline must change real Native sparse motion"
    );
    let root = evaluate(&active);
    fs::write(
        root.join("native.sparse.json"),
        serde_json::to_vec_pretty(&native).unwrap(),
    )
    .unwrap();
    require_equal(&root);
}

fn evaluate(case: &ParityCase) -> std::path::PathBuf {
    let root = tempfile::Builder::new()
        .prefix("grid-mode0-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &case.project).unwrap();
    fs::write(root.join("native.gcode"), &case.reference).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = runtime.block_on(ares_core::slice_project_plate_validated(
        &case.project,
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(bytes) = &result {
        fs::write(root.join("ares.gcode"), bytes).unwrap();
    }
    let api_comparison = result
        .as_ref()
        .map_err(|error| format!("{error:?}"))
        .and_then(|bytes| {
            crate::golden::compare_ordered_bytes_generator_only(&case.reference, bytes)
        });
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            crate::golden::compare_ordered_bytes_generator_only(&case.reference, bytes)
        });
    let api_layers_equal = result
        .as_ref()
        .is_ok_and(|bytes| layer_lines(bytes) == layer_lines(&case.reference));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":case.label, "input_sha256":runner::cache_identity::raw_digest(&case.project),
        "native_sha256":runner::cache_identity::raw_digest(&case.reference),
        "native_lines":line_count(&case.reference),
        "api_admitted":result.is_ok(), "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
        "api_comparison":api_comparison, "api_layers_equal":api_layers_equal,
        "api_lines":result.as_ref().ok().map(|b|line_count(b)),
        "cli_code":cli.status.code(), "cli_comparison":cli_comparison,
        "cli_lines":cli_bytes.as_ref().ok().map(|b|line_count(b)),
    })).unwrap()).unwrap();
    root
}

fn require_equal(root: &std::path::Path) {
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("comparison.json")).unwrap()).unwrap();
    assert_eq!(result["api_admitted"], true, "{root:?}: {result}");
    assert_eq!(result["cli_code"], 0, "{root:?}: {result}");
    assert_eq!(result["api_layers_equal"], true, "{root:?}: {result}");
    assert_eq!(
        result["api_lines"], result["native_lines"],
        "{root:?}: {result}"
    );
    assert_eq!(
        result["cli_lines"], result["native_lines"],
        "{root:?}: {result}"
    );
    assert_eq!(
        result["api_comparison"],
        json!({"Ok":null}),
        "{root:?}: {result}"
    );
    assert_eq!(
        result["cli_comparison"],
        json!({"Ok":null}),
        "{root:?}: {result}"
    );
}

fn line_count(bytes: &[u8]) -> usize {
    std::str::from_utf8(bytes).unwrap().lines().count()
}

fn layer_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
        .collect()
}

fn sparse_commands(bytes: &[u8]) -> Vec<&str> {
    let mut sparse = false;
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                sparse = role == "Sparse infill";
            }
            sparse
                && line.starts_with("G1 ")
                && line.contains('E')
                && (line.contains('X') || line.contains('Y'))
        })
        .collect()
}

fn has_combined_sparse(bytes: &[u8]) -> bool {
    let mut height = 0.0_f64;
    let mut sparse = false;
    std::str::from_utf8(bytes).unwrap().lines().any(|line| {
        if let Some(value) = line.strip_prefix(";HEIGHT:") {
            height = value.parse().unwrap();
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            sparse = role == "Sparse infill";
        }
        sparse
            && height == 0.4
            && line.starts_with("G1 ")
            && line.contains('E')
            && (line.contains('X') || line.contains('Y'))
    })
}
