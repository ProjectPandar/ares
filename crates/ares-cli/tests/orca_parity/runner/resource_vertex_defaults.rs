//! Target-equality RED for shared ObjectImporter/main vertex zero defaults.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[test]
fn resource_vertex_missing_coordinate_matches_native_public_slicing() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = super::repo_root().join("tests/parity/3mf-resource-vertex-defaults");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    let native = [
        native_case(&runner, &fixtures, &manifest, "control", "control"),
        native_case(&runner, &fixtures, &manifest, "positive", "positive"),
        native_case(&runner, &fixtures, &manifest, "positive", "repeat"),
    ];
    public_case(&runner.work, &fixtures, &native[0]);
    public_case(&runner.work, &fixtures, &native[1]);
    public_case(&runner.work, &fixtures, &native[2]);
}

fn native_case(
    runner: &OrcaRunner,
    fixtures: &Path,
    manifest: &Value,
    case: &str,
    label: &str,
) -> Value {
    let directory = runner.work.join(label);
    fs::create_dir(&directory).unwrap();
    let input = fs::read(fixtures.join(format!("{case}.3mf"))).unwrap();
    let digest = cache_identity::raw_digest(&input);
    assert_eq!(digest, manifest["sha256"][format!("{case}.3mf")]);
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, &input).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            directory.to_str().unwrap().to_owned(),
            input_path.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let output = fs::read(directory.join("plate_1.gcode")).unwrap();
    assert!(
        std::str::from_utf8(&output)
            .unwrap()
            .lines()
            .any(|line| line == "; CONFIG_BLOCK_END")
    );
    let normalized =
        crate::golden::normalize_one_generator_line(&output, crate::golden::GeneratorKind::Orca)
            .unwrap();
    json!({"label":label,"case":case,"input_sha256":digest,"native_path":directory.join("plate_1.gcode"),
        "native_sha256":cache_identity::raw_digest(&output),"native_normalized_sha256":cache_identity::raw_digest(&normalized),"native_commands":directory})
}

fn public_case(work: &Path, fixtures: &Path, native: &Value) {
    let case = native["case"].as_str().unwrap();
    let label = native["label"].as_str().unwrap();
    let directory = work.join(format!("public-{label}"));
    fs::create_dir(&directory).unwrap();
    let input = fs::read(fixtures.join(format!("{case}.3mf"))).unwrap();
    assert_eq!(cache_identity::raw_digest(&input), native["input_sha256"]);
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, &input).unwrap();
    let native_bytes = fs::read(native["native_path"].as_str().unwrap()).unwrap();
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(directory.join("cli.gcode"))
        .arg(&input_path)
        .output()
        .unwrap();
    fs::write(directory.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("cli.stderr"), &cli.stderr).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    assert!(
        cli.status.success(),
        "target CLI slicing failed for {label}: {}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let api = api.unwrap_or_else(|error| panic!("target API slicing failed for {label}: {error}"));
    fs::write(directory.join("api.gcode"), &api).unwrap();
    let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI bytes for {label}");
    crate::golden::compare_ordered_bytes_generator_only(&native_bytes, &api)
        .unwrap_or_else(|difference| panic!("strict Native/API output for {label}: {difference}"));
    crate::golden::compare_ordered_bytes_generator_only(&native_bytes, &cli_bytes)
        .unwrap_or_else(|difference| panic!("strict Native/CLI output for {label}: {difference}"));
    profile_entry::write_json(
        &directory.join("public-result.json"),
        &json!({"label":label,"case":case,"status":"TARGET_EQUAL"}),
    );
}
