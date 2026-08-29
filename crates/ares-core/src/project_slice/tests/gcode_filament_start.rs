use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn filament_start_position_uses_assignments_not_machine_moves() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("printer_model".to_owned(), json!("SeeMeCNC Artemis"));
    settings.insert(
        "machine_start_gcode".to_owned(),
        json!("{position[0] = 2}\nG90\nG1 Z1"),
    );
    settings.insert(
        "filament_start_gcode".to_owned(),
        json!([
            ";FILAMENT-START\n{if position[0] != 2 || position[2] > first_layer_height}\n;POSITION-WRONG\n{else} \n;POSITION-CLEARED\n{endif}",
            ""
        ]),
    );
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();
    let first_layer = output.find(";LAYER_CHANGE").unwrap();
    let preamble = &output[..first_layer];

    assert!(preamble.contains(";FILAMENT-START\n \n;POSITION-CLEARED\n"));
    assert!(!preamble.contains(";POSITION-WRONG"));
}

#[tokio::test]
async fn bbl_filament_start_renders_before_vt_without_disabled_aux_fan() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("machine_start_gcode".to_owned(), json!(";MACHINE-START"));
    settings.insert("machine_end_gcode".to_owned(), json!(";MACHINE-END"));
    settings.insert(
        "filament_start_gcode".to_owned(),
        json!([";FILAMENT-START\n\n", ""]),
    );
    settings.insert("auxiliary_fan".to_owned(), json!("0"));
    settings.insert("enable_pressure_advance".to_owned(), json!(["0", "0"]));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let output = std::str::from_utf8(&output).unwrap();
    let first_layer = output.find("; CHANGE_LAYER").unwrap();
    let preamble = &output[..first_layer];

    assert!(preamble.contains(";FILAMENT-START\n\n;VT0\n"));
    assert!(!preamble.lines().any(|line| line == "M106 P2 S0"));
}
