use ares_core::ConfigOverrides;
use serde_json::json;

use super::{input, support::Case};

#[tokio::test]
async fn selection_preserves_loaded_model_and_applies_explicit_overrides_last() {
    let mut settings = input::settings("Loaded Process", "0.27");
    settings["filament_map_mode"] = json!("Manual");
    settings["filament_map"] = json!(["1"]);
    settings["different_settings_to_system"] = json!(["", "", ""]);
    let catalog = input::catalog(&[input::process("Loaded Process", "0.2", "system")], &[]);
    let Some(case) = Case::with_catalog("selection-application", settings, catalog) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Loaded Process");
    assert_eq!(native["dirty"], false);
    assert_eq!(
        native["edited"]["config"]["layer_height"]["serialized"],
        "0.2"
    );
    case.write("native-premise.json", &json!({"passed": true}));
    let control = case.placed(false).await.unwrap();
    assert_eq!(control.objects().len(), 1);
    assert_eq!(control.objects()[0].name(), "Synthetic cube");
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.27");
    case.write("cli-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    for key in [
        "filament_map",
        "filament_map_mode",
        "filament_ids",
        "filament_colour",
    ] {
        assert_eq!(view["settings"][key], cli[key], "{key}");
    }
    let selected = case.placed(true).await.unwrap();
    assert_eq!(selected.models(), control.models());
    assert_eq!(selected.objects(), control.objects());
    assert_eq!(selected.plates(), control.plates());
    let overrides = ConfigOverrides::from_json(br#"{"layer_height":"0.31"}"#).unwrap();
    let overridden = case.api_with_overrides(&overrides).await.unwrap();
    assert_eq!(overridden["settings"]["layer_height"], "0.31");
}

#[tokio::test]
async fn single_filament_always_selection_remains_admitted() {
    let settings = input::settings("Missing Process", "0.2");
    let Some(case) = Case::new("selection-single-boundary", settings.clone()) else {
        return;
    };
    assert_eq!(
        case.native["projects"][0]["result"]["process"]["dirty"],
        false
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "scope": "single-filament control"}),
    );
    case.boundary(&settings, false).await.unwrap();
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "filament_slots": 1}),
    );
    case.boundary(&settings, true).await.unwrap();
}

