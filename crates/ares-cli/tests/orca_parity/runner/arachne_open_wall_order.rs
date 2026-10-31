//! `PerimeterGenerator.cpp:2326–2328`: open candidates precede closed candidates.
use super::{ExportedCase, OrcaRunner, application};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};

#[test]
fn arachne_open_wall_c0_neck0_25_r0() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r0.3mf"
        ),
        "6c5c67976071ea79e663b8a4d428f3f51202b3fb9821a49fbe6b9da1f399a07d",
        "C0-neck0.25-r0",
        "aligned_back",
    );
}

#[test]
fn arachne_open_wall_c0_neck0_25_r17() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r17.3mf"
        ),
        "98e80f92caedbee7824eb587e026f6e786d9a02b2a0af7c8f88726b688af560d",
        "C0-neck0.25-r17",
        "aligned_back",
    );
}

#[test]
fn arachne_open_wall_c0_neck0_25_r43() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/arachne-open-wall-order/C0-neck0.25-r43.3mf"
        ),
        "87833a359652053d8b8208485deb16db6e135eca625864d33e758da1f9a37eef",
        "C0-neck0.25-r43",
        "aligned_back",
    );
}

#[test]
fn arachne_open_wall_nearest_control_neck0_25_r0() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/arachne-open-wall-order/nearest-control-neck0.25-r0.3mf"
        ),
        "ce5caae069fdf344382fc395f1ec85f576623bd33fbe483706a9b980705bcc41",
        "nearest-control-neck0.25-r0",
        "nearest",
    );
}

#[test]
fn arachne_open_wall_nearest_control_neck0_25_r17() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/arachne-open-wall-order/nearest-control-neck0.25-r17.3mf"
        ),
        "3e33655889bf7acc578d2a36f4b10611b23119f3a827f9f88bd51cea53612cb9",
        "nearest-control-neck0.25-r17",
        "nearest",
    );
}

#[test]
fn arachne_open_wall_nearest_control_neck0_25_r43() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/arachne-open-wall-order/nearest-control-neck0.25-r43.3mf"
        ),
        "82b1200bdbedba3ec487dbbf3ab61f75b9fdc4fd1ffe5a06ad33d1f4f3d05601",
        "nearest-control-neck0.25-r43",
        "nearest",
    );
}

#[test]
fn arachne_open_wall_wide_neck_control() {
    compare(
        include_bytes!(
            "../../../../../tests/parity/validation/seam-aligned-back-single-control.3mf"
        ),
        "08f309f90e688f48d1cf0a012e45dff6ff484d40bca28cbb6adeff603c3285cc",
        "wide-neck-control",
        "aligned_back",
    );
}

fn compare(project: &[u8], sha256: &str, label: &str, mode: &str) {
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
        ("wall_generator", "arachne"),
        ("wall_loops", "1"),
        ("initial_layer_print_height", "0.2"),
        ("layer_height", "0.2"),
    ] {
        application::verify_value(project, key, &json!(value)).unwrap();
    }
    let loaded = ares_core::load_project(project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), 1);
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut metadata = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut metadata)
        .unwrap();
    let object = metadata
        .split_once("<object id=\"2\">")
        .unwrap()
        .1
        .split_once("</object>")
        .unwrap()
        .0;
    assert!(object.contains(&format!(
        "<metadata key=\"seam_position\" value=\"{mode}\"/>"
    )));
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, project).unwrap();
    let output_dir = runner.work.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .rerun_reference(&ExportedCase {
            label: format!("arachne-open-wall-order/{label}"),
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
