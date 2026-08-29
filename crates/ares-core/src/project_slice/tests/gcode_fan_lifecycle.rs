use serde_json::json;

use super::support::{KsrArchive, metadata};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn non_bbl_fans_follow_orca_print_start_layer_and_finish_order() {
    let mut archive = KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    let settings = settings.as_object_mut().unwrap();
    settings.insert("printer_model".to_owned(), json!("SeeMeCNC Artemis"));
    settings.insert("machine_start_gcode".to_owned(), json!(";MACHINE-START"));
    settings.insert("machine_end_gcode".to_owned(), json!(";MACHINE-END"));
    settings.insert("filament_start_gcode".to_owned(), json!([" ", " "]));
    settings.insert("filament_end_gcode".to_owned(), json!(["", ""]));
    settings.insert("support_air_filtration".to_owned(), json!("1"));
    settings.insert("activate_air_filtration".to_owned(), json!(["1", "1"]));
    settings.insert(
        "activate_air_filtration_during_print".to_owned(),
        json!(["1", "1"]),
    );
    settings.insert(
        "during_print_exhaust_fan_speed".to_owned(),
        json!(["100", "40"]),
    );
    settings.insert("auxiliary_fan".to_owned(), json!("1"));
    settings.insert(
        "additional_cooling_fan_speed".to_owned(),
        json!(["70", "70"]),
    );
    settings.insert(
        "close_additional_fan_first_x_layers".to_owned(),
        json!(["0", "0"]),
    );
    settings.insert("close_fan_the_first_x_layers".to_owned(), json!(["0", "0"]));
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(&archive.bytes(), metadata())
        .await
        .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();

    let machine_start = position(&lines, ";MACHINE-START");
    let exhaust_on = position(&lines, "M106 P3 S255");
    let flavor_preamble = position(&lines, "G90");
    let auxiliary_on = position(&lines, "M106 P2 S178");
    let first_layer = position(&lines, ";LAYER_CHANGE");
    assert!(machine_start < exhaust_on);
    assert!(exhaust_on < flavor_preamble);
    assert!(flavor_preamble < auxiliary_on);
    assert!(auxiliary_on < first_layer);

    let last_layer = lines
        .iter()
        .rposition(|line| *line == ";LAYER_CHANGE")
        .unwrap();
    let part_off = lines.iter().rposition(|line| *line == "M106 S0").unwrap();
    let auxiliary_off = lines
        .iter()
        .rposition(|line| *line == "M106 P2 S0")
        .unwrap();
    let machine_end = position(&lines, ";MACHINE-END");
    assert!(last_layer < part_off);
    assert!(part_off < auxiliary_off);
    assert!(auxiliary_off < machine_end);
    assert!(!lines.iter().any(|line| line.starts_with("M981 ")));
}

fn position(lines: &[&str], expected: &str) -> usize {
    lines
        .iter()
        .position(|line| *line == expected)
        .unwrap_or_else(|| panic!("missing {expected:?}"))
}
