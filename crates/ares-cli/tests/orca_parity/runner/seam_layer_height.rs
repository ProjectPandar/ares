//! `SeamPlacer.cpp:1084–1091`: overhang uses actual current layer thickness.

use super::{ExportedCase, OrcaRunner, application};
use serde_json::json;
use sha2::{Digest, Sha256};

#[test]
fn second_layer_overhang_seam_uses_current_layer_height() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = include_bytes!(
        "../../../../../tests/parity/validation/seam-overhang-nonuniform-height.3mf"
    );
    assert_eq!(
        Sha256::digest(project)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6"
    );
    for (key, value) in [
        ("initial_layer_print_height", "0.3"),
        ("layer_height", "0.2"),
        ("seam_position", "nearest"),
    ] {
        application::verify_value(project, key, &json!(value)).unwrap();
    }
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, project).unwrap();
    let output_dir = runner.work.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .rerun_reference(&ExportedCase {
            label: "seam-height/second-layer-overhang".into(),
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
