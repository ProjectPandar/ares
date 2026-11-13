use serde_json::json;

use super::support::Case;

#[tokio::test]
async fn same_name_user_selection_precedes_an_equal_system_parent() {
    let catalog = json!({
        "vendors": [{
            "id": "Acme",
            "index": {
                "name": "Acme", "version": "02.05.00.00",
                "machine_model_list": [], "machine_list": [], "filament_list": [],
                "process_list": [{"name": "Parent Process", "sub_path": "process/parent.json"}]
            },
            "files": {"process/parent.json": {
                "type": "process", "name": "Parent Process", "from": "system",
                "instantiation": "true", "version": "02.05.00.00", "layer_height": "0.2"
            }}
        }],
        "imports": [{"profiles": [{
            "type": "process", "name": "User Process", "from": "user",
            "instantiation": "true", "version": "02.05.00.00",
            "inherits": "Parent Process", "layer_height": "0.3"
        }]}]
    });
    let settings = json!({
        "print_settings_id": "User Process",
        "printer_settings_id": "Missing Printer",
        "filament_settings_id": ["Missing Filament"],
        "filament_ids": ["fixture-filament"],
        "filament_colour": ["#AABBCC"],
        "filament_diameter": ["1.75"],
        "layer_height": "0.2",
        "nozzle_diameter": ["0.6"],
        "printable_area": ["0x0", "200x0", "200x200", "0x200"],
        "inherits_group": ["Parent Process", "", ""],
        "different_settings_to_system": ["", "", ""]
    });
    let mut missing_original = settings.clone();
    missing_original["print_settings_id"] = json!("Missing Child");
    let Some(case) = Case::with_native_controls(
        "selection-user-before-parent",
        settings,
        catalog,
        &[missing_original],
    ) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    let parent = &case.native["projects"][1]["result"]["process"];
    assert_eq!(native["selected"]["name"], "User Process");
    assert_eq!(native["selected"]["system"], false);
    assert_eq!(native["selected"]["external"], false);
    assert_eq!(
        native["selected"]["config"]["layer_height"]["serialized"],
        "0.3"
    );
    assert_eq!(
        native["edited"]["config"]["layer_height"]["serialized"],
        "0.2"
    );
    assert_eq!(native["dirty"], true);
    assert_eq!(
        native["dirty_options"],
        json!(["layer_height", "print_settings_id"])
    );
    assert_eq!(parent["selected"]["name"], "Parent Process");
    assert_eq!(parent["selected"]["system"], true);
    assert_eq!(parent["dirty"], false);
    assert_eq!(
        parent["edited"]["config"]["layer_height"]["serialized"],
        "0.2"
    );
    assert_eq!(
        native["records"].as_array().unwrap().len(),
        case.native["before"]["process"]["records"]
            .as_array()
            .unwrap()
            .len()
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "equal_parent_control": true}),
    );

    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.2");
    assert!(cli.get("preset_selection").is_none());
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    assert_eq!(view["settings"]["layer_height"], "0.2");
    let process = &view["preset_selection"]["process"];
    assert_eq!(process["outcome"], "existing_edited");
    assert_eq!(process["selected"]["name"], native["selected"]["name"]);
    assert_eq!(process["selected"]["config"]["layer_height"], "0.3");
    assert_eq!(process["edited"]["config"]["layer_height"], "0.2");
    assert_eq!(process["selected"]["config"]["print_settings_id"], "");
    assert_eq!(
        process["edited"]["config"]["print_settings_id"],
        "User Process"
    );
    assert_eq!(process["dirty"], native["dirty"]);
    assert_eq!(process["dirty_options"], native["dirty_options"]);
}

#[tokio::test]
async fn selecting_hidden_default_match_makes_only_that_selection_visible() {
    let catalog = super::input::catalog(
        &[super::input::process("Other Process", "0.2", "system")],
        &[],
    );
    let Some(case) = Case::with_catalog(
        "selection-hidden-default",
        super::input::settings("Default Setting", "0.2"),
        catalog,
    ) else {
        return;
    };
    let before = &case.native["before"]["process"]["records"];
    let hidden = before
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["name"] == "Default Setting")
        .unwrap();
    assert_eq!(hidden["visible"], false);
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Default Setting");
    assert_eq!(native["selected"]["visible"], true);
    assert_eq!(native["selected"]["external"], false);
    assert_eq!(native["dirty"], false);
    assert_eq!(
        native["records"].as_array().unwrap().len(),
        before.as_array().unwrap().len()
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "hidden_selection_made_visible": true}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.2");
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    assert_eq!(
        view["preset_selection"]["process"]["outcome"],
        "existing_clean"
    );
    assert_eq!(
        view["preset_selection"]["process"]["selected"]["name"],
        "Default Setting"
    );
    assert_eq!(
        view["preset_selection"]["process"]["selected"]["visible"],
        true
    );
    assert_eq!(view["preset_selection"]["process"]["dirty"], false);
}

#[tokio::test]
async fn typed_equal_user_match_does_not_import_ignored_setting_id() {
    let catalog =
        super::input::catalog(&[], &[super::input::process("Typed User", "0.20", "user")]);
    let Some(case) = Case::with_catalog(
        "selection-user-typed-equal",
        super::input::settings("Typed User", "0.2"),
        catalog,
    ) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Typed User");
    assert_eq!(native["selected"]["system"], false);
    assert_eq!(native["dirty"], false);
    assert_eq!(native["dirty_options"], json!([]));
    assert_eq!(
        native["selected"]["config"]["layer_height"]["serialized"],
        "0.2"
    );
    assert_eq!(native["edited"]["config"], native["selected"]["config"]);
    assert_eq!(
        native["edited"]["config"]["print_settings_id"]["serialized"],
        ""
    );
    assert_eq!(
        native["records"].as_array().unwrap().len(),
        case.native["before"]["process"]["records"]
            .as_array()
            .unwrap()
            .len()
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "typed_equal": true, "ignored_id_not_imported": true}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.2");
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let process = &view["preset_selection"]["process"];
    assert_eq!(process["outcome"], "existing_clean");
    assert_eq!(process["selected"]["name"], "Typed User");
    assert_eq!(process["dirty"], false);
    assert_eq!(process["dirty_options"], json!([]));
    assert_eq!(process["edited"]["config"]["print_settings_id"], "");
    assert_eq!(
        process["edited"]["config"]["layer_height"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        0.2
    );
    assert_eq!(
        view["settings"]["layer_height"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        0.2
    );
}
