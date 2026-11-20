use serde_json::{Value, json};

use super::{input, support::Case};

// The plater runs sync_loaded_project_filament_config (Plater.cpp:8905-8959) on the loaded
// project config (Plater.cpp:9755): filament_multi_colour takes the filament_colour count, and
// an entry whose first colour is not that filament's colour becomes the colour (8927-8937).
async fn plater_multi_colour(label: &str, settings: Value, expected: Value) {
    let Some(case) = Case::new(label, settings) else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    let plater: Value = serde_json::from_str(native["full_fff_config"].as_str().unwrap()).unwrap();
    // Native premise: the plater's project config holds the synced value.
    assert_eq!(plater["filament_multi_colour"], expected);
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    case.write(
        "api-multi-colour.json",
        &json!(view["settings"]["filament_multi_colour"]),
    );
    assert_eq!(view["settings"]["filament_multi_colour"], expected);
}

#[tokio::test]
async fn loaded_project_without_multi_colour_takes_the_filament_colours() {
    plater_multi_colour(
        "selection-multi-colour-absent",
        input::settings("Missing Process", "0.2"),
        json!(["#AABBCC"]),
    )
    .await;
}

#[tokio::test]
async fn loaded_multi_colour_keeps_matching_entries_and_resyncs_the_rest() {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(["Missing A", "Missing B", "Missing C"]);
    settings["filament_ids"] = json!(["fixture-a", "fixture-b", "fixture-c"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE", "#EECCAA"]);
    settings["filament_multi_colour"] = json!(["#AABBCC #FFFFFF", "#000000"]);
    settings["filament_diameter"] = json!(["1.75", "1.75", "1.75"]);
    settings["filament_extruder_variant"] = json!(vec!["Direct Drive Standard"; 3]);
    settings["different_settings_to_system"] = json!(vec![""; 5]);
    plater_multi_colour(
        "selection-multi-colour-resync",
        settings,
        json!(["#AABBCC #FFFFFF", "#CCDDEE", "#EECCAA"]),
    )
    .await;
}
