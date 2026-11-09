//! Config.cpp:882-923/1004-1060: typed JSON-array wire joining.

use std::{
    fs,
    io::{Cursor, Read, Write},
};

use crate::{
    OrcaRunner, golden,
    runner::{
        application,
        native_api_cli::{NativeCase, sha},
        native_stream_case, stages,
    },
};

const CONTROL: &[u8] = include_bytes!(
    "../../../../tests/parity/machine-acceleration-cache-rounding/stock-control.3mf"
);
const UPSTREAM: &[&str] = &[
    "Config.cpp:882-923,1004-1060",
    "PrintConfig.cpp:8595,10045-10070",
];

#[test]
fn thumbnails_string_array_complete_parity() {
    check("thumbnails", "thumbnails", &["32x32/PNG", "64x64/PNG"]);
}

#[test]
fn thumbnails_empty_array_complete_parity() {
    check("thumbnails-empty", "thumbnails", &[]);
}

#[test]
fn notes_string_array_complete_parity() {
    check(
        "notes",
        "notes",
        &["first stock token", "second stock token"],
    );
}

#[test]
fn mixed_thumbnail_array_native_outcome_recorded_without_coercion() {
    let project = with_option("thumbnails", serde_json::json!(["32x32/PNG", 64]));
    let root = tempfile::Builder::new()
        .prefix("project-scalar-string-mixed-control-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.3mf");
    let output = root.join("native");
    fs::write(&input, &project).unwrap();
    fs::create_dir(&output).unwrap();
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let native = stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            input.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &root,
    );
    if let Err(error) = &native {
        assert_eq!(error.kind, stages::FailureKind::Process, "{error}");
    }
    let error = ares_core::load_project(&project).unwrap_err().to_string();
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(root.join("ares.cli.gcode"))
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    fs::write(
        root.join("mixed-control.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": sha(&project), "upstream": UPSTREAM,
            "native_stage_success": native.is_ok(),
            "native_error": native.as_ref().err().map(ToString::to_string),
            "native_gcode_exists": output.join("plate_1.gcode").is_file(),
            "ares_api_error": error, "ares_cli_code": cli.status.code(),
            "slice_parity_gate": false,
            "control": "Native mixed-array partial load versus Ares admission rejection",
            "native_command_status_and_diagnostics": "command-*/status.json and stdout/stderr snapshots",
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(error.contains("invalid Orca option thumbnails"));
    assert!(!cli.status.success());
    assert!(!root.join("ares.cli.gcode").exists());
    for value in [serde_json::json!([true]), serde_json::json!([null])] {
        assert!(ares_core::load_project(with_option("thumbnails", value)).is_err());
    }
}

fn check(label: &str, key: &str, values: &[&str]) {
    let project = with_option(key, serde_json::json!(values));
    let scalar = with_option(key, serde_json::json!(values.join(",")));
    let (reference, _) = native_stream_case::qualify(
        ("project-scalar-string-arrays", UPSTREAM),
        label,
        &project,
        &sha(&project),
        false,
    );
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let control = NativeCase::slice(&runner, &scalar, "project-scalar-string-control-");
    let normalized = |value: &str| {
        golden::normalize_one_generator_line(value.as_bytes(), golden::GeneratorKind::Orca).unwrap()
    };
    let difference =
        golden::first_difference(&normalized(&control.native), &normalized(&reference.native))
            .map(|value| value.to_string());
    fs::write(
        reference.root.join("native-scalar-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "array_sha256": sha(&project), "scalar_sha256": sha(&scalar),
            "scalar_root": control.root, "array_root": reference.root,
            "native_scalar_difference": difference, "complete_ordered_comparison": true,
            "join_separator": ",", "option_kind": "coString",
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        difference.is_none(),
        "Native scalar control: {difference:?}"
    );
    reference.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}

fn with_option(key: &str, value: serde_json::Value) -> Vec<u8> {
    const MEMBER: &str = "Metadata/project_settings.config";
    let mut archive = zip::ZipArchive::new(Cursor::new(CONTROL)).unwrap();
    let mut json = String::new();
    archive
        .by_name(MEMBER)
        .unwrap()
        .read_to_string(&mut json)
        .unwrap();
    let mut settings: serde_json::Value = serde_json::from_str(&json).unwrap();
    settings
        .as_object_mut()
        .unwrap()
        .insert(key.to_owned(), value);
    let changed = serde_json::to_vec(&settings).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == MEMBER {
            writer
                .start_file(MEMBER, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(&changed).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
