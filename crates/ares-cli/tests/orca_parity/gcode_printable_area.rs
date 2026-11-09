//! GCodeProcessor.cpp:1866-1999 / GCode.cpp:2166-2173, post-emission rejection.
use std::{
    fs,
    io::{Cursor, Read, Write},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/timelapse-unprintable-gcode.3mf");
const ERROR: &str = "Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer.";

/// `INPUT` on a bed its skirt runs past, with the object still inside the print volume and the
/// tower out of the way: only the emitted path can reach the rejection.
fn skirt_past_the_bed_edge() -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(INPUT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["printable_area"] = json!(["0x0", "150x0", "150x150", "0x150"]);
            settings["enable_prime_tower"] = json!("0");
            settings["timelapse_type"] = json!("0");
            settings["skirt_distance"] = json!("40");
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

#[tokio::test]
async fn skirt_past_the_bed_edge_rejects_after_processing() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = skirt_past_the_bed_edge();
    let directory = tempfile::Builder::new()
        .prefix("gcode-printable-area-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    fs::write(&path, &input).unwrap();
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
    let exit = Some(if cfg!(unix) { 154 } else { -102 });
    assert_eq!(failure.captured.as_ref().unwrap().code, exit);
    let result: Value =
        serde_json::from_slice(&fs::read(output.join("result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], -102);
    assert_eq!(result["error_string"], ERROR);
    let retained = fs::read(output.join("plate_1.gcode")).unwrap();

    let overrides = ConfigOverrides::default();
    let bytes = crate::runner::application::slice_project_plate_validated(
        &input,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    )
    .await;
    let processed = crate::runner::application::slice_project_plate_processed_validated(
        &input,
        GenerationMetadata::default(),
        None,
        &overrides,
        SlicerContext::new(0),
    )
    .await;
    let Err(byte_error) = bytes else {
        panic!("byte entry accepted");
    };
    let Err(processed_error) = processed else {
        panic!("processed entry accepted");
    };
    for error in [byte_error, processed_error] {
        assert_eq!(error.to_string(), ERROR);
        assert!(
            !matches!(error, SliceError::InvalidInput(_)),
            "must expose a typed processor failure"
        );
    }

    let cli_path = directory.join("ares.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(cli.status.code(), exit, "CLI at {directory:?}");
    crate::golden::compare_ordered_bytes_generator_only(&retained, &fs::read(cli_path).unwrap())
        .unwrap_or_else(|error| panic!("retained G-code at {directory:?}: {error}"));
}
