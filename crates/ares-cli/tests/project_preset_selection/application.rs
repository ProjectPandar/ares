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
async fn multiple_filament_admission_is_rejected_without_changing_cli_admission() {
    let settings = input::settings("Missing Process", "0.2");
    let Some(case) = Case::new("selection-multiple-boundary", settings.clone()) else {
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
    let mut multiple = settings;
    for key in [
        "filament_settings_id",
        "filament_ids",
        "filament_colour",
        "filament_diameter",
    ] {
        let first = multiple[key][0].clone();
        multiple[key] = json!([first, first]);
    }
    case.boundary(&multiple, false).await.unwrap();
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "filament_slots": 2}),
    );
    let error = case
        .boundary(&multiple, true)
        .await
        .expect_err("multi-filament Always is outside this slice");
    assert!(error.to_string().contains("single filament"), "{error}");
}
