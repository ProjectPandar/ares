use serde_json::json;

use super::{input, support::Case};

#[tokio::test]
async fn protected_layer_difference_edits_existing_system_without_overwriting_reference() {
    let catalog = input::catalog(&[input::process("Protected Process", "0.2", "system")], &[]);
    let mut settings = input::settings("Protected Process", "0.27");
    settings["different_settings_to_system"] = json!(["layer_height", "", ""]);
    let Some(case) = Case::with_catalog("selection-protected-layer", settings, catalog) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Protected Process");
    assert_eq!(native["selected"]["system"], true);
    assert_eq!(
        native["selected"]["config"]["layer_height"]["serialized"],
        "0.2"
    );
    assert_eq!(
        native["edited"]["config"]["layer_height"]["serialized"],
        "0.27"
    );
    assert_eq!(native["dirty"], true);
    assert_eq!(
        native["dirty_options"],
        json!(["layer_height", "print_settings_id"])
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
        &json!({"passed": true, "protected_layer": true}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.27");
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let process = &view["preset_selection"]["process"];
    assert_eq!(process["outcome"], "existing_edited");
    assert_eq!(process["selected"]["name"], "Protected Process");
    assert_eq!(process["selected"]["config"]["layer_height"], "0.2");
    assert_eq!(process["edited"]["config"]["layer_height"], "0.27");
    assert_eq!(process["dirty_options"], native["dirty_options"]);
    assert_eq!(view["settings"]["layer_height"], "0.27");
}

#[tokio::test]
async fn protected_non_panel_postprocessing_option_participates_in_typed_equality() {
    let mut parent = input::process("Hook Parent", "0.2", "system");
    parent["post_process"] = json!(["base-hook"]);
    let mut user = input::process("Hook User", "0.2", "user");
    user["inherits"] = json!("Hook Parent");
    user["post_process"] = json!(["stored-hook"]);
    let catalog = input::catalog(&[parent], &[user]);
    let mut settings = input::settings("Hook User", "0.2");
    settings["inherits_group"] = json!(["Hook Parent", "", ""]);
    settings["different_settings_to_system"] = json!(["post_process", "", ""]);
    settings["post_process"] = json!(["project-hook"]);
    let Some(case) = Case::with_catalog("selection-protected-postprocess", settings, catalog)
    else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Hook User");
    assert_eq!(native["dirty"], true);
    assert_eq!(
        native["dirty_options"],
        json!(["post_process", "print_settings_id"])
    );
    assert_ne!(
        native["selected"]["config"]["post_process"],
        native["edited"]["config"]["post_process"]
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "non_panel_post_process_differs": true}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.2");
    assert_eq!(cli["post_process"], json!(["project-hook"]));
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let process = &view["preset_selection"]["process"];
    assert_eq!(process["outcome"], "existing_edited");
    assert_eq!(process["selected"]["name"], "Hook User");
    assert_eq!(
        process["selected"]["config"]["post_process"],
        json!(["stored-hook"])
    );
    assert_eq!(
        process["edited"]["config"]["post_process"],
        json!(["project-hook"])
    );
    assert_eq!(view["settings"]["post_process"], json!(["project-hook"]));
    assert_eq!(process["dirty_options"], native["dirty_options"]);
}