#[tokio::test]
async fn two_filament_always_selection_loads_each_external_filament() {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(["Missing Filament A", "Missing Filament B"]);
    settings["filament_ids"] = json!(["fixture-a", "fixture-b"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE"]);
    settings["filament_diameter"] = json!(["1.75", "1.75"]);
    settings["filament_flow_ratio"] = json!(["0.91", "0.92"]);
    settings["filament_extruder_variant"] =
        json!(["Direct Drive Standard", "Direct Drive Standard"]);
    settings["different_settings_to_system"] = json!(["", "", "", ""]);
    let Some(case) = Case::new("selection-two-filament-always", settings) else {
        return;
    };
    case.write("native-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("two-filament Always must load external filament presets");
    assert_eq!(
        view["settings"]["filament_flow_ratio"],
        json!(["0.91", "0.92"])
    );
    assert_eq!(
        view["settings"]["filament_settings_id"],
        json!([
            "Missing Filament A(selection.3mf)",
            "Missing Filament B(selection.3mf)"
        ])
    );
}

#[tokio::test]
async fn multifilament_selection_modes_keep_loaded_names_for_modified_slots() {
    let catalog = input::catalog(
        &[
            json!({
                "type": "filament", "name": "Loaded Filament A", "from": "system",
                "instantiation": "true", "version": "02.05.00.00",
                "filament_id": "fixture-a", "filament_flow_ratio": ["0.91"]
            }),
            json!({
                "type": "filament", "name": "Loaded Filament B", "from": "system",
                "instantiation": "true", "version": "02.05.00.00",
                "filament_id": "fixture-b", "filament_flow_ratio": ["0.93"]
            }),
        ],
        &[],
    );
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!([
        "Loaded Filament A",
        "Loaded Filament A",
        "Loaded Filament B"
    ]);
    settings["filament_ids"] = json!(["fixture-a", "fixture-a", "fixture-b"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE", "#EECCAA"]);
    settings["filament_diameter"] = json!(["1.75", "1.75", "1.75"]);
    settings["filament_flow_ratio"] = json!(["0.91", "0.92", "0.94"]);
    settings["filament_extruder_variant"] = json!([
        "Direct Drive Standard",
        "Direct Drive Standard",
        "Direct Drive Standard"
    ]);
    settings["different_settings_to_system"] = json!(["", "", "", "", ""]);
    let Some(case) = Case::with_catalog("selection-modes", settings, catalog) else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    assert_eq!(
        native["filament_presets"],
        json!([
            "Loaded Filament A",
            "Loaded Filament A",
            "Loaded Filament B"
        ])
    );
    let view = case
        .api()
        .await
        .expect("multi-filament selection modes must load external filament presets");
    assert_eq!(
        view["settings"]["filament_settings_id"],
        json!([
            "Loaded Filament A",
            "Loaded Filament A",
            "Loaded Filament B"
        ])
    );
    // full_fff_config takes each filament's values from its loaded preset
    // (PresetBundle.cpp:5658-5661, 5738-5757), not from the project's vector.
    assert_eq!(
        view["settings"]["filament_flow_ratio"],
        json!(["0.91", "0.91", "0.93"])
    );
}

#[tokio::test]
async fn non_first_modified_system_filament_gets_external_name() {
    let catalog = input::catalog(
        &[
            json!({
                "type": "filament", "name": "Loaded Filament A", "from": "system",
                "instantiation": "true", "version": "02.05.00.00",
                "filament_id": "fixture-a", "filament_flow_ratio": ["0.91"]
            }),
            json!({
                "type": "filament", "name": "Loaded Filament B", "from": "system",
                "instantiation": "true", "version": "02.05.00.00",
                "filament_id": "fixture-b", "filament_flow_ratio": ["0.93"],
                "filament_diameter": ["1.75"]
            }),
        ],
        &[],
    );
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(["Loaded Filament A", "Loaded Filament B"]);
    settings["filament_ids"] = json!(["fixture-a", "fixture-b"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE"]);
    settings["filament_flow_ratio"] = json!(["0.91", "0.94"]);
    settings["filament_diameter"] = json!(["1.75", "1.80"]);
    settings["filament_extruder_variant"] =
        json!(["Direct Drive Standard", "Direct Drive Standard"]);
    settings["different_settings_to_system"] = json!(["", "", "filament_flow_ratio", ""]);
    let Some(case) = Case::with_catalog("selection-existing-system", settings, catalog) else {
        return;
    };
    case.write("native-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("modified non-first system filament must load");
    assert_eq!(
        view["settings"]["filament_settings_id"],
        json!(["Loaded Filament A", "Loaded Filament B(selection.3mf)"])
    );
}

#[tokio::test]
async fn variant_list_without_self_index_is_completed_before_validation() {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(["Missing Filament A", "Missing Filament B"]);
    settings["filament_ids"] = json!(["fixture-a", "fixture-b"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE"]);
    settings["filament_diameter"] = json!(["1.75", "1.75"]);
    settings["filament_flow_ratio"] = json!(["0.91", "0.92"]);
    settings["filament_extruder_variant"] =
        json!(["Direct Drive Standard", "Direct Drive High Flow"]);
    settings["extruder_variant_list"] = json!(["Direct Drive Standard", "Direct Drive High Flow"]);
    let Some(case) = Case::new("selection-variant-missing-index", settings) else {
        return;
    };
    case.write("native-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("missing self-index must be completed before validation");
    assert_eq!(
        view["settings"]["filament_flow_ratio"],
        json!(["0.91", "0.92"])
    );
}

#[tokio::test]
async fn equal_variant_and_filament_counts_use_set_at_path() {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(["Missing Filament A", "Missing Filament B"]);
    settings["filament_ids"] = json!(["fixture-a", "fixture-b"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE"]);
    settings["filament_diameter"] = json!(["1.75", "1.75"]);
    settings["filament_flow_ratio"] = json!(["0.91", "0.92"]);
    settings["filament_extruder_variant"] =
        json!(["Direct Drive Standard", "Direct Drive High Flow"]);
    settings["filament_self_index"] = json!(["2", "1"]);
    settings["extruder_variant_list"] = json!(["Direct Drive Standard", "Direct Drive High Flow"]);
    let Some(case) = Case::new("selection-variant-equal-count", settings) else {
        return;
    };
    case.write("native-premise.json", &json!({"passed": true}));
    let view = case
        .api()
        .await
        .expect("equal variant and filament counts must select");
    assert_eq!(
        view["settings"]["filament_flow_ratio"],
        json!(["0.91", "0.92"])
    );
}

#[tokio::test]
async fn h2d_always_selection_slices_filament_variant_ranges() {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(["H2D Filament A", "H2D Filament B"]);
    settings["filament_ids"] = json!(["h2d-a", "h2d-b"]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE"]);
    settings["filament_diameter"] = json!(["1.75", "1.75"]);
    settings["filament_self_index"] = json!(["1", "1", "2", "2"]);
    settings["filament_extruder_variant"] = json!([
        "Direct Drive Standard",
        "Direct Drive High Flow",
        "Direct Drive Standard",
        "Direct Drive High Flow"
    ]);
    settings["filament_flow_ratio"] = json!(["0.91", "0.92", "0.93", "0.94"]);
    settings["extruder_variant_list"] = json!(["Direct Drive Standard", "Direct Drive High Flow"]);
    settings["nozzle_diameter"] = json!(["0.4", "0.4"]);
    settings["different_settings_to_system"] = json!(["", "", "", ""]);
    let Some(case) = Case::with_catalog("selection-h2d-always", settings, input::catalog(&[], &[]))
    else {
        return;
    };
    case.write(
        "native-premise.json",
        &json!({"passed": true, "scope": "H2D"}),
    );
    let view = case
        .api()
        .await
        .expect("H2D multi-filament Always must load external filament presets");
    assert_eq!(
        view["settings"]["filament_flow_ratio"],
        json!(["0.91", "0.92", "0.93", "0.94"])
    );
}
