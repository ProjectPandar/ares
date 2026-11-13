use serde_json::json;

use super::{input, support::Case};

#[tokio::test]
async fn ignored_compatibility_metadata_does_not_modify_an_equal_selected_user() {
    let mut settings = input::settings("User Process", "0.2");
    settings["print_compatible_printers"] = json!(["Unrelated Printer"]);
    settings["compatible_machine_expression_group"] =
        json!(["printer_model == \"synthetic\"", "", ""]);
    let catalog = input::catalog(&[], &[input::process("User Process", "0.2", "user")]);
    let Some(case) = Case::with_catalog("selection-ignored-compatibility", settings, catalog)
    else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "User Process");
    assert_eq!(native["dirty"], false);
    assert_eq!(native["dirty_options"], json!([]));
    assert_eq!(
        native["edited"]["config"]["compatible_printers"]["serialized"],
        ""
    );
    assert_eq!(
        native["edited"]["config"]["compatible_printers_condition"]["serialized"],
        ""
    );
    case.write("native-premise.json", &json!({"passed": true}));
    case.cli().await.unwrap();
    case.write("cli-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let actual = &view["preset_selection"]["process"];
    assert_eq!(actual["outcome"], "existing_clean");
    assert_eq!(actual["dirty"], false);
    for key in [
        "compatible_printers",
        "compatible_printers_condition",
        "print_settings_id",
    ] {
        assert_eq!(
            actual["edited"]["config"][key],
            native["edited"]["config"][key]
        );
    }
}
