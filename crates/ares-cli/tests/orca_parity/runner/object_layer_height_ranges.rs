//! Slicing.cpp:234-305,807-870 and PrintObject.cpp:3805-3834, object range heights.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{ExportedCase, OrcaRunner};

const INPUT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/object-range-layer-height.3mf");
const GOLDEN: &[u8] = include_bytes!(
    "../../../../../tests/parity/validation/object-range-layer-height.native-emitted.gcode"
);

#[tokio::test]
async fn object_range_layer_height_changes_layer_schedule_and_overhang_seam() {
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        hash,
        "fc938f743bed802de2691134f232015e715d525fc87f9f4b33ce6a51785b9b5f"
    );
    let mut builder = tempfile::Builder::new();
    builder.prefix("object-range-height-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, INPUT).unwrap();
    let reference = if let Some(runner) = OrcaRunner::from_env().unwrap() {
        let output_dir = directory.join("native");
        fs::create_dir(&output_dir).unwrap();
        runner
            .rerun_reference(&ExportedCase {
                label: "object-range-height/overhang-seam".into(),
                project: INPUT.to_vec(),
                project_path,
                output_dir,
            })
            .unwrap()
            .reference
    } else {
        GOLDEN.to_vec()
    };
    fs::write(directory.join("native.gcode"), &reference).unwrap();
    let layers = std::str::from_utf8(&reference)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
        .collect::<Vec<_>>();
    assert_eq!(
        layers,
        [
            ";Z:0.3",
            ";HEIGHT:0.3",
            ";Z:0.4",
            ";HEIGHT:0.1",
            ";Z:0.5",
            ";HEIGHT:0.1",
            ";Z:0.6",
            ";HEIGHT:0.1"
        ]
    );
    let actual = ares_core::slice_project_plate_validated(
        INPUT,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(gcode) = &actual {
        fs::write(directory.join("ares.gcode"), gcode).unwrap();
    }
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": hash,
            "native_layers": layers,
            "native_bytes": reference.len(),
            "ares": actual.as_ref().map(|bytes| bytes.len()).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
    let actual = actual.unwrap_or_else(|error| panic!("{error}; artifacts {directory:?}"));
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|error| panic!("{error}; artifacts {directory:?}"));
}
#[tokio::test]
async fn mixed_height_and_region_only_ranges_keep_the_unsupported_boundary() {
    let input = include_bytes!(
        "../../../../../tests/parity/validation/object-range-layer-height-mixed.3mf"
    );
    let result = ares_core::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    assert_eq!(
        result.unwrap_err(),
        ares_core::SliceError::UnsupportedProjectFeature("layer_config_ranges".into())
    );
}
