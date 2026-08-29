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

#[tokio::test]
async fn chamber_wait_brackets_with_auxiliary_fan_before_machine_start() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("printer_model".to_owned(), json!("Creality K1_CFS-C"));
    settings.insert("machine_start_gcode".to_owned(), json!(";MACHINE-START"));
    settings.insert("machine_end_gcode".to_owned(), json!(";MACHINE-END"));
    settings.insert("auxiliary_fan".to_owned(), json!("1"));
    settings.insert(
        "activate_chamber_temp_control".to_owned(),
        json!(["1", "1"]),
    );
    settings.insert("chamber_temperature".to_owned(), json!(["35", "35"]));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();

    let custom = position_with(&lines, |line| *line == ";TYPE:Custom");
    let fan_on = position_with(&lines, |line| *line == "M106 P2 S255 ");
    let wait = position_with(&lines, |line| {
        *line == "M191 S35 ;set chamber_temperature and wait for it to be reached"
    });
    let fan_off = position_with(&lines, |line| *line == "M106 P2 S0 ");
    let machine_start = position_with(&lines, |line| *line == ";MACHINE-START");
    assert!(custom < fan_on);
    assert!(fan_on < wait);
    assert!(wait < fan_off);
    assert!(fan_off < machine_start);
}

fn position_with(lines: &[&str], predicate: impl Fn(&&str) -> bool) -> usize {
    lines
        .iter()
        .position(predicate)
        .expect("expected G-code line")
}
