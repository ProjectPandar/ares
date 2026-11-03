//! GCodeProcessor.cpp:1866-1999 / GCode.cpp:2166-2173, post-emission rejection.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/timelapse-unprintable-gcode.3mf");
const SHA: &str = "67a27bafcda838742db058b1edb812055c9470608e47dc83a314357a8e9b233d";
const ERROR: &str = "Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer.";

#[tokio::test]
async fn timelapse_extrusion_outside_printable_area_rejects_after_processing() {
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hash, SHA);
    let mut builder = tempfile::Builder::new();
    builder.prefix("gcode-printable-area-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let path = directory.join("input.3mf");
    fs::write(&path, INPUT).unwrap();
    let native = OrcaRunner::from_env().unwrap().map(|runner| {
        let output = directory.join("native");
        fs::create_dir(&output).unwrap();
        let failure = stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                output.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ],
            stages::Stage::Slice,
            &directory,
        )
        .unwrap_err();
        let captured = failure.captured.as_ref().unwrap();
        assert_eq!(captured.code, Some(if cfg!(unix) { 154 } else { -102 }));
        assert!(output.join("plate_1.gcode").exists());
        let result: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
        assert_eq!(result["return_code"], -102);
        assert_eq!(result["error_string"], ERROR);
        json!({"result":result, "evidence":failure.evidence})
    });
    let overrides = ConfigOverrides::default();
    let bytes = crate::runner::application::slice_project_plate_validated(
        INPUT,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    )
    .await;
    let processed = crate::runner::application::slice_project_plate_processed_validated(
        INPUT,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    )
    .await;
    if let Ok(gcode) = &bytes {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":SHA,
            "native":native,
            "byte_entry":bytes.as_ref().map(|gcode| gcode.len()).map_err(ToString::to_string),
            "processed_entry":processed.as_ref().map(|output| output.gcode.len()).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        bytes.is_err(),
        "byte entry accepted; evidence {directory:?}"
    );
    assert!(
        processed.is_err(),
        "processed entry accepted; evidence {directory:?}"
    );
    for error in [bytes.unwrap_err(), processed.unwrap_err()] {
        assert_eq!(error.to_string(), ERROR);
        assert!(
            !matches!(error, SliceError::InvalidInput(_)),
            "must expose a typed processor failure"
        );
    }
}
