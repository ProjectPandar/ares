//! PrintApply.cpp:342-395,958-1027; PrintObjectSlice.cpp:100-266,1149-1199.
//! Inputs are authored; complete Native streams are produced at runtime.
use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;

use crate::runner::{ExportedCase, OrcaRunner};

#[path = "spiral_layer_ranges.rs"]
mod spiral;

macro_rules! input {
    ($name:literal) => {
        include_bytes!(concat!(
            "../../../../../tests/parity/object-layer-ranges/",
            $name,
            ".3mf"
        ))
    };
}

#[tokio::test]
async fn object_explicit_height_density_range_preserves_complete_native_output() {
    compare("density", input!("density"), "baseline", input!("baseline")).await;
}

#[tokio::test]
async fn object_explicit_height_width_range_preserves_complete_native_output() {
    compare("width", input!("width"), "baseline", input!("baseline")).await;
}

#[tokio::test]
async fn object_explicit_height_speed_range_preserves_complete_native_output() {
    compare("speed", input!("speed"), "baseline", input!("baseline")).await;
}

#[tokio::test]
async fn object_retained_profile_mixed_ranges_preserve_complete_native_output() {
    compare(
        "profile-mixed",
        input!("profile-mixed"),
        "profile-baseline",
        input!("profile-baseline"),
    )
    .await;
}

#[tokio::test]
async fn object_retained_profile_region_only_ranges_preserve_complete_native_output() {
    compare(
        "profile-region-only",
        input!("profile-region-only"),
        "profile-baseline",
        input!("profile-baseline"),
    )
    .await;
}

#[tokio::test]
async fn object_profile_range_overlap_preserves_complete_native_output() {
    compare(
        "profile-overlap",
        input!("profile-overlap"),
        "profile-baseline",
        input!("profile-baseline"),
    )
    .await;
}

#[tokio::test]
async fn object_profile_range_gap_preserves_complete_native_output() {
    compare(
        "profile-gap",
        input!("profile-gap"),
        "profile-baseline",
        input!("profile-baseline"),
    )
    .await;
}

#[tokio::test]
async fn object_profile_range_epsilon_preserves_complete_native_output() {
    compare(
        "profile-epsilon",
        input!("profile-epsilon"),
        "profile-baseline",
        input!("profile-baseline"),
    )
    .await;
}

#[tokio::test]
async fn object_range_overrides_model_part_width_after_volume_config_like_native() {
    compare(
        "volume-precedence",
        input!("volume-precedence"),
        "volume-baseline",
        input!("volume-baseline"),
    )
    .await;
}

#[tokio::test]
async fn object_range_retains_modifier_parent_configuration_like_native() {
    compare(
        "modifier-range",
        input!("modifier-range"),
        "modifier-baseline",
        input!("modifier-baseline"),
    )
    .await;
}

async fn compare(label: &str, input: &[u8], baseline_label: &str, baseline_input: &[u8]) {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/parity/object-layer-ranges/provenance.json"
    ))
    .unwrap();
    for (name, bytes) in [(label, input), (baseline_label, baseline_input)] {
        assert_eq!(
            crate::runner::cache_identity::raw_digest(bytes),
            manifest["variants"][name]["input_sha256"].as_str().unwrap()
        );
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("object-layer-range-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let baseline = reference(&runner, "baseline", baseline_input, &directory);
    let baseline_actual = slice(baseline_input).await;
    record(
        &directory.join("baseline"),
        baseline_label,
        baseline_input,
        &baseline,
        &baseline_actual,
    );
    let baseline_actual =
        baseline_actual.unwrap_or_else(|error| panic!("inactive: {error}; {directory:?}"));
    if label == "modifier-range" {
        super::first_travel_acceleration_ordering::assert_first_travel_prefix(
            &directory.join("baseline"),
            baseline_label,
            &baseline,
            &baseline_actual,
        );
    }
    crate::golden::compare_ordered_bytes_generator_only(&baseline, &baseline_actual)
        .unwrap_or_else(|error| panic!("inactive: {error}; {directory:?}"));
    let native = reference(&runner, label, input, &directory);
    assert_ne!(
        motion_lines(&baseline),
        motion_lines(&native),
        "range must alter actual Native motion; {directory:?}"
    );
    let actual = slice(input).await;
    record(&directory, label, input, &native, &actual);
    let actual = actual.unwrap_or_else(|error| panic!("{error}; {directory:?}"));
    assert_eq!(layer_lines(&native), layer_lines(&actual), "{directory:?}");
    crate::golden::compare_ordered_bytes_generator_only(&native, &actual)
        .unwrap_or_else(|error| panic!("{error}; {directory:?}"));
}

fn reference(runner: &OrcaRunner, label: &str, input: &[u8], root: &std::path::Path) -> Vec<u8> {
    let directory = root.join(label);
    fs::create_dir(&directory).unwrap();
    let path = directory.join("input.3mf");
    fs::write(&path, input).unwrap();
    runner
        .slice_case(&ExportedCase {
            label: format!("object-layer-range/{label}"),
            project: input.to_vec(),
            project_path: path,
            output_dir: directory,
        })
        .unwrap()
        .reference
}

async fn slice(input: &[u8]) -> Result<Vec<u8>, ares_core::SliceError> {
    crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await
}

fn record(
    root: &std::path::Path,
    label: &str,
    input: &[u8],
    native: &[u8],
    actual: &Result<Vec<u8>, ares_core::SliceError>,
) {
    fs::write(root.join("native.gcode"), native).unwrap();
    if let Ok(bytes) = actual {
        fs::write(root.join("ares.gcode"), bytes).unwrap();
    }
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"input_sha256":crate::runner::cache_identity::raw_digest(input),
            "native_sha256":crate::runner::cache_identity::raw_digest(native),
            "native_layers":layer_lines(native),
            "ares":actual.as_ref().map(Vec::len).map_err(ToString::to_string),
        }))
        .unwrap(),
    )
    .unwrap();
}

fn layer_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
        .collect()
}

fn motion_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("G0 ") || line.starts_with("G1 "))
        .collect()
}
