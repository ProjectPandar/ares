use super::filament_map::verify;
use serde_json::json;
use std::io::{Cursor, Write};

#[test]
fn filament_map_storage_failure_is_io_not_a_value_mismatch() {
    let work = tempfile::NamedTempFile::new().unwrap();
    let runner = super::OrcaRunner {
        bin: "unused".into(),
        work: work.path().to_owned(),
        cache: work.path().to_owned(),
    };
    let exported = super::ExportedCase {
        label: "storage-failure".to_owned(),
        project: project(
            "Manual",
            "<metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>",
        ),
        project_path: "unused".into(),
        output_dir: "unused".into(),
    };
    let error = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .err()
        .unwrap();
    assert_eq!(error.stage, super::stages::Stage::Application);
    assert_eq!(error.kind, super::stages::FailureKind::Io);
}

fn project(global: &str, plate: &str) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in [
        (
            "Metadata/project_settings.config",
            serde_json::to_vec(&json!({"filament_map_mode": global})).unwrap(),
        ),
        (
            "Metadata/model_settings.config",
            format!("<config><plate>{plate}</plate></config>").into_bytes(),
        ),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[test]
fn filament_map_application_requires_matching_global_and_plate_owners() {
    for mode in ["Auto For Flush", "Auto For Match", "Manual"] {
        let plate = format!("<metadata key=\"filament_map_mode\" value=\"{mode}\"/>");
        let proof = verify(&project(mode, &plate), &json!(mode)).unwrap();
        assert_eq!(proof["effective"], mode);
        assert_eq!(proof["plate"]["effective"], mode);
    }
    for (global, plate) in [
        ("Manual", ""),
        (
            "Manual",
            "<metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>",
        ),
        (
            "Auto For Flush",
            "<metadata key=\"filament_map_mode\" value=\"Manual\"/>",
        ),
    ] {
        assert!(verify(&project(global, plate), &json!("Manual")).is_err());
    }
}
