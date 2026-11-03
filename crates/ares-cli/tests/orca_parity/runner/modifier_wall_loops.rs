use std::{
    fs,
    io::{Cursor, Read, Write},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use sha2::{Digest, Sha256};

use super::{ExportedCase, OrcaRunner};

#[path = "modifier_wall_loops/variants.rs"]
mod variants;

#[tokio::test]
async fn classic_modifier_wall_loops_match_complete_native_output() {
    let Some(runner) = native_runner() else {
        return;
    };
    let input =
        include_bytes!("../../../../../tests/parity/validation/modifier-wall-loops-classic.3mf");
    compare(
        &runner,
        "classic-inherited-control",
        &without_override(input),
    )
    .await;
    compare(&runner, "classic-modifier-wall4", input).await;
}

#[tokio::test]
async fn arachne_modifier_wall_loops_match_complete_native_output() {
    let Some(runner) = native_runner() else {
        return;
    };
    let input =
        include_bytes!("../../../../../tests/parity/validation/modifier-wall-loops-arachne.3mf");
    compare(
        &runner,
        "arachne-inherited-control",
        &without_override(input),
    )
    .await;
    compare(&runner, "arachne-modifier-wall4", input).await;
}

fn native_runner() -> Option<OrcaRunner> {
    crate::native_environment::binary("ARES_ORCA_BIN")?;
    let runner = OrcaRunner::from_env().unwrap();
    if runner.is_none() {
        crate::native_environment::skipped(
            "Native runner unavailable for modifier wall-loop comparison",
        );
    }
    runner
}

fn without_override(input: &[u8]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace("<metadata key=\"wall_loops\" value=\"4\"/>", "")
                .into_bytes();
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

async fn compare(runner: &OrcaRunner, label: &str, input: &[u8]) {
    let mut builder = tempfile::Builder::new();
    builder.prefix("modifier-wall-loops-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, input).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let reference = runner
        .rerun_reference(&ExportedCase {
            label: format!("modifier-wall-loops/{label}"),
            project: input.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap()
        .reference;
    fs::write(directory.join("native.gcode"), &reference).unwrap();
    let actual = crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(bytes) = &actual {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
    }
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "label": label,
        "input_sha256": Sha256::digest(input).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "native_bytes": reference.len(),
        "ares": actual.as_ref().map(|bytes| bytes.len()).map_err(ToString::to_string),
    })).unwrap()).unwrap();
    let actual = actual.unwrap_or_else(|error| panic!("{label}: {error}; artifacts {directory:?}"));
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|error| panic!("{label}: {error}; artifacts {directory:?}"));
}
