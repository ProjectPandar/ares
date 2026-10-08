use std::fs;

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use sha2::{Digest, Sha256};

use super::{ExportedCase, OrcaRunner};

#[tokio::test]
async fn complex_region_range_within_epsilon_matches_complete_native_output() {
    if crate::native_environment::binary("ARES_ORCA_BIN").is_none() {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped(
            "Native runner unavailable for complex epsilon comparison",
        );
        return;
    };
    for (label, input) in [
        (
            "complex-control",
            include_bytes!(
                "../../../../../tests/parity/validation/object-range-epsilon-complex-control.3mf"
            )
            .as_slice(),
        ),
        (
            "complex-within-above",
            include_bytes!(
                "../../../../../tests/parity/validation/object-range-epsilon-complex-within.3mf"
            )
            .as_slice(),
        ),
    ] {
        let mut builder = tempfile::Builder::new();
        builder.prefix("range-epsilon-");
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
                label: format!("region-range-epsilon/{label}"),
                project: input.to_vec(),
                project_path,
                output_dir,
            })
            .unwrap()
            .reference;
        fs::write(directory.join("native.gcode"), &reference).unwrap();
        let actual = ares_core::slice_project_plate_validated(
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
        fs::write(
            directory.join("result.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "label": label,
                "input_sha256": Sha256::digest(input).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                "native_bytes": reference.len(),
                "ares": actual.as_ref().map(|bytes| bytes.len()).map_err(ToString::to_string),
            }))
            .unwrap(),
        )
        .unwrap();
        let actual =
            actual.unwrap_or_else(|error| panic!("{label}: {error}; artifacts {directory:?}"));
        crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
            .unwrap_or_else(|error| panic!("{label}: {error}; artifacts {directory:?}"));
    }
}
