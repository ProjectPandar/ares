//! PrintObjectSlice.cpp:401-472 / Layer.cpp:183-260 / Fill.cpp:1016-1049,1213-1356.
//! Frozen authored modifier inputs; Native output is runtime-only via OrcaRunner.
use super::{ExportedCase, OrcaRunner, ParityCase, repo_root};
use crate::runner;
use serde_json::json;
use std::{fs, process::Command};

#[test]
fn adaptive_without_modifier_matches_complete_native() {
    compare(
        "A-C0",
        "d6ed45dca46d2b9f7a395df577083f37fb3f747e89b69be70eb956b9868484dd",
    );
}

#[test]
fn adaptive_noop_modifier_preserves_complete_native_sparse_output() {
    compare(
        "A-C1",
        "7fd8549eacaff99b71fe3b5eaea58d365a545d831174255cdc2bec1a69005619",
    );
}

#[test]
fn adaptive_outside_modifier_preserves_complete_native_sparse_output() {
    compare(
        "A-C2",
        "e117163a8c090d4a5611e20912cbdadd8f100ee4c16b2338ce63e65109b7c720",
    );
}

fn compare(label: &str, expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = fs::read(
        repo_root()
            .join("tests/parity/modifier-sparse-loss")
            .join(format!("{label}.3mf")),
    )
    .unwrap();
    assert_eq!(runner::cache_identity::raw_digest(&project), expected_sha);
    let project_path = runner.work.join(format!("{label}.3mf"));
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let active = runner
        .slice_case(&ExportedCase {
            label: format!("modifier-sparse-loss/{label}"),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let root = evaluate(&active);
    assert!(
        role_extrusions(&active.reference, "Sparse infill") > 0,
        "{root:?}: Native must emit sparse extrusion"
    );
    require_equal(&root);
}

fn role_extrusions(bytes: &[u8], target: &str) -> usize {
    let mut active = false;
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                active = role == target;
            }
            active
                && line.starts_with("G1 ")
                && (line.contains('X') || line.contains('Y'))
                && line
                    .split_whitespace()
                    .find_map(|word| word.strip_prefix('E'))
                    .and_then(|number| number.parse::<f64>().ok())
                    .is_some_and(|e| e > 0.0)
        })
        .count()
}

fn evaluate(case: &ParityCase) -> std::path::PathBuf {
    let root = tempfile::Builder::new()
        .prefix("modifier-sparse-loss-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &case.project).unwrap();
    fs::write(root.join("native.gcode"), &case.reference).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = runtime
        .block_on(ares_core::slice_project_plate_validated(
            &case.project,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .map_err(|error| format!("{error:?}"));
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
    let cli_layers_equal = cli_bytes
        .as_ref()
        .is_ok_and(|bytes| layer_lines(bytes) == layer_lines(&case.reference));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":case.label, "input_sha256":runner::cache_identity::raw_digest(&case.project),
        "native_sha256":runner::cache_identity::raw_digest(&case.reference),
        "native_lines":line_count(&case.reference),
        "native_positive_sparse_xy_e":role_extrusions(&case.reference,"Sparse infill"),
        "api_positive_sparse_xy_e":result.as_ref().ok().map(|b|role_extrusions(b,"Sparse infill")),
        "cli_positive_sparse_xy_e":cli_bytes.as_ref().ok().map(|b|role_extrusions(b,"Sparse infill")),
        "debug_assertions":cfg!(debug_assertions),
        "api_admitted":result.is_ok(), "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
        "api_comparison":api_comparison, "api_layers_equal":api_layers_equal,
        "api_lines":result.as_ref().ok().map(|b|line_count(b)),
        "api_cli_raw_equal":result.as_ref().ok().zip(cli_bytes.as_ref().ok()).is_some_and(|(a,c)|a==c),
        "cli_code":cli.status.code(), "cli_comparison":cli_comparison,
        "cli_layers_equal":cli_layers_equal,
        "cli_lines":cli_bytes.as_ref().ok().map(|b|line_count(b)),
    })).unwrap()).unwrap();
    root
}

fn require_equal(root: &std::path::Path) {
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("comparison.json")).unwrap()).unwrap();
    assert_eq!(result["api_cli_raw_equal"], true, "{root:?}: {result}");
    assert_eq!(result["api_admitted"], true, "{root:?}: {result}");
    assert_eq!(result["cli_code"], 0, "{root:?}: {result}");
    assert_eq!(result["api_layers_equal"], true, "{root:?}: {result}");
    assert_eq!(result["cli_layers_equal"], true, "{root:?}: {result}");
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
