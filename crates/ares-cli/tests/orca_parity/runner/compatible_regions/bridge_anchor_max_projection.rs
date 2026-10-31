//! D-M: Fill.cpp::group_fills:978-989 projects local FloatOrPercent values
//! before sparse anchoring (:1477-1482), without a finite-maximum gate.
use crate::{self as parity, runner::OrcaRunner};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

use super::fixture::{self, Scope};

#[test]
fn local_object_anchor_max_float_overflow() {
    check(Scope::Object, "1e39");
}

#[test]
fn local_part_anchor_max_float_overflow() {
    check(Scope::Part, "1e39");
}

#[test]
fn local_object_anchor_max_percent_overflow() {
    check(Scope::Object, "1e39%");
}

#[test]
fn local_part_anchor_max_percent_overflow() {
    check(Scope::Part, "1e39%");
}

fn check(scope: Scope, extreme: &str) {
    for value in ["20", extreme] {
        let Some(runner) = OrcaRunner::from_env().unwrap() else {
            return;
        };
        // Local solid monotonic equals the global fixture value; only max varies.
        let (mut exported, fixture_proof) = fixture::prepare(&runner, scope, "monotonic", false);
        let (project, unchanged, metadata) = inject_max(&exported.project, scope, value);
        exported.project = project;
        fs::write(&exported.project_path, &exported.project).unwrap();
        let carrier = value.trim_end_matches('%').parse::<f64>().unwrap();
        assert!(carrier.is_finite());
        assert_eq!((carrier as f32).is_finite(), value == "20");
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let directory = outcome.artifacts.as_ref().unwrap();
        let (bridges, sparse) = roles(&sliced.reference);
        let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
            .arg("slice")
            .arg("-o")
            .arg(directory.join("cli.gcode"))
            .arg(directory.join("input.3mf"))
            .output()
            .unwrap();
        fs::write(directory.join("cli.stdout"), &cli.stdout).unwrap();
        fs::write(directory.join("cli.stderr"), &cli.stderr).unwrap();
        fs::write(
            directory.join("anchor-max-witness.json"),
            serde_json::to_vec_pretty(&json!({
                "scope":scope.name(), "local_max":value, "fixture":fixture_proof,
                "local_metadata":metadata, "unchanged_members":unchanged,
                "input_sha256":fixture::hash(&sliced.project),
                "native_sha256":fixture::hash(&sliced.reference),
                "internal_bridge_moves":bridges, "sparse_moves":sparse,
                "status":outcome.status, "detail":outcome.detail,
                "cli_exit":cli.status.code(),
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(
            sliced
                .reference
                .windows(b"; CONFIG_BLOCK_END".len())
                .any(|window| window == b"; CONFIG_BLOCK_END")
        );
        assert!(
            bridges > 0 && sparse > 0,
            "active ceiling roles: {directory:?}"
        );
        assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
        assert!(cli.status.success(), "CLI local max: {directory:?}");
        let actual = fs::read(directory.join("ares.gcode")).unwrap();
        let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
        assert_eq!(actual, cli_bytes, "API/CLI raw output: {directory:?}");
        parity::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes)
            .unwrap();
    }
}

fn inject_max(project: &[u8], scope: Scope, value: &str) -> (Vec<u8>, serde_json::Value, String) {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = serde_json::Map::new();
    let mut metadata = String::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("key=\"infill_anchor_max\""));
            let expression = match scope {
                Scope::Object => r#"<object id="[^"]+">"#,
                Scope::Part => r#"<part id="[^"]+" subtype="normal_part">"#,
            };
            let regex = regex::Regex::new(expression).unwrap();
            let starts = regex.find_iter(&text).collect::<Vec<_>>();
            assert_eq!(starts.len(), 1);
            let start = starts.first().unwrap();
            metadata = format!(
                "{}\n      <metadata key=\"infill_anchor_max\" value=\"{value}\"/>{}",
                &text[..start.end()],
                &text[start.end()..]
            );
            assert_eq!(metadata.matches("key=\"infill_anchor_max\"").count(), 1);
            bytes = metadata.as_bytes().to_vec();
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
    let mut final_input = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    for (name, expected) in &unchanged {
        let mut bytes = Vec::new();
        final_input
            .by_name(name)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(json!(fixture::hash(&bytes)), *expected);
    }
    (project, json!(unchanged), metadata)
}

fn roles(bytes: &[u8]) -> (usize, usize) {
    let mut role = "";
    let mut bridge = 0;
    let mut sparse = 0;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if line.starts_with("G1 ")
            && (line.contains(" X") || line.contains(" Y"))
            && line
                .split_ascii_whitespace()
                .find_map(|token| token.strip_prefix('E'))
                .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
        {
            match role {
                "Internal Bridge" => bridge += 1,
                "Sparse infill" => sparse += 1,
                _ => {}
            }
        }
    }
    (bridge, sparse)
}
