#[path = "project_preset_selection/input.rs"]
mod input;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;
#[path = "project_preset_selection/support.rs"]
mod support;

use serde_json::json;
use support::Case;

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
