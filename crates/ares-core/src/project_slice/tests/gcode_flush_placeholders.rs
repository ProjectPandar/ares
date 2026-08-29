use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn flush_placeholders_use_configured_values_with_zero_fallbacks() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert(
        "machine_start_gcode".to_owned(),
        json!(";FLUSH {flush_volumetric_speeds[0]},{flush_volumetric_speeds[1]} {flush_temperatures[0]},{flush_temperatures[1]}"),
    );
    settings.insert("machine_end_gcode".to_owned(), json!(";END"));
    settings.insert(
        "filament_flush_volumetric_speed".to_owned(),
        json!(["18", "0"]),
    );
    settings.insert(
        "filament_max_volumetric_speed".to_owned(),
        json!(["15", "20"]),
    );
    settings.insert("filament_flush_temp".to_owned(), json!(["230", "0"]));
    settings.insert(
        "nozzle_temperature_range_high".to_owned(),
        json!(["250", "260"]),
    );
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(
        output.lines().find(|line| line.starts_with(";FLUSH ")),
        Some(";FLUSH 18,18 230,230")
    );
}
