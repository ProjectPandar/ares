//! Bounded default-config comparison through the existing settings export.
//! Built-in names are source-derived (PresetBundle.cpp:484-554 / Preset.hpp:573-579).
//! The project settings export is not a Native default-entity or GUI-state export.

use ares_core::{PresetCatalogHandle, ProfileKind};
use serde_json::json;

#[test]
fn source_defined_builtin_configs_match_bounded_native_settings_and_definition_defaults() {
    let Some((runner, native)) = super::native_defaults() else {
        return;
    };
    let catalog = PresetCatalogHandle::new();
    let mut rows = Vec::new();
    for (kind, name, key, token) in [
        (
            ProfileKind::Process,
            "Default Setting",
            "bridge_speed",
            "25",
        ),
        (
            ProfileKind::Filament,
            "Default Filament",
            "filament_diameter",
            "1.75",
        ),
        (
            ProfileKind::Machine,
            "Default Printer",
            "nozzle_diameter",
            "0.4",
        ),
    ] {
        let definition = super::exported_definition(&runner, key);
        let actual = catalog.preset_values(kind, &[name.to_owned()], &[key]);
        rows.push(json!({
            "name": name, "key": key, "expected_token": token,
            "native_project_value": native[key],
            "definition_type": definition.kind,
            "definition_display_default": definition.default_value,
            "actual": actual.as_ref().map(|value| serde_json::to_value(value).unwrap()).map_err(ToString::to_string),
        }));
    }
    let reference_valid = rows.iter().all(|row| {
        row["native_project_value"] == json!([row["expected_token"]])
            && row["definition_display_default"] == row["expected_token"]
    });
    std::fs::write(runner.work.join("public-catalog-builtin-defaults.json"),
        serde_json::to_vec_pretty(&json!({
            "identity_source": "PresetBundle.cpp:484-554; Preset.hpp:573-579",
            "config_reference": "existing original-project --export-settings plus public PrintConfigDef",
            "reference_valid": reference_valid,
            "native_default_entity_exported": false,
            "native_gui_state_observed": false,
            "full_initialized_config_parity": false,
            "rows": rows,
        })).unwrap()).unwrap();
    assert!(
        reference_valid,
        "Native project/definition reference premise changed: {:?}",
        runner.work
    );
    for row in rows {
        let key = row["key"].as_str().unwrap();
        assert_eq!(
            row["actual"]["Ok"],
            json!([{key: row["native_project_value"]}]),
            "public catalog initialized config for {}: {:?}",
            row["name"],
            runner.work
        );
    }
}
