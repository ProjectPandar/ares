//! Selected 3MF importer projection -> Model.cpp:500-501 read exception.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const MESSAGE: &str = "The supplied file couldn't be read because it's empty.";
const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/print-bed-two-plates-min.3mf");
const ORIGINAL: &[u8] =
    include_bytes!("../../../../tests/parity/validation/print-bed-two-plates.3mf");

#[derive(Clone, Copy)]
enum Entry {
    Validate,
    Slice,
    Validated,
    Processed,
}

#[test]
fn empty_selected_plate_validation_matches_native_model_read_error() {
    verify("empty-selected-plate/validate", INPUT, Entry::Validate);
}

#[test]
fn empty_selected_plate_slice_matches_native_model_read_error() {
    verify("empty-selected-plate/slice", INPUT, Entry::Slice);
}

#[test]
fn empty_selected_plate_validated_original_matches_native_model_read_error() {
    verify(
        "empty-selected-plate/validated-original",
        ORIGINAL,
        Entry::Validated,
    );
}

#[test]
fn empty_selected_plate_processed_matches_native_model_read_error() {
    verify("empty-selected-plate/processed", INPUT, Entry::Processed);
}

fn verify(label: &str, input: &[u8], entry: Entry) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("empty-selected-plate-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let error = stages::run(
        runner.bin(),
        [
            "--slice",
            "1",
            "--outputdir",
            output.to_str().unwrap(),
            path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    let captured = error.captured.as_ref().unwrap();
    assert_eq!(captured.code, Some(if cfg!(unix) { 250 } else { -6 }));
    assert_eq!(
        captured.stderr,
        format!("{}: {MESSAGE}\n", path.display()).as_bytes()
    );
    let native: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
    assert_eq!(native["return_code"], -6);
    assert_eq!(
        native["error_string"],
        "The input model file to the slicer can not be parsed."
    );
    assert!(fs::read_dir(&output).unwrap().all(|file| {
        file.unwrap()
            .path()
            .extension()
            .is_none_or(|extension| extension != "gcode")
    }));
    let overrides = ConfigOverrides::default();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = match entry {
        Entry::Validate => ares_core::validate_project_plate(input, Some(1), &overrides),
        Entry::Slice => runtime
            .block_on(crate::runner::application::slice_project_plate(
                input,
                GenerationMetadata::default(),
                Some(1),
                &overrides,
                SlicerContext::new(0),
            ))
            .map(|_| ()),
        Entry::Validated => runtime
            .block_on(crate::runner::application::slice_project_plate_validated(
                input,
                GenerationMetadata::default(),
                Some(1),
                &overrides,
                SlicerContext::new(0),
            ))
            .map(|_| ()),
        Entry::Processed => runtime
            .block_on(
                crate::runner::application::slice_project_plate_processed_validated(
                    input,
                    GenerationMetadata::default(),
                    Some(1),
                    &overrides,
                    SlicerContext::new(0),
                ),
            )
            .map(|_| ()),
    };
    let hash = Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let matched = matches!(&actual, Err(SliceError::NoSuitableObjects(text)) if text == MESSAGE);
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "label": label, "input_sha256": hash,
        "source": "bbs_3mf.cpp:2059-2080,2351-2363; Model.cpp:500-501; OrcaSlicer.cpp:1874-1877",
        "status": if matched { "PASS" } else { "DIVERGENT_MODEL_READ_ERROR" },
        "native": native, "native_stderr": String::from_utf8_lossy(&captured.stderr),
        "native_command_evidence": error.evidence, "expected_model_message": MESSAGE,
        "ares_error": actual.as_ref().err().map(ToString::to_string),
        "ares_debug": format!("{actual:?}"), "ares_error_kind_required": "InvalidInput",
        "native_cli_category_not_reimplemented": "CLI_DATA_FILE_ERROR=-6; POSIX250",
    })).unwrap()).unwrap();
    assert!(
        matched,
        "{label}: observed {actual:?}; artifact {directory:?}"
    );
}
