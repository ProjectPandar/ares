//! PrintObject.cpp:1530-1723 (`detect_surfaces_type`, interface_shells:
//! per-region top diff at :1589, extra `stBottom` at :1611-1630) and
//! PrintObject.cpp:2024/:2121 (`discover_vertical_shells` per-region path).
//!
//! Fixture `tests/parity/interface-shells/two-region-prism.3mf`: one object,
//! two model parts on extruders 1/2 — part 2 (10×10×6) is nested inside the
//! part 1 body (20×20×10). Layers below z=6 carry both regions and the
//! overlay's top surface at z≈6 lies inside the base body: with
//! `interface_shells=1` the overlay must print internal solid top shells
//! there; with `0` the area stays sparse infill over the whole layer.
use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const BASE: &[u8] =
    include_bytes!("../../../../../tests/parity/interface-shells/two-region-prism.3mf");

/// Rewrites `Metadata/model_settings.config`, setting the object-level
/// `interface_shells` metadata. Verifies the fixture shape it patches.
fn project(interface_shells: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert_eq!(
                text.matches("<part ").count(),
                2,
                "fixture must keep two model parts"
            );
            assert!(
                text.contains(r#"<metadata key="extruder" value="2"/>"#),
                "fixture part 2 must stay on extruder 2"
            );
            assert!(
                !text.contains("interface_shells"),
                "fixture baseline must not carry interface_shells metadata"
            );
            let text = text.replacen(
                "    <metadata key=\"extruder\" value=\"1\"/>\n",
                &format!(
                    "    <metadata key=\"extruder\" value=\"1\"/>\n    <metadata key=\"interface_shells\" value=\"{interface_shells}\"/>\n"
                ),
                1,
            );
            bytes = text.into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn native(runner: &OrcaRunner, directory: &std::path::Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir(directory).unwrap();
    let path = directory.join("input.3mf");
    fs::write(&path, project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        output.to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    stages::run(runner.bin(), args, stages::Stage::Slice, directory).unwrap();
    let result = fs::read(output.join("result.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(metadata["return_code"], 0, "{directory:?}");
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(!reference.is_empty());
    reference
}

fn leg(label: &str, interface_shells: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("interface-shells-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = project(interface_shells);
    let reference = native(&runner, &directory, &project);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(ares_core::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(gcode) = &public {
        fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("validated API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"interface_shells":interface_shells,
            "input_sha256":sha(&project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "CLI_code":cli.status.code(),
            "api_error":public.as_ref().err().map(|error| format!("{error:?}"))
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}

/// Control: interface_shells=0 keeps whole-layer surface semantics and must
/// stay byte-identical before and after the alignment slice.
#[test]
fn interface_shells_alignment_off_control() {
    leg("off-control", "0");
}

/// Alignment: interface_shells=1 on a two-region object must match the
/// Native oracle byte-for-byte (per-region top surfaces plus the extra
/// `stBottom` pass of PrintObject.cpp:1611-1630).
#[test]
fn interface_shells_alignment_on() {
    leg("on", "1");
}
