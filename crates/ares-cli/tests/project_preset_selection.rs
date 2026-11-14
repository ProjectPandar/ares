#[path = "project_preset_selection/application.rs"]
mod application;
#[path = "project_preset_selection/differences.rs"]
mod differences;
#[path = "project_preset_selection/existing.rs"]
mod existing;
#[path = "project_preset_selection/filament_composition.rs"]
mod filament_composition;
#[path = "project_preset_selection/input.rs"]
mod input;
#[path = "project_preset_selection/loaded_filament_sync.rs"]
mod loaded_filament_sync;
#[path = "project_preset_selection/local.rs"]
mod local;
#[path = "project_preset_selection/lookup.rs"]
mod lookup;
#[path = "project_preset_selection/metadata.rs"]
mod metadata;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;
#[path = "project_preset_selection/points_groups.rs"]
mod points_groups;
#[path = "project_preset_selection/support.rs"]
mod support;
#[path = "project_preset_selection/system.rs"]
mod system;
#[path = "project_preset_selection/variants.rs"]
mod variants;

use serde_json::json;
use support::Case;

#[tokio::test]
async fn empty_difference_entries_refresh_system_only_for_always() {
    let catalog = json!({
        "vendors": [{
            "id": "Acme",
            "index": {
                "name": "Acme", "version": "02.05.00.00",
                "machine_model_list": [], "machine_list": [], "filament_list": [],
                "process_list": [{"name": "Loaded Process", "sub_path": "process/loaded.json"}]
            },
            "files": {"process/loaded.json": {
                "type": "process", "name": "Loaded Process", "from": "system",
                "instantiation": "true", "version": "02.05.00.00", "layer_height": "0.2"
            }}
        }],
        "imports": []
    });
    let Some(case) = Case::with_catalog(
        "selection-system-empty-differences",
        json!({
            "print_settings_id": "Loaded Process",
            "printer_settings_id": "Missing Printer",
            "filament_settings_id": ["Missing Filament"],
            "filament_ids": ["fixture-filament"],
            "filament_colour": ["#AABBCC"],
            "filament_diameter": ["1.75"],
            "layer_height": "0.27",
            "nozzle_diameter": ["0.6"],
            "printable_area": ["0x0", "200x0", "200x200", "0x200"],
            "different_settings_to_system": ["", "", ""]
        }),
        catalog,
    ) else {
        return;
    };
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], "Loaded Process");
    assert_eq!(native["selected"]["system"], true);
    assert_eq!(native["selected"]["external"], false);
    assert_eq!(native["selected"]["project_local"], false);
    assert_eq!(native["dirty"], false);
    assert_eq!(native["dirty_options"], json!([]));
    assert_eq!(native["selected"]["config"], native["edited"]["config"]);
    assert_eq!(
        native["edited"]["config"]["layer_height"]["serialized"],
        "0.2"
    );
    assert_eq!(
        native["records"],
        case.native["before"]["process"]["records"]
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "system_refresh": true}),
    );

    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.27");
    assert!(cli.get("settings").is_none());
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
    assert_eq!(process["outcome"], "existing_clean");
    for field in [
        "name",
        "system",
        "external",
        "project_local",
        "visible",
        "version",
    ] {
        assert_eq!(
            process["selected"][field], native["selected"][field],
            "{field}"
        );
    }
    assert_eq!(process["dirty"], native["dirty"]);
    assert_eq!(process["dirty_options"], native["dirty_options"]);
}

#[tokio::test]
async fn unavailable_loaded_presets_become_clean_project_local_selections() {
    let Some(case) = Case::new(
        "selection-unavailable",
        json!({
            "print_settings_id": "Missing Process",
            "printer_settings_id": "Missing Printer",
            "filament_settings_id": ["Missing Filament"],
            "filament_ids": ["fixture-filament"],
            "filament_colour": ["#AABBCC"],
            "filament_diameter": ["1.75"],
            "layer_height": "0.27",
            "nozzle_diameter": ["0.6"],
            "printable_area": ["0x0", "200x0", "200x200", "0x200"]
        }),
    ) else {
        return;
    };
    let observed = &case.native["projects"][0];
    assert_eq!(observed["materialized"].as_array().unwrap().len(), 3);
    let native = &observed["result"];
    for (kind, name) in [
        ("process", "Missing Process(selection.3mf)"),
        ("printer", "Missing Printer(selection.3mf)"),
        ("filament", "Missing Filament(selection.3mf)"),
    ] {
        let collection = &native[kind];
        assert_eq!(collection["selected"]["name"], name);
        assert_eq!(collection["selected"]["external"], true);
        assert_eq!(collection["selected"]["project_local"], true);
        assert_eq!(collection["selected"]["visible"], true);
        assert_eq!(collection["dirty"], false);
        assert_eq!(collection["dirty_options"], json!([]));
        assert_eq!(
            collection["selected"]["config"],
            collection["edited"]["config"]
        );
        let before = case.native["before"][kind]["records"].as_array().unwrap();
        let after = collection["records"].as_array().unwrap();
        assert!(before.iter().all(|record| record["default"] == true));
        assert_eq!(after.len(), before.len() + 1);
        for stored in before {
            let retained = after
                .iter()
                .find(|record| record["name"] == stored["name"])
                .unwrap();
            assert_eq!(retained["config"], stored["config"]);
        }
    }
    assert_eq!(
        native["process"]["edited"]["config"]["layer_height"]["serialized"],
        "0.27"
    );
    assert_eq!(
        native["printer"]["edited"]["config"]["nozzle_diameter"]["serialized"],
        "0.6"
    );
    case.write(
        "native-premise.json",
        &json!({"passed": true, "collections": 3}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.27");
    assert!(cli.get("preset_selection").is_none());
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
        assert_eq!(actual["outcome"], "project_local");
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
        assert_eq!(actual["dirty"], native[kind]["dirty"]);
        assert_eq!(actual["dirty_options"], native[kind]["dirty_options"]);
    }
    assert_eq!(view["settings"]["layer_height"], "0.27");
    assert_eq!(view["settings"]["nozzle_diameter"], json!(["0.6"]));
    assert_eq!(
        view["settings"]["print_settings_id"],
        native["process"]["selected"]["name"]
    );
    assert_eq!(
        view["settings"]["printer_settings_id"],
        native["printer"]["selected"]["name"]
    );
    assert_eq!(
        view["settings"]["filament_settings_id"],
        json!([native["filament"]["selected"]["name"]])
    );
}
