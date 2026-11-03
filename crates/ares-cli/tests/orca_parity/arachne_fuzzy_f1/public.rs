use super::{native::NativeCase, sha, witness};
use serde_json::json;
use std::{fs, process::Command};

pub(super) fn slice(project: &[u8], seed: u32) -> Result<Vec<u8>, ares_core::SliceError> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(seed),
        ))
}

pub(super) fn compare(case: NativeCase, project: &[u8]) {
    let root = &case.root;
    let api = slice(project, 0);
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.validated.gcode"), gcode).unwrap();
        fs::write(
            root.join("ares.wall-witness.json"),
            serde_json::to_vec_pretty(&witness::capture(gcode)).unwrap(),
        )
        .unwrap();
    }
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
    let cli_bytes = fs::read(output);
    let api_comparison = case.strict.then(|| {
        api.as_ref()
            .map_err(|error| format!("API: {error:?}"))
            .and_then(|g| crate::golden::compare_ordered_bytes_generator_only(&case.reference, g))
    });
    let cli_comparison = case.strict.then(|| {
        cli_bytes
            .as_ref()
            .map_err(|error| format!("CLI: {error}"))
            .and_then(|g| crate::golden::compare_ordered_bytes_generator_only(&case.reference, g))
    });
    fs::write(root.join("comparison.json"),serde_json::to_vec_pretty(&json!({
        "input_sha256":sha(project),"settings":case.settings,"native_sha256":sha(&case.reference),
        "strict_parity_required":case.strict,
        "strict_parity_performed":case.strict && api.is_ok() && cli_bytes.is_ok(),
        "oracle_contract":if case.strict {"strict full ordered generator bytes"} else {"official stochastic diagnostic ONLY, never parity PASS"},
        "api_admitted":api.is_ok(),"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
        "api_sha256":api.as_ref().ok().map(|g|sha(g)),"api_comparison":api_comparison,
        "cli_code":cli.status.code(),"cli_binary_sha256":sha(&fs::read(binary).unwrap()),
        "cli_sha256":cli_bytes.as_ref().ok().map(|g|sha(g)),"cli_comparison":cli_comparison,
        "comparator":"unchanged golden::compare_ordered_bytes_generator_only; never runs on official stochastic reference"
    })).unwrap()).unwrap();
    assert!(api.is_ok(), "public F1 admission RED: {root:?}: {api:?}");
    assert!(
        cli.status.success(),
        "CLI F1 admission RED: {root:?}: {cli:?}"
    );
    let api = api.unwrap();
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "public API/CLI raw equality: {root:?}");
    if let Some(result) = api_comparison {
        result.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    }
    if let Some(result) = cli_comparison {
        result.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    }
    if !case.strict {
        witness::verify_diagnostic(&witness::capture(&api), &case.contract);
        witness::verify_diagnostic(&witness::capture(&cli_bytes), &case.contract);
    }
}
