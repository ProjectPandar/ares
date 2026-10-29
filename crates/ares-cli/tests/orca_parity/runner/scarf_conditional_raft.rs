//! Raft + conditional scarf overhang admission: Native-backed E2E witness.
//!
//! Upstream `GCode.cpp:5777-5791`: with raft layers the first object slice
//! has `layer_id() > 0`, so the conditional checks run on a loop whose seam
//! overhang came from `SeamPlacer.cpp:1563` reading
//! `SeamCandidate::unsupported_dist` — which `SeamPlacer.cpp:1083-1092`
//! only assigns through a previous-layer distancer, leaving the layer-0
//! candidate's field uninitialized (`SeamPlacer.hpp:59-70`). Measured Native
//! behavior keeps the first object layer's scarf (16 segments at z .6 in
//! the 5% variant too), while computed later layers flip off below the
//! threshold. Admission does not prove the uninitialized field is zero;
//! the owner-approved absent-distance policy is recorded in
//! `docs/architecture/ard-scarf-raft-unset-distance.md`.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const LABEL: &str = "taper-raft-40";

#[test]
fn raft_conditional_scarf_matches_native_public_slicing() {
    if !super::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = super::repo_root().join("tests/parity/scarf-conditional-raft");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    let input = fs::read(fixtures.join(format!("{LABEL}.3mf"))).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&input),
        manifest["cases"][LABEL]["sha256"]
    );
    let native_bytes = native_case(&runner, &manifest, &input);
    let native = activity(&native_bytes);
    let public = public_case(&runner.work, &input, &native_bytes);
    profile_entry::write_json(
        &runner.work.join("case-before-assertions.json"),
        &json!((LABEL, &native, &public)),
    );
    assert_eq!(
        native["slope_count"], 64,
        "Native keeps the scarf on all four object layers over the raft"
    );
    assert_eq!(native["layers"], 6, "Native layers: 2 raft + 4 object");
    assert_eq!(
        native["first_layer_slope_count"], 0,
        "raft layers themselves carry no scarf"
    );
    assert_eq!(
        native["first_slope"], "G1 X109.72 Y114.063 Z.63 E.0027 ; perimeter",
        "Native first slope sits on the first object layer (z .6 raft offset)"
    );
    let layers: Vec<_> = native["slope_moves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["layer"].as_u64().unwrap())
        .collect();
    for object_layer in 3..=6 {
        assert_eq!(
            layers
                .iter()
                .filter(|&&layer| layer == object_layer)
                .count(),
            16,
            "object layer {object_layer} scarf segment count"
        );
    }
    for movement in native["slope_moves"].as_array().unwrap() {
        assert_eq!(movement["role"], "Outer wall", "Native role");
    }
    assert_eq!(public["api_ok"], true, "API admission");
    assert_eq!(public["cli_exit"], 0, "CLI admission");
    assert_eq!(public["api_complete"], true, "API complete");
    assert_eq!(public["cli_complete"], true, "CLI complete");
    assert_eq!(public["raw_api_cli_equal"], true, "raw API/CLI");
    assert_eq!(
        public["api_comparison"]["status"], "PASS",
        "strict Native/API: {public}"
    );
    assert_eq!(
        public["cli_comparison"]["status"], "PASS",
        "strict Native/CLI: {public}"
    );
}

fn complete(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .any(|line| line == "; CONFIG_BLOCK_END")
}

fn native_case(runner: &OrcaRunner, manifest: &Value, input: &[u8]) -> Vec<u8> {
    let directory = runner.work.join(LABEL);
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
        &json!({"label":LABEL,"input_sha256":cache_identity::raw_digest(input),
        "native_sha256":cache_identity::raw_digest(&bytes),"complete":complete(&bytes)}),
    );
    assert!(complete(&bytes), "incomplete Native stream");
    let text = std::str::from_utf8(&bytes).unwrap();
    for (key, value) in manifest["cases"][LABEL]["settings"].as_object().unwrap() {
        let setting = format!("; {key} = {}", value.as_str().unwrap());
        assert!(
            text.lines().any(|line| line == setting),
            "Native setting {LABEL}: {setting}"
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

fn public_case(work: &Path, input: &[u8], native: &[u8]) -> Value {
    let directory = work.join(format!("public-{LABEL}"));
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.3mf");
    let cli_path = directory.join("cli.gcode");
    fs::write(&input_path, input).unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
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
        .block_on(ares_core::slice_project_plate_validated(
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
    json!({"label":LABEL,"api_ok":api.is_ok(),"cli_exit":cli.status.code(),
        "api_error":api.as_ref().err().map(|error|format!("{error:?}")),
        "api_complete":api_bytes.is_some_and(complete),
        "cli_complete":cli_bytes.as_deref().is_some_and(complete),
        "api_sha256":api_bytes.map(cache_identity::raw_digest),
        "cli_sha256":cli_bytes.as_deref().map(cache_identity::raw_digest),
        "raw_api_cli_equal":api_bytes.zip(cli_bytes.as_deref()).map(|(api,cli)|api==cli),
        "api_comparison":comparison(native,api_bytes),
        "cli_comparison":comparison(native,cli_bytes.as_deref())})
}
