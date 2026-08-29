use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn missing_startup_temperatures_emit_before_custom_role_and_template() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("printer_model".to_owned(), json!("Folgertech FT-5"));
    settings.insert("machine_start_gcode".to_owned(), json!(";MACHINE-START"));
    settings.insert("machine_end_gcode".to_owned(), json!(";MACHINE-END"));
    settings.insert("filament_start_gcode".to_owned(), json!(["", ""]));
    settings.insert("auxiliary_fan".to_owned(), json!("0"));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();

    let bed = position_with(&lines, |line| line.starts_with("M190 S"));
    let nozzle = position_with(&lines, |line| line.starts_with("M104 S"));
    let custom = position_with(&lines, |line| *line == ";TYPE:Custom");
    let machine_start = position_with(&lines, |line| *line == ";MACHINE-START");
    assert!(bed < nozzle);
    assert!(nozzle < custom);
    assert!(custom < machine_start);
}

fn position_with(lines: &[&str], predicate: impl Fn(&&str) -> bool) -> usize {
    lines
        .iter()
        .position(predicate)
        .expect("expected G-code line")
}
