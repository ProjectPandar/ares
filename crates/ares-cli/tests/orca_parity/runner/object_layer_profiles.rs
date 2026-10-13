//! bbs_3mf.cpp:2824-2883; PrintObject.cpp:3805-3833; Slicing.cpp:807-866.
//! Authored profiles are public inputs; Native bytes are generated at runtime.

use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;

use crate::runner::{ExportedCase, OrcaRunner};

const ABSENT: &[u8] =
    include_bytes!("../../../../../tests/parity/object-layer-profiles/absent.3mf");
const RAMP: &[u8] = include_bytes!("../../../../../tests/parity/object-layer-profiles/ramp.3mf");
const STEP: &[u8] = include_bytes!("../../../../../tests/parity/object-layer-profiles/step.3mf");

#[tokio::test]
async fn object_stored_ramp_profile_preserves_complete_native_output() {
    compare_profile("ramp", RAMP).await;
}

#[tokio::test]
async fn object_stored_step_profile_preserves_complete_native_output() {
    compare_profile("step", STEP).await;
}

async fn compare_profile(label: &str, input: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("object-layer-profile-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let baseline = reference(&runner, "absent", ABSENT, &directory);
    let baseline_actual = slice(ABSENT).await.unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&baseline, &baseline_actual)
        .unwrap_or_else(|error| panic!("inactive control: {error}; {directory:?}"));
    let native = reference(&runner, label, input, &directory);
    let native_layers = layer_lines(&native);
    assert_ne!(native_layers, layer_lines(&baseline), "{directory:?}");
    let actual = slice(input).await;
    if let Ok(bytes) = &actual {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
    }
    fs::write(directory.join("native.gcode"), &native).unwrap();
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,
            "input_sha256":crate::runner::cache_identity::raw_digest(input),
            "native_sha256":crate::runner::cache_identity::raw_digest(&native),
            "native_layers":native_layers,
            "ares_result":actual.as_ref().map(Vec::len).map_err(ToString::to_string),
            "runner":runner.work,
        }))
        .unwrap(),
    )
    .unwrap();
    let actual = actual.unwrap_or_else(|error| panic!("{error}; {directory:?}"));
    assert_eq!(layer_lines(&native), layer_lines(&actual), "{directory:?}");
    crate::golden::compare_ordered_bytes_generator_only(&native, &actual)
        .unwrap_or_else(|error| panic!("{error}; {directory:?}"));
}

fn reference(runner: &OrcaRunner, label: &str, input: &[u8], root: &std::path::Path) -> Vec<u8> {
    let directory = root.join(label);
    fs::create_dir(&directory).unwrap();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, input).unwrap();
    runner
        .slice_case(&ExportedCase {
            label: format!("object-layer-profile/{label}"),
            project: input.to_vec(),
            project_path,
            output_dir: directory,
        })
        .unwrap()
        .reference
}

async fn slice(input: &[u8]) -> Result<Vec<u8>, ares_core::SliceError> {
    ares_core::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
}

fn layer_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
        .collect()
}
