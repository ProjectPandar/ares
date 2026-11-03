//! `support_ironing_pattern` domain handling.
//!
//! Upstream: `PrintConfig.cpp:6565-6573` defines the option as a coEnum with
//! GUI values {rectilinear, concentric} only, but the 3MF project-settings
//! load path does not restrict to that enum domain: any InfillPattern name
//! reaches `SupportParameters.hpp:65-68` and the ironing path generator at
//! `SupportCommon.cpp:1877-1903`, which dispatches through
//! `FillBase.cpp:40-77` (`Fill::new_from_type`) and fills the top contact
//! layer polys with that pattern (role erIroning). `PrintConfig.cpp:8206`
//! additionally normalizes the legacy "zig-zag" alias to rectilinear at
//! config load.
//!
//! Ares currently implements only Rectilinear and Concentric in
//! `support_material/ironing.rs` and rejects every other pattern with
//! `UnsupportedProjectFeature`. This suite pins the Native oracle behavior
//! for patterns ares already implements as surface fills elsewhere
//! (`fill_entities/generate.rs`): grid, monotonic, honeycomb and triangles
//! must slice byte-identically to Native instead of erroring, while the
//! rectilinear and legacy zig-zag legs guard the already-aligned paths.
//!
//! Fixture `tests/parity/support-ironing/overhang-organic.3mf`: organic tree
//! overhang object with `support_ironing=1` and
//! `support_ironing_pattern=rectilinear`; the pattern value is rewritten per
//! leg in `Metadata/project_settings.config`.
use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

pub(super) const BASE: &[u8] =
    include_bytes!("../../../../../tests/parity/support-ironing/overhang-organic.3mf");

/// Rewrites `Metadata/project_settings.config`, swapping the
/// `support_ironing_pattern` value. Verifies the fixture shape it patches.
fn project(pattern: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert!(
                text.contains(r#""support_ironing": "1""#),
                "fixture must enable support ironing"
            );
            let needle = r#""support_ironing_pattern": "rectilinear""#;
            assert!(
                text.contains(needle),
                "fixture must carry the rectilinear baseline pattern"
            );
            bytes = text
                .replacen(
                    needle,
                    &format!(r#""support_ironing_pattern": "{pattern}""#),
                    1,
                )
                .into_bytes();
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

/// Rewrites `Metadata/project_settings.config`, swapping the sparse infill
/// pattern value. Control seam for the connect-stage change: sparse
/// lateral-lattice output must stay byte-identical to Native.
fn sparse_project(pattern: &str) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            let needle = r#""sparse_infill_pattern": "crosshatch""#;
            assert!(
                text.contains(needle),
                "fixture must carry the crosshatch sparse baseline"
            );
            bytes = text
                .replacen(
                    needle,
                    &format!(r#""sparse_infill_pattern": "{pattern}""#),
                    1,
                )
                .into_bytes();
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
    assert!(
        String::from_utf8_lossy(&reference)
            .matches("ironing")
            .count()
            >= 15,
        "{directory:?}: native oracle must iron the support contact layers"
    );
    reference
}

fn leg(label: &str, pattern: &str) {
    leg_with_project(label, pattern, project(pattern));
}

pub(super) fn leg_with_project(label: &str, pattern: &str, project: Vec<u8>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("support-ironing-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let reference = native(&runner, &directory, &project);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(crate::runner::application::slice_project_plate_validated(
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
            "label":label,"pattern":pattern,
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

#[path = "support_ironing_pattern_domain/legs.rs"]
mod legs;
