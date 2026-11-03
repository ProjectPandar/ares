//! Slicing.cpp:234-305,807-870 and PrintObject.cpp:3805-3834, object range heights.
//! Native references use the existing slice_case cache; optional missing Native
//! returns before producing comparison artifacts. No frozen fallback.
#[path = "object_layer_height_ranges_runtime.rs"]
mod runtime;

use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{ExportedCase, OrcaRunner};

const INPUT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/object-range-layer-height.3mf");

#[tokio::test]
async fn object_range_layer_height_changes_layer_schedule_and_overhang_seam() {
    compare_range(
        "overhang-seam",
        INPUT,
        "fc938f743bed802de2691134f232015e715d525fc87f9f4b33ce6a51785b9b5f",
        &[
            ";Z:0.3",
            ";HEIGHT:0.3",
            ";Z:0.4",
            ";HEIGHT:0.1",
            ";Z:0.5",
            ";HEIGHT:0.1",
            ";Z:0.6",
            ";HEIGHT:0.1",
        ],
    )
    .await;
}

async fn compare_range(label: &str, input: &[u8], expected_hash: &str, expected_layers: &[&str]) {
    let hash = Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hash, expected_hash);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
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
    fs::write(&project_path, input).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let reference = runner
        .slice_case(&ExportedCase {
            label: format!("object-range-height/{label}"),
            project: input.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap()
        .reference;
    fs::write(directory.join("native.gcode"), &reference).unwrap();
    let layers = std::str::from_utf8(&reference)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
        .collect::<Vec<_>>();
    assert_eq!(layers, expected_layers);
    let actual = crate::runner::application::slice_project_plate_validated(
        input,
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
            "case": label,
            "input_sha256": hash,
            "native_layers": layers,
            "native_bytes": reference.len(),
            "native_sha256": crate::runner::cache_identity::raw_digest(&reference),
            "native_artifacts": runner.work,
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
async fn object_range_height_boundaries_match_native_full_output() {
    for (label, input, hash, layers) in [
        (
            "overlap",
            include_bytes!(
                "../../../../../tests/parity/validation/object-range-layer-height-overlap.3mf"
            )
            .as_slice(),
            "7f078c18fa8eb8119f2215a5c98afbaa629e9946c430259ca962cd1c929bab11",
            &[
                ";Z:0.3",
                ";HEIGHT:0.3",
                ";Z:0.4",
                ";HEIGHT:0.1",
                ";Z:0.5",
                ";HEIGHT:0.1",
                ";Z:0.65",
                ";HEIGHT:0.15",
            ][..],
        ),
        (
            "gap-top",
            include_bytes!(
                "../../../../../tests/parity/validation/object-range-layer-height-gap-top.3mf"
            )
            .as_slice(),
            "9a03a0528b31b292a0494c76fdb985c788c91b3928f6176eadf6a85eeefc65c7",
            &[
                ";Z:0.3",
                ";HEIGHT:0.3",
                ";Z:0.5",
                ";HEIGHT:0.2",
                ";Z:0.65",
                ";HEIGHT:0.15",
            ][..],
        ),
        (
            "epsilon",
            include_bytes!(
                "../../../../../tests/parity/validation/object-range-layer-height-epsilon.3mf"
            )
            .as_slice(),
            "ea1ebb3e63343fb0ff3140623a15ea3553995093dbadbec670199778d4ab4099",
            &[
                ";Z:0.3",
                ";HEIGHT:0.3",
                ";Z:0.5",
                ";HEIGHT:0.2",
                ";Z:0.65",
                ";HEIGHT:0.15",
            ][..],
        ),
    ] {
        compare_range(label, input, hash, layers).await;
    }
}

#[tokio::test]
async fn mixed_height_and_region_only_ranges_keep_the_unsupported_boundary() {
    let input = include_bytes!(
        "../../../../../tests/parity/validation/object-range-layer-height-mixed.3mf"
    );
    let result = crate::runner::application::slice_project_plate_validated(
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
