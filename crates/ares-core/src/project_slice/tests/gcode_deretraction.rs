use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn zero_deretraction_speed_falls_back_to_retraction_speed() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("deretraction_speed".to_owned(), json!(["0", "0"]));
    settings.insert("retraction_speed".to_owned(), json!(["30", "30"]));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("G1 E") && line.ends_with(" F0"))
    );
    assert!(output.lines().any(|line| {
        line.starts_with("G1 E") && !line.contains("E-") && line.ends_with(" F1800")
    }));
}
