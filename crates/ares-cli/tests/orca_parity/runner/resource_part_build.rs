//! ObjectImporter resource parsing: missing build must not reject a slicing request.
//! Failures: source drift, Native admission/incomplete/non-neutral output,
//! mismatched public errors, API/CLI raw disagreement, complete-stream divergence.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn resource_part_without_build_public_slicing() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = super::repo_root().join("tests/parity/3mf-resource-build");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    let references = native_references(&runner, &fixtures, &manifest);
    let rows: Vec<_> = references
        .iter()
        .map(|reference| public_slice(&runner.work, &fixtures, reference))
        .collect();
    profile_entry::write_json(
        &runner.work.join("resource-build-summary.json"),
        &json!({"legs":rows}),
    );
    let control = &rows[2];
    assert!(
        control["api_error"].is_null() && control["cli_exit"] == 0,
        "control public admission: {control}"
    );
    assert_eq!(
        control["api_cli_raw_equal"], true,
        "control raw API/CLI equality"
    );
    assert!(
        control["difference"].is_null() && control["cli_difference"].is_null(),
        "STOP residual control G-code: {control}"
    );
    for row in &rows[..2] {
        assert!(
            row["api_error"].is_null(),
            "public slicing admission: {row}"
        );
        assert_eq!(row["cli_exit"], 0, "CLI public admission");
        assert_eq!(
            row["api_cli_raw_equal"], true,
            "raw public API/CLI equality"
        );
        assert!(
            row["difference"].is_null() && row["cli_difference"].is_null(),
            "STOP residual G-code: {row}"
        );
    }
}

fn native_references(runner: &OrcaRunner, fixtures: &Path, manifest: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    for (label, case) in [
        ("positive", "positive"),
        ("repeat", "positive"),
        ("control", "control"),
    ] {
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
                .any(|line| line == "; CONFIG_BLOCK_END"),
            "complete Native footer"
        );
        let normalized = crate::golden::normalize_one_generator_line(
            &output,
            crate::golden::GeneratorKind::Orca,
        )
        .unwrap();
        let row = json!({"label":label,"case":case,"input_sha256":digest,
            "native_path":directory.join("plate_1.gcode"),"native_sha256":cache_identity::raw_digest(&output),
            "native_normalized_sha256":cache_identity::raw_digest(&normalized),"native_commands":directory});
        profile_entry::write_json(&directory.join("native-reference.json"), &row);
        rows.push(row);
    }
    for row in &rows[1..] {
        assert_eq!(
            rows[0]["native_normalized_sha256"], row["native_normalized_sha256"],
            "Native fresh repeat/control neutrality"
        );
    }
    profile_entry::write_json(
        &runner.work.join("native-references.json"),
        &json!({"approved_spec":manifest["approved_spec"],"references":rows}),
    );
    rows
}

fn public_slice(work: &Path, fixtures: &Path, reference: &Value) -> Value {
    let case = reference["case"].as_str().unwrap();
    let directory: PathBuf = work.join(format!("public-{}", reference["label"].as_str().unwrap()));
    fs::create_dir(&directory).unwrap();
    let input = fs::read(fixtures.join(format!("{case}.3mf"))).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&input),
        reference["input_sha256"]
    );
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, &input).unwrap();
    let native = fs::read(reference["native_path"].as_str().unwrap()).unwrap();
    fs::write(directory.join("native.gcode"), &native).unwrap();
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
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(directory.join("cli.gcode"))
        .arg(&input_path)
        .output()
        .unwrap();
    fs::write(directory.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("cli.stderr"), &cli.stderr).unwrap();
    let api_error = api.as_ref().err().map(|error| format!("{error:?}"));
    let mut row = json!({"case":case,"label":reference["label"],"reference":reference,
        "api_error":api_error,"cli_exit":cli.status.code(),
        "cli_stderr":String::from_utf8_lossy(&cli.stderr),
        "api_cli_raw_equal":null,"difference":null,"cli_difference":null,
        "comparison":"not evaluated: no complete Ares stream",
        "rust_entry":"ares_core::slice_project_plate_validated"});
    if let Ok(bytes) = &api {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
        row["difference"] =
            json!(crate::golden::compare_ordered_bytes_generator_only(&native, bytes).err());
        row["api_sha256"] = json!(cache_identity::raw_digest(bytes));
        if cli.status.success() {
            let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
            row["api_cli_raw_equal"] = json!(bytes == &cli_bytes);
            row["cli_sha256"] = json!(cache_identity::raw_digest(&cli_bytes));
            row["cli_difference"] = json!(
                crate::golden::compare_ordered_bytes_generator_only(&native, &cli_bytes).err()
            );
            row["comparison"] = json!(
                "complete ordered bytes; generator identity only; motion/arcs/M73/stats/footer retained"
            );
        }
    }
    profile_entry::write_json(&directory.join("public-result.json"), &row);
    row
}
