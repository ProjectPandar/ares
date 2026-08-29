use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn first_filament_placeholders_keep_unused_physical_hotend_at_minus_one() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert(
        "machine_start_gcode".to_owned(),
        json!(";FIRST {first_filaments[0]},{first_filaments[1]} NON {first_non_support_filaments[0]},{first_non_support_filaments[1]}"),
    );
    settings.insert("machine_end_gcode".to_owned(), json!(";END"));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.lines().any(|line| line == ";FIRST -1,0 NON -1,0"));
}
