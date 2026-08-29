use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn file_start_gcode_renders_stats_before_the_header() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert(
        "file_start_gcode".to_owned(),
        json!(";FLAVOR:Marlin\n;TIME:{print_time_sec}\n;Filament used:{used_filament_length}m\n;Layer height:{layer_height}"),
    );
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .take(5)
        .collect::<Vec<_>>();

    assert_eq!(lines[0], ";FLAVOR:Marlin");
    assert!(lines[1].starts_with(";TIME:"));
    assert!(lines[2].starts_with(";Filament used:"));
    assert_eq!(lines[3], ";Layer height:0.2");
    assert_eq!(lines[4], "; HEADER_BLOCK_START");
    assert!(!lines.iter().any(|line| line.contains(['{', '}'])));
}
