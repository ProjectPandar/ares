//! PrintApply.cpp::generate_print_object_regions and PrintObjectSlice.cpp routing.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use sha2::{Digest, Sha256};

use super::{ExportedCase, OrcaRunner};

const INPUT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/object-range-wall-loops.3mf");

#[tokio::test]
async fn object_range_wall_loops_changes_only_the_selected_constant_height_interval() {
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        hash,
        "53050d3c87aa45b0eb66677237b49b7d900bcaa03b65541c4416bed5b19920ae"
    );
    let mut builder = tempfile::Builder::new();
    builder.prefix("range-wall-loops-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, INPUT).unwrap();
    if crate::native_environment::binary("ARES_ORCA_BIN").is_none() {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("Native runner unavailable for region range comparison");
        return;
    };
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let reference = runner
        .rerun_reference(&ExportedCase {
            label: "region-range/wall-loops".into(),
            project: INPUT.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap()
        .reference;
    fs::write(directory.join("native.gcode"), &reference).unwrap();
    let zs = std::str::from_utf8(&reference)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("; Z_HEIGHT: "))
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(zs.len(), 50);
    assert_eq!(zs[0], 0.2);
    assert_eq!(zs[49], 10.0);
    let actual = crate::runner::application::slice_project_plate_validated(
        INPUT,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(bytes) = &actual {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
    }
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": hash, "native_bytes": reference.len(), "layer_z": zs,
            "ares": actual.as_ref().map(|bytes| bytes.len()).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
    let actual = actual.unwrap_or_else(|error| panic!("{error}; artifacts {directory:?}"));
    super::first_travel_acceleration_ordering::assert_first_travel_prefix(
        &directory,
        "wall-loops",
        &reference,
        &actual,
    );
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|error| panic!("{error}; artifacts {directory:?}"));
}
