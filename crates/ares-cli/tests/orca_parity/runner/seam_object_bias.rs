//! `SeamPlacer.cpp:1431–1441,165–167`: visibility bias is per PrintObject.

use super::{ExportedCase, OrcaRunner, application};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};

#[test]
fn aligned_back_object_does_not_bias_nearest_object() {
    compare(
        include_bytes!("../../../../../tests/parity/validation/seam-mixed-object-bias.3mf"),
        "3ab40b2ea7cf55598aecf52b0de61404c7b890737a2b8e638a47f6e2acd2c9f8",
        "mixed",
        &[(2, "aligned_back"), (3, "nearest")],
    );
}

#[test]
fn two_nearest_objects_preserve_complete_output() {
    compare(
        include_bytes!("../../../../../tests/parity/validation/seam-two-nearest-control.3mf"),
        "9df9c0de9faf1ab6854c74b58d372f4f78e9a4d773987edd481b428ddf522115",
        "two-nearest-control",
        &[(2, "nearest"), (3, "nearest")],
    );
}

#[test]
fn single_aligned_back_preserves_complete_output() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/seam-aligned-back-single-control.3mf"
        ),
        "08f309f90e688f48d1cf0a012e45dff6ff484d40bca28cbb6adeff603c3285cc",
        "single-aligned-back-control",
        &[(2, "aligned_back")],
    );
}

fn compare(project: &[u8], sha256: &str, label: &str, object_modes: &[(u32, &str)]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(
        Sha256::digest(project)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        sha256
    );
    for (key, value) in [
        ("initial_layer_print_height", "0.2"),
        ("layer_height", "0.2"),
        ("seam_position", "nearest"),
    ] {
        application::verify_value(project, key, &json!(value)).unwrap();
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut metadata = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut metadata)
        .unwrap();
    for (id, mode) in object_modes {
        let object = metadata
            .split_once(&format!("<object id=\"{id}\">"))
            .unwrap()
            .1
            .split_once("</object>")
            .unwrap()
            .0;
        assert!(object.contains(&format!(
            "<metadata key=\"seam_position\" value=\"{mode}\"/>"
        )));
    }
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, project).unwrap();
    let output_dir = runner.work.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .rerun_reference(&ExportedCase {
            label: format!("seam-object-bias/{label}"),
            project: project.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{}; artifacts: {:?}",
        outcome.detail, outcome.artifacts
    );
}
