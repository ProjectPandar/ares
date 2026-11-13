use serde_json::json;

use super::{input, support::Case};

#[tokio::test]
async fn renamed_system_original_uses_the_loaded_rename_map() {
    renamed_selection(false).await;
}

#[tokio::test]
async fn missing_original_can_select_a_renamed_system_parent() {
    renamed_selection(true).await;
}

async fn renamed_selection(use_parent: bool) {
    let mut preset = input::process("Current Process", "0.2", "system");
    preset["renamed_from"] = json!("Previous Process");
    let catalog = input::catalog(&[preset], &[]);
    let mut settings = input::settings(
        if use_parent {
            "Missing Child"
        } else {
            "Previous Process"
        },
        "0.2",
    );
    if use_parent {
        settings["inherits_group"] = json!(["Previous Process", "", ""]);
    }
    let Some(case) = Case::with_catalog(
        if use_parent {
            "selection-renamed-parent"
        } else {
            "selection-renamed-original"
        },
        settings,
        catalog,
    ) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Current Process");
    assert_eq!(native["selected"]["system"], true);
    assert_eq!(native["selected"]["external"], false);
    assert_eq!(native["dirty"], false);
    assert_eq!(
        native["records"].as_array().unwrap().len(),
        case.native["before"]["process"]["records"]
            .as_array()
            .unwrap()
            .len()
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "renamed_parent": use_parent}),
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
    assert_eq!(process["selected"]["name"], native["selected"]["name"]);
    assert_eq!(process["dirty"], false);
    assert_eq!(view["settings"]["print_settings_id"], "Current Process");
}

#[tokio::test]
async fn unavailable_original_does_not_scan_unrelated_equivalent_users() {
    let catalog = input::catalog(&[], &[input::process("Equivalent User", "0.2", "user")]);
    let Some(case) = Case::with_catalog(
        "selection-no-equivalence-scan",
        input::settings("Missing Child", "0.2"),
        catalog,
    ) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Missing Child(selection.3mf)");
    assert_eq!(native["selected"]["external"], true);
    assert_eq!(native["selected"]["project_local"], true);
    assert_eq!(native["dirty"], false);
    let before = case.native["before"]["process"]["records"]
        .as_array()
        .unwrap();
    assert_eq!(
        native["records"].as_array().unwrap().len(),
        before.len() + 1
    );
    for record in before {
        assert_eq!(
            native["records"]
                .as_array()
                .unwrap()
                .iter()
                .find(|after| after["name"] == record["name"])
                .unwrap(),
            record
        );
    }
    case.write(
        "native-premise.json",
        &json!({"passed": true, "unrelated_equivalent_not_selected": true}),
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
    assert_eq!(process["outcome"], "project_local");
    assert_eq!(process["selected"]["name"], native["selected"]["name"]);
    assert_eq!(process["selected"]["external"], true);
    assert_eq!(process["dirty"], false);
}
