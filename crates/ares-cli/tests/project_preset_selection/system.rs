use serde_json::json;

use super::{input, support::Case};

#[tokio::test]
async fn all_three_loaded_system_collections_select_clean_references() {
    let mut catalog = input::catalog(
        &[
            input::process("Loaded Process", "0.2", "system"),
            json!({
                "type": "machine", "name": "Loaded Printer", "from": "system",
                "instantiation": "true", "version": "02.05.00.00",
                "printer_technology": "FFF", "printer_model": "Acme M", "printer_variant": "0.4",
                "nozzle_diameter": ["0.4"], "enable_pre_heating": "1"
            }),
            json!({
                "type": "filament", "name": "Loaded Filament", "from": "system",
                "instantiation": "true", "version": "02.05.00.00", "filament_id": "fixture-system-filament",
                "filament_type": ["PLA"], "filament_diameter": ["1.75"]
            }),
        ],
        &[],
    );
    catalog["vendors"][0]["index"]["machine_model_list"] =
        json!([{"name": "Acme M", "sub_path": "machine/model.json"}]);
    catalog["vendors"][0]["files"]["machine/model.json"] = json!({
        "type": "machine_model", "name": "Acme M", "model_id": "Acme M",
        "nozzle_diameter": "0.4", "machine_tech": "FFF", "family": "Acme",
        "url": "", "bed_model": "", "bed_texture": "", "default_materials": ""
    });
    let mut settings = input::settings("Loaded Process", "0.27");
    settings["printer_settings_id"] = json!("Loaded Printer");
    settings["filament_settings_id"] = json!(["Loaded Filament"]);
    settings["printer_model"] = json!("Acme M");
    settings["printer_variant"] = json!("0.4");
    settings["different_settings_to_system"] = json!(["", "", ""]);
    assert!(settings.get("enable_pre_heating").is_none());
    let Some(case) = Case::with_catalog("selection-all-system", settings, catalog) else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    for (kind, name) in [
        ("process", "Loaded Process"),
        ("printer", "Loaded Printer"),
        ("filament", "Loaded Filament"),
    ] {
        let state = &native[kind];
        assert_eq!(state["selected"]["name"], name);
        assert_eq!(state["selected"]["system"], true);
        assert_eq!(state["selected"]["external"], false);
        assert_eq!(state["dirty"], false);
        assert_eq!(state["edited"]["config"], state["selected"]["config"]);
        let mut expected_records = case.native["before"][kind]["records"].clone();
        for record in expected_records
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .filter(|record| record["default"] == true)
        {
            record["visible"] = json!(false);
        }
        for record in expected_records
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .filter(|record| record["name"] == name)
        {
            record["visible"] = json!(true);
        }
        assert_eq!(state["records"], expected_records);
    }
    assert_eq!(
        native["printer"]["edited"]["config"]["nozzle_diameter"]["serialized"],
        "0.4"
    );
    assert_eq!(
        native["printer"]["edited"]["config"]["enable_pre_heating"]["serialized"],
        "1"
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "system_collections": 3, "missing_project_key": "enable_pre_heating", "system_value": "1"}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.27");
    assert_eq!(cli["nozzle_diameter"], json!(["0.6"]));
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    for kind in ["process", "printer", "filament"] {
        let actual = &view["preset_selection"][kind];
        assert_eq!(actual["outcome"], "existing_clean");
        for field in [
            "name",
            "system",
            "external",
            "project_local",
            "visible",
            "filament_id",
            "version",
        ] {
            assert_eq!(
                actual["selected"][field], native[kind]["selected"][field],
                "{kind}.{field}"
            );
        }
        assert_eq!(actual["dirty"], false);
    }
    assert_eq!(view["settings"]["layer_height"], "0.2");
    assert_eq!(view["settings"]["nozzle_diameter"], json!(["0.4"]));
    assert_eq!(view["settings"]["filament_colour"], json!(["#AABBCC"]));
    assert_eq!(view["settings"]["enable_pre_heating"], "1");
}
