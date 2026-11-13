use std::{fs, path::PathBuf, process::Command};

use ares_core::{ConfigOverrides, Placement, ProfileSliceRequest, profile_settings};
use serde_json::{Value, json};

#[path = "project_preset_selection/input.rs"]
mod input;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

fn fixture(stale: bool) -> (Value, Value) {
    let mut project = input::settings("Unlisted Synthetic Process", "0.2");
    project["printer_settings_id"] = json!("Unlisted Synthetic Printer");
    project["nozzle_diameter"] = json!(["0.4"]);
    project["filament_settings_id"] = json!(["Synthetic Material A", "Synthetic Material B"]);
    project["filament_ids"] = json!(["ASYN01", "ASYN02"]);
    project["filament_colour"] = json!(["#AABBCC", "#DDEEFF"]);
    project["filament_diameter"] = json!(["1.75", "1.75"]);
    project["filament_type"] = json!(["PLA", "PLA"]);
    project["filament_is_support"] = json!(["0", "0"]);
    project["filament_flow_ratio"] = json!(["0.91", "0.92"]);
    project["filament_self_index"] = if stale {
        json!(["1", "1"])
    } else {
        json!(["1", "2"])
    };
    project["filament_extruder_variant"] =
        json!(["Direct Drive Standard", "Direct Drive Standard"]);
    project["inherits_group"] = json!(["", "", "", ""]);
    project["different_settings_to_system"] = json!([
        "layer_height",
        "filament_flow_ratio",
        "filament_flow_ratio",
        ""
    ]);
    let filaments = ["Synthetic Material A", "Synthetic Material B"]
        .into_iter()
        .enumerate()
        .map(|(slot, name)| json!({
            "type": "filament", "name": name, "from": "system", "instantiation": "true",
            "version": "02.05.00.00", "filament_id": (["ASYN01", "ASYN02"][slot]),
            "filament_extruder_variant": ["Direct Drive Standard"], "filament_type": ["PLA"],
            "filament_diameter": ["1.75"], "filament_flow_ratio": [if slot == 0 { "1.02" } else { "1.04" }]
        }))
        .chain(std::iter::once(input::process("Unused Synthetic Process", "0.2", "system")))
        .collect::<Vec<_>>();
    (project, input::catalog(&filaments, &[]))
}

async fn compare(stale: bool) {
    let Some(binary) = native_environment::binary("ARES_ORCA_FILAMENT_REFRESH_BIN") else {
        return;
    };
    let root = PathBuf::from(native_environment::value("ARES_PARITY_ARTIFACT_ROOT").unwrap());
    assert!(root.is_absolute());
    fs::create_dir_all(&root).unwrap();
    let root = tempfile::Builder::new()
        .prefix(if stale {
            "filament-stale-index-"
        } else {
            "filament-valid-index-"
        })
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("artifacts: {}", root.display());
    let (project, catalog) = fixture(stale);
    let archive = input::project(&project);
    fs::write(root.join("input.3mf"), &archive).unwrap();
    fs::write(root.join("catalog.json"), catalog.to_string()).unwrap();
    fs::write(root.join("project-settings.json"), project.to_string()).unwrap();
    let resources = root.join("resources");
    for location in [resources.join("profiles"), root.join("data/system")] {
        fs::create_dir_all(&location).unwrap();
        for vendor in catalog["vendors"].as_array().unwrap() {
            let id = vendor["id"].as_str().unwrap();
            fs::write(
                location.join(format!("{id}.json")),
                vendor["index"].to_string(),
            )
            .unwrap();
            for (filename, document) in vendor["files"].as_object().unwrap() {
                let path = location.join(id).join(filename);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, document.to_string()).unwrap();
            }
        }
    }
    let arguments = [
        "--debug".to_owned(),
        "3".to_owned(),
        "--datadir".to_owned(),
        root.join("data").display().to_string(),
        "--export-settings".to_owned(),
        root.join("native.json").display().to_string(),
        root.join("input.3mf").display().to_string(),
    ];
    fs::write(
        root.join("native-command.json"),
        json!({"program": binary, "args": arguments,
        "cwd": root, "resources": resources})
        .to_string(),
    )
    .unwrap();
    let output = Command::new(binary)
        .args(&arguments)
        .current_dir(&root)
        .env("ARES_ORCA_RESOURCES_DIR", &resources)
        .env_remove("ARES_PRESET_SELECT_REQUEST")
        .output()
        .unwrap();
    fs::write(root.join("native.stdout"), &output.stdout).unwrap();
    fs::write(root.join("native.stderr"), &output.stderr).unwrap();
    fs::write(
        root.join("native-exit.json"),
        json!({"code": output.status.code()}).to_string(),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "Native CLI failed: {}",
        root.display()
    );
    assert_eq!(
        fs::read_to_string(root.join("native-affinity.txt"))
            .unwrap()
            .split(':')
            .nth(1)
            .unwrap()
            .trim(),
        "60"
    );
    let native: Value =
        serde_json::from_slice(&fs::read(root.join("native.json")).unwrap()).unwrap();
    assert_eq!(native["filament_self_index"], json!(["1", "2"]));
    assert_eq!(native["filament_flow_ratio"], json!(["0.91", "0.92"]));
    assert_eq!(native["filament_colour"], project["filament_colour"]);
    assert_eq!(
        native["filament_extruder_variant"],
        project["filament_extruder_variant"]
    );
    let log = fs::read_to_string(root.join("00000.log")).unwrap();
    assert!(!log.contains("Could not find vendor for preset Synthetic Material"));
    assert_eq!(log.contains("regenerating"), stale);
    fs::write(
        root.join("native-premise.json"),
        json!({"passed": true, "stale": stale}).to_string(),
    )
    .unwrap();
    let catalog = catalog.to_string();
    let overrides = ConfigOverrides::default();
    let result = profile_settings(
        &archive,
        ProfileSliceRequest {
            filename: "input.3mf",
            catalog_json: &catalog,
            selection_json: "{\"from_project\":true}",
            plate: None,
            overrides: &overrides,
            placement: Placement::None,
        },
    )
    .await
    .map(|value| serde_json::to_value(value).unwrap());
    fs::write(
        root.join("api-result.json"),
        match &result {
            Ok(value) => json!({"ok": true, "result": value}),
            Err(error) => json!({"ok": false, "error": error.to_string()}),
        }
        .to_string(),
    )
    .unwrap();
    let actual = result.unwrap();
    for key in [
        "filament_flow_ratio",
        "filament_self_index",
        "filament_colour",
        "filament_settings_id",
        "filament_extruder_variant",
    ] {
        assert_eq!(actual[key], native[key], "{key}: {}", root.display());
    }
}

#[tokio::test]
async fn valid_two_slot_indices_preserve_protected_values() {
    compare(false).await;
}

#[tokio::test]
async fn stale_two_slot_indices_regenerate_before_restoring_protected_values() {
    compare(true).await;
}
