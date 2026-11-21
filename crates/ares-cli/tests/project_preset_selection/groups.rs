use serde_json::{Value, json};

use super::{
    input,
    support::{Case, wire},
};

/// The groups `PresetBundle::full_fff_config` collects from the selected presets and stores only
/// when some entry is non-empty (PresetBundle.cpp:5823-5841).
const GROUPS: [&str; 5] = [
    "compatible_machine_expression_group",
    "compatible_process_expression_group",
    "inherits_group",
    "different_settings_to_system",
    "print_compatible_printers",
];

/// Each group of the plater's config, as the project view must hold it: absent where
/// full_fff_config stores none.
async fn assert_plater_groups(case: &Case) {
    let native = &case.native["projects"][0]["result"];
    let plater: Value = serde_json::from_str(native["full_fff_config"].as_str().unwrap()).unwrap();
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let groups =
        |config: &Value| GROUPS.map(|key| (key, config.get(key).map(|value| wire(key, value))));
    case.write("plater-groups.json", &json!(groups(&plater)));
    case.write("api-groups.json", &json!(groups(&view["settings"])));
    assert_eq!(groups(&view["settings"]), groups(&plater));
}

// full_fff_config starts from the defaults and the selected presets, so the all-empty groups a
// 3MF stores do not reach the plater's config (PresetBundle.cpp:5536-5542, 5823-5841).
#[tokio::test]
async fn stored_empty_groups_leave_the_plater_config() {
    let catalog = input::catalog(&[input::process("Grouped Process", "0.2", "system")], &[]);
    let mut settings = input::settings("Grouped Process", "0.2");
    settings["inherits_group"] = json!(["", "", ""]);
    settings["different_settings_to_system"] = json!(["", "", ""]);
    settings["compatible_machine_expression_group"] = json!(["", ""]);
    let Some(case) = Case::with_catalog("selection-empty-groups", settings, catalog) else {
        return;
    };
    assert_plater_groups(&case).await;
}

// The edited system process records its dirty option in the process column (5586-5595), and the
// project-local presets record what they inherit (5583, 5642, 5778).
#[tokio::test]
async fn an_edited_system_process_records_its_difference_column() {
    let catalog = input::catalog(&[input::process("Grouped Process", "0.2", "system")], &[]);
    let mut settings = input::settings("Grouped Process", "0.27");
    settings["different_settings_to_system"] = json!(["layer_height", "", ""]);
    let Some(case) = Case::with_catalog("selection-process-difference-group", settings, catalog)
    else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    let plater: Value = serde_json::from_str(native["full_fff_config"].as_str().unwrap()).unwrap();
    // Native premise: the process column carries the edited option.
    assert_eq!(
        plater["different_settings_to_system"][0],
        json!("layer_height")
    );
    assert_plater_groups(&case).await;
}
