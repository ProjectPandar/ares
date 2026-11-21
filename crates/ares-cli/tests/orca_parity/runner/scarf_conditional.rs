//! Source-owned conditional scarf eligibility: complete Native/API/CLI witnesses.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[path = "scarf_conditional_raft.rs"]
mod raft;

const LABELS: [&str; 5] = [
    "control",
    "positive",
    "sharp",
    "overhang-low",
    "overhang-high",
];

#[test]
fn conditional_scarf_matches_native_public_slicing() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = super::repo_root().join("tests/parity/scarf-conditional");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    let mut results = Vec::new();
    for label in LABELS {
        let input = fs::read(fixtures.join(format!("{label}.3mf"))).unwrap();
        assert_eq!(
            cache_identity::raw_digest(&input),
            manifest["cases"][label]["sha256"]
        );
        let native = native_case(&runner, label, &input);
        let public = public_case(&runner.work, label, &input, &native);
        results.push((label, activity(&native), public));
    }
    profile_entry::write_json(
        &runner.work.join("all-cases-before-assertions.json"),
        &json!(results),
    );
    // All five cases, including the independent overhang witness, are durable
    // before an expected active-positive target failure can terminate the test.
    for (label, native, public) in &results {
        let expected = &manifest["cases"][*label];
        assert_eq!(
            native["slope_count"].as_u64().unwrap() > 0,
            expected["scarf_active"].as_bool().unwrap(),
            "Native activity {label}"
        );
        assert_eq!(native["layers"], 4, "Native layers {label}");
        assert_eq!(
            native["first_layer_slope_count"], 0,
            "Native first layer {label}"
        );
        for movement in native["slope_moves"].as_array().unwrap() {
            assert_eq!(movement["role"], "Outer wall", "Native role {label}");
            assert!(movement["layer"].as_u64().unwrap() > 1);
        }
        assert_eq!(public["api_ok"], true, "API admission {label}");
        assert_eq!(public["cli_exit"], 0, "CLI admission {label}");
        assert_eq!(public["api_complete"], true, "API complete {label}");
        assert_eq!(public["cli_complete"], true, "CLI complete {label}");
        assert_eq!(public["raw_api_cli_equal"], true, "raw API/CLI {label}");
        assert_eq!(
            public["api_first_slope"], native["first_slope"],
            "Native/API first slope {label}"
        );
        assert_eq!(
            public["cli_first_slope"], native["first_slope"],
            "Native/CLI first slope {label}"
        );
    }
    for label in [
        "control",
        "sharp",
        "overhang-low",
        "positive",
        "overhang-high",
    ] {
        let public = &results.iter().find(|row| row.0 == label).unwrap().2;
        assert_eq!(
            public["api_comparison"]["status"], "PASS",
            "strict Native/API output for {label}: {public}"
        );
        assert_eq!(
            public["cli_comparison"]["status"], "PASS",
            "strict Native/CLI output for {label}: {public}"
        );
    }
}

fn complete(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .any(|line| line == "; CONFIG_BLOCK_END")
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
    profile_entry::write_json(
        &directory.join("native-result.json"),
        &json!({
        "label":label,"input_sha256":cache_identity::raw_digest(input),
        "native_sha256":cache_identity::raw_digest(&bytes),"complete":complete(&bytes),
        "activity":activity(&bytes)}),
    );
    assert!(complete(&bytes), "incomplete Native stream {label}");
    let manifest: Value = serde_json::from_slice(
        &fs::read(super::repo_root().join("tests/parity/scarf-conditional/manifest.json")).unwrap(),
    )
    .unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    for (key, value) in manifest["cases"][label]["settings"].as_object().unwrap() {
        let setting = format!("; {key} = {}", value.as_str().unwrap());
        assert!(
            text.lines().any(|line| line == setting),
            "Native setting {label}: {setting}"
        );
    }
    bytes
}

fn activity(bytes: &[u8]) -> Value {
    let mut layer = 0;
    let mut role = "";
    let mut movements = Vec::new();
    for (index, line) in std::str::from_utf8(bytes).unwrap().lines().enumerate() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let tokens: Vec<_> = line.split(';').next().unwrap().split_whitespace().collect();
        if tokens
            .first()
            .is_none_or(|token| !matches!(*token, "G0" | "G1" | "G2" | "G3"))
        {
            continue;
        }
        let has_xy = tokens.iter().any(|token| token.starts_with(['X', 'Y']));
        let has_z = tokens.iter().any(|token| token.starts_with('Z'));
        let extrudes = tokens
            .iter()
            .find_map(|token| token.strip_prefix('E'))
            .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0);
        if has_xy && has_z && extrudes {
            movements.push(json!({"line":index+1,"text":line,"layer":layer,"role":role}));
        }
    }
    json!({"slope_count":movements.len(),"layers":layer,
        "first_layer_slope_count":movements.iter().filter(|row|row["layer"]==1).count(),
        "first_slope":movements.first().map(|row|&row["text"]),"slope_moves":movements})
}

fn comparison(native: &[u8], output: Option<&[u8]>) -> Value {
    let Some(bytes) = output.filter(|bytes| complete(bytes)) else {
        return json!({"status":"NOT EVALUATED","reason":"no complete public stream"});
    };
    let result = crate::golden::compare_ordered_bytes_generator_only(native, bytes);
    json!({"status":if result.is_ok(){"PASS"}else{"FAIL"},
        "difference":result.err().map(|difference|difference.to_string())})
}

fn public_case(work: &Path, label: &str, input: &[u8], native: &[u8]) -> Value {
    let directory = work.join(format!("public-{label}"));
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.3mf");
    let cli_path = directory.join("cli.gcode");
    fs::write(&input_path, input).unwrap();
    profile_entry::write_json(
        &directory.join("calls.json"),
        &json!({
        "input_sha256":cache_identity::raw_digest(input),
        "cli":[env!("CARGO_BIN_EXE_ares"),"slice","-o",cli_path,input_path],
        "api":{"function":"slice_project_plate_validated","overrides":"ConfigOverrides::default()",
            "plate":null,"seed":0,"generation_metadata":[2026,8,27,0,0,0]}}),
    );
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
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
    let cli_bytes = fs::read(&cli_path).ok();
    let api_bytes = api.as_ref().ok().map(Vec::as_slice);
    let cli_bytes = cli_bytes.as_deref();
    let attempt = json!({"label":label,"api_ok":api.is_ok(),"cli_exit":cli.status.code(),
        "api_error":api.as_ref().err().map(|error|format!("{error:?}")),
        "api_complete":api_bytes.is_some_and(complete),"cli_complete":cli_bytes.is_some_and(complete),
        "comparison":"NOT EVALUATED before complete-stream admission"});
    profile_entry::write_json(&directory.join("public-attempt.json"), &attempt);
    let result = json!({"label":label,"api_ok":api.is_ok(),"cli_exit":cli.status.code(),
        "api_complete":api_bytes.is_some_and(complete),"cli_complete":cli_bytes.is_some_and(complete),
        "api_sha256":api_bytes.map(cache_identity::raw_digest),
        "cli_sha256":cli_bytes.map(cache_identity::raw_digest),
        "api_first_slope":api_bytes.map(|bytes|activity(bytes)["first_slope"].take()),
        "cli_first_slope":cli_bytes.map(|bytes|activity(bytes)["first_slope"].take()),
        "raw_api_cli_equal":api_bytes.zip(cli_bytes).map(|(api,cli)|api==cli),
        "api_comparison":comparison(native,api_bytes),"cli_comparison":comparison(native,cli_bytes)});
    profile_entry::write_json(&directory.join("public-result.json"), &result);
    result
}
