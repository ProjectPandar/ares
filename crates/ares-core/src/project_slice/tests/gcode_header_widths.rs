use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn support_material_width_precedes_first_layer_width_when_enabled() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("enable_support".to_owned(), json!("1"));
    settings.insert("support_line_width".to_owned(), json!("96%"));
    settings.insert("support_filament".to_owned(), json!("0"));
    settings.insert("nozzle_diameter".to_owned(), json!(["0.4", "0.4"]));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let top = position(&lines, "; top infill extrusion width = 0.42mm");
    let support = position(&lines, "; support material extrusion width = 0.38mm");
    let first = position(&lines, "; first layer extrusion width = 0.50mm");

    assert!(top < support);
    assert!(support < first);
}

fn position(lines: &[&str], expected: &str) -> usize {
    lines
        .iter()
        .position(|line| *line == expected)
        .unwrap_or_else(|| panic!("missing {expected}"))
}
