//! Public whole-output RED for SeamPlacer's physical Layer::slice_z handoff.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[test]
fn modifier_physical_layer_seam_plane_matches_native_public_slicing() {
    if !super::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = super::repo_root().join("tests/parity/seam-physical-plane");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    for label in ["control", "positive"] {
        let input = fs::read(fixtures.join(format!("{label}.3mf"))).unwrap();
        assert_eq!(
            cache_identity::raw_digest(&input),
            manifest["sha256"][format!("{label}.3mf")]
        );
        let native = native_case(&runner, label, &input);
        public_case(&runner.work, label, &input, &native);
    }
}

fn native_case(runner: &OrcaRunner, label: &str, input: &[u8]) -> Vec<u8> {
    let directory = runner.work.join(label);
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, input).unwrap();
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
    let bytes = fs::read(directory.join("plate_1.gcode")).unwrap();
    assert!(
        std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .any(|line| line == "; CONFIG_BLOCK_END")
    );
    profile_entry::write_json(
        &directory.join("native-result.json"),
        &json!({"label":label,"input_sha256":cache_identity::raw_digest(input),
            "native_sha256":cache_identity::raw_digest(&bytes),"complete":true}),
    );
    bytes
}

fn public_case(work: &Path, label: &str, input: &[u8], native: &[u8]) {
    let directory = work.join(format!("public-{label}"));
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, input).unwrap();
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
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(directory.join("api.gcode"), bytes).unwrap();
    }
    profile_entry::write_json(
        &directory.join("public-attempt.json"),
        &json!({"label":label,"api_ok":api.is_ok(),"cli_exit":cli.status.code(),
            "api_error":api.as_ref().err().map(|error|format!("{error:?}")),
            "comparison":"NOT EVALUATED before complete-stream admission"}),
    );
    assert!(
        cli.status.success(),
        "target CLI slicing failed for {label}: {}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let api = api.unwrap_or_else(|error| panic!("target API slicing failed for {label}: {error}"));
    let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
    let api_difference = crate::golden::compare_ordered_bytes_generator_only(native, &api);
    let cli_difference = crate::golden::compare_ordered_bytes_generator_only(native, &cli_bytes);
    profile_entry::write_json(
        &directory.join("public-result.json"),
        &json!({"label":label,"raw_api_cli_equal":api==cli_bytes,
            "api_sha256":cache_identity::raw_digest(&api),
            "cli_sha256":cache_identity::raw_digest(&cli_bytes),
            "api_difference":api_difference.as_ref().err(),
            "cli_difference":cli_difference.as_ref().err(),
            "status":if api==cli_bytes && api_difference.is_ok() && cli_difference.is_ok() {
                "TARGET_EQUAL"
            } else { "TARGET_DIVERGENT" }}),
    );
    assert_eq!(api, cli_bytes, "raw API/CLI bytes for {label}");
    api_difference
        .unwrap_or_else(|difference| panic!("strict Native/API output for {label}: {difference}"));
    cli_difference
        .unwrap_or_else(|difference| panic!("strict Native/CLI output for {label}: {difference}"));
}
