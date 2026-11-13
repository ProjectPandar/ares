use serde_json::{Value, json};

use super::{input, support::Case};

#[tokio::test]
async fn protected_machine_limits_restore_normal_silent_pairs_by_variant() {
    mapped_limits(
        "selection-mapped-pairs",
        json!(["Direct Drive High Flow", "Direct Drive Standard"]),
        json!([1, 1]),
        json!(["11", "22", "33", "44"]),
        "33,44,11,22",
    )
    .await;
}

#[tokio::test]
async fn protected_child_layout_larger_than_parent_is_not_truncated() {
    mapped_limits(
        "selection-child-layout",
        json!(["Direct Drive Standard", "Direct Drive High Flow", "Bowden"]),
        json!([1, 1, 2]),
        json!(["11", "22", "33", "44", "55", "66"]),
        "11,22,33,44,55,66",
    )
    .await;
}

async fn mapped_limits(label: &str, variants: Value, ids: Value, limits: Value, expected: &str) {
    let parent = json!({
        "type": "machine", "name": "Parent Printer", "from": "user", "version": "02.05.00.00", "instantiation": "true",
        "printer_extruder_id": ["1", "1"], "printer_extruder_variant": ["Direct Drive Standard", "Direct Drive High Flow"],
        "machine_max_speed_x": ["100", "200", "300", "400"], "nozzle_diameter": ["0.4"]
    });
    let mut settings = input::settings("Missing Process", "0.2");
    settings["inherits_group"] = json!(["", "", "Parent Printer"]);
    settings["printer_extruder_id"] = ids;
    settings["printer_extruder_variant"] = variants;
    settings["machine_max_speed_x"] = limits;
    settings["different_settings_to_system"] = json!([
        "",
        "",
        "machine_max_speed_x;printer_extruder_id;printer_extruder_variant"
    ]);
    let Some(case) = Case::with_catalog(label, settings, input::catalog(&[], &[parent])) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["printer"];
    assert_eq!(native["selected"]["name"], "Parent Printer");
    assert_eq!(native["dirty"], true);
    assert_eq!(
        native["edited"]["config"]["machine_max_speed_x"]["serialized"],
        expected
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "machine_max_speed_x": expected}),
    );
    case.cli().await.unwrap();
    case.write("cli-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    assert_eq!(
        view["preset_selection"]["printer"]["outcome"],
        "existing_edited"
    );
    assert_eq!(
        view["preset_selection"]["printer"]["edited"]["config"]["machine_max_speed_x"],
        native["edited"]["config"]["machine_max_speed_x"]
    );
    assert_eq!(
        view["preset_selection"]["printer"]["edited"]["config"]["printer_extruder_variant"],
        native["edited"]["config"]["printer_extruder_variant"]
    );
}
