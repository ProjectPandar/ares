//! APA-R2 M producer admission: no PA repair may hide an Arachne baseline mismatch.

use crate::{OrcaRunner, runner};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn adaptive_pa_m_arachne_pa_off_control() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let project = include_bytes!(
        "../../../../../tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf"
    );
    let input_sha = Sha256::digest(project)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        input_sha,
        "6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d"
    );
    for (key, value) in [
        ("wall_generator", json!("arachne")),
        ("enable_pressure_advance", json!(["0"])),
        ("adaptive_pressure_advance", json!(["0"])),
        ("enable_prime_tower", json!("0")),
    ] {
        runner::application::verify_value(project, key, &value).unwrap();
    }
    let directory = tempfile::Builder::new()
        .prefix("adaptive-pa-m-off-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    // The admitted ZIP already has both toggles off; preserve its exact bytes.
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, project).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .rerun_reference(&runner::ExportedCase {
            label: "adaptive-pa/r2-m/arachne-pa-off-control".into(),
            project: project.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = crate::compare_case(&case);
    fs::write(
        directory.join("apa-r2-m-admission.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": input_sha,
            "status": outcome.status,
            "detail": outcome.detail,
            "artifacts": outcome.artifacts,
            "source": "PerimeterGenerator.cpp:370,555-566 -> GCode.cpp:6092-6095,6038",
            "scope": "PA-off producer baseline only; failure is not Adaptive PA RED"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "PA-off producer mismatch; retain and refer geometry, do not repair in APA-R2: {}",
        outcome.detail
    );
}
