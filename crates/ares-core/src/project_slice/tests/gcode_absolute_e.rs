use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn absolute_e_preamble_resets_and_depositions_accumulate() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("printer_model".to_owned(), json!("Eryone ER20"));
    settings.insert("machine_start_gcode".to_owned(), json!(";START"));
    settings.insert("machine_end_gcode".to_owned(), json!(";END"));
    settings.insert("filament_start_gcode".to_owned(), json!(["", ""]));
    settings.insert("use_relative_e_distances".to_owned(), json!("0"));
    settings.insert("exclude_object".to_owned(), json!("1"));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();
    assert!(output.contains("M82 ; use absolute distances for extrusion\nG92 E0\n"));

    let first_feature = output
        .find(";TYPE:Inner wall")
        .or_else(|| output.find("; FEATURE: Inner wall"))
        .unwrap();
    let values = output[first_feature..]
        .lines()
        .skip(1)
        .take_while(|line| !line.starts_with(";TYPE:") && !line.starts_with("; FEATURE: "))
        .filter(|line| line.starts_with("G1 X") && line.contains(" E"))
        .filter_map(extrusion_value)
        .take(3)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 3);
    assert!(
        values.windows(2).all(|pair| pair[0] < pair[1]),
        "{values:?}"
    );
}

fn extrusion_value(line: &str) -> Option<f64> {
    line.split_ascii_whitespace()
        .find_map(|word| word.strip_prefix('E')?.parse().ok())
}
