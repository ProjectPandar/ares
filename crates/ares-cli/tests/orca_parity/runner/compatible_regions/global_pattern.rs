//! Global PrintConfig enum subsets remain distinct from local full-keymap metadata.
use super::fixture::{self, Scope};
use crate::{
    artifacts,
    runner::{EarlyExit, OrcaRunner},
};
use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use std::io::{Cursor, Read, Write};

#[test]
fn global_top_grid_has_native_config_error() {
    check("top_surface_pattern", "grid");
}
#[test]
fn global_bottom_grid_has_native_config_error() {
    check("bottom_surface_pattern", "grid");
}
#[test]
fn global_solid_grid_has_native_config_error() {
    check("internal_solid_infill_pattern", "grid");
}
#[test]
fn global_sparse_monotonic_has_native_config_error() {
    check("sparse_infill_pattern", "monotonic");
}

fn check(key: &str, value: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (exported, _) = fixture::prepare(&runner, Scope::Object, "rectilinear", false);
    let root = tempfile::Builder::new()
        .prefix("global-pattern-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert!(settings.get(key).is_some());
            settings[key] = json!(value);
            bytes = serde_json::to_vec(&settings).unwrap();
        } else {
            unchanged.insert(file.name().into(), json!(fixture::hash(&bytes)));
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let project = output.finish().unwrap().into_inner();
    let mut final_zip = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    for (name, expected) in &unchanged {
        let mut bytes = Vec::new();
        final_zip
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(json!(fixture::hash(&bytes)), *expected);
    }
    let path = root.join("input.3mf");
    artifacts::write(&path, &project).unwrap();
    let native = runner
        .early_cli_reference(&path, EarlyExit::InvalidValues)
        .unwrap();
    artifacts::write(&root.join("native.stdout"), &native.stdout).unwrap();
    artifacts::write(&root.join("native.stderr"), &native.stderr).unwrap();
    artifacts::write(&root.join("native.result.json"), &native.result_json).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &project,
        GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    ));
    if let Ok(bytes) = &api {
        artifacts::write(&root.join("ares.gcode"), bytes).unwrap();
    }
    let cli = crate::runner::application::cli_command()
        .arg("slice")
        .arg("-o")
        .arg(root.join("cli.gcode"))
        .arg(&path)
        .output()
        .unwrap();
    artifacts::write(&root.join("cli.stdout"), &cli.stdout).unwrap();
    artifacts::write(&root.join("cli.stderr"), &cli.stderr).unwrap();
    artifacts::write(
        &root.join("witness.json"),
        &serde_json::to_vec_pretty(&json!({
            "key":key,"value":value,"input_sha256":fixture::hash(&project),
            "unchanged_members":unchanged,"native_receipt_directory":native.directory,
            "api_error":api.as_ref().err().map(|error| format!("{error:?}")),
            "cli_exit":cli.status.code(),"cli_gcode_exists":root.join("cli.gcode").exists()
        }))
        .unwrap(),
    )
    .unwrap();
    let detail = format!("{key}: invalid value {value}");
    let expected = format!("Param values in 3mf/config error: \n{detail}\n");
    assert_eq!(
        native.stderr,
        expected.as_bytes(),
        "Native exact error: {root:?}"
    );
    assert!(native.gcode.is_none(), "Native rejected input: {root:?}");
    assert!(
        matches!(&api, Err(SliceError::InvalidValuesInProject(actual)) if actual == &detail),
        "public API config error: {:?}; {root:?}",
        api.as_ref().err()
    );
    assert_eq!(
        cli.status.code(),
        Some(if cfg!(unix) { 238 } else { -18 }),
        "{root:?}"
    );
    assert_eq!(cli.stderr, native.stderr, "CLI exact error: {root:?}");
    assert!(!root.join("cli.gcode").exists(), "{root:?}");
}
