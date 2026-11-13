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
    assert_eq!(
        actual["edited"]["config"]["compatible_printers"],
        json!(super::support::native_strings(
            &native["edited"]["config"]["compatible_printers"]
        ))
    );
    for key in ["compatible_printers_condition", "print_settings_id"] {
        assert_eq!(native["edited"]["config"][key]["type"], 3);
        assert_eq!(
            actual["edited"]["config"][key],
            native["edited"]["config"][key]["serialized"]
        );
    }
}

#[tokio::test]
async fn version_three_component_patch_uses_native_bbs_projection() {
    version_case("2.5.123", "2.5.1.23").await;
}

#[tokio::test]
async fn version_suffixes_are_retained_for_project_local_presets() {
    version_case("2.5.123-beta.1+build.7", "2.5.1.23-beta.1+build.7").await;
}

#[tokio::test]
async fn version_two_component_input_has_source_zero_patch() {
    version_case("2.5", "2.5.0.0").await;
}

async fn version_case(version: &str, expected: &str) {
    let Some(case) = Case::with_version(
        &format!("selection-version-{version}"),
        input::settings("Version Process", "0.2"),
        input::catalog(&[], &[]),
        version,
    ) else {
        return;
    };
    for kind in ["process", "printer", "filament"] {
        assert_eq!(
            case.native["projects"][0]["result"][kind]["selected"]["version"],
            expected
        );
        assert_eq!(
            case.native["projects"][0]["result"][kind]["selected"]["project_local"],
            true
        );
    }
    case.write(
        "native-premise.json",
        &json!({"passed": true, "version": expected}),
    );
    case.cli().await.unwrap();
    case.write("cli-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    for kind in ["process", "printer", "filament"] {
        assert_eq!(
            view["preset_selection"][kind]["selected"]["version"],
            expected
        );
        assert_eq!(
            view["preset_selection"][kind]["edited"]["version"],
            expected
        );
    }
}
