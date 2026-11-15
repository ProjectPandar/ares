//! Public catalog projection of the FFF defaults in OrcaSlicer 40ecc66f7.
//! Names and constructor shapes below are source-derived from Preset.hpp:573-579,
//! Preset.cpp:1766-1824 and PresetBundle.cpp:484-554, not observed GUI state.

use ares_core::{PresetCatalogHandle, ProfileKind};
use serde_json::{Value, json};

#[path = "preset_catalog_defaults/selection.rs"]
mod selection;

fn observe(name: &str, value: &Value) {
    let root = std::env::var_os("ARES_PRESET_DEFAULT_ARTIFACT_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("target/preset-catalog-defaults"));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(format!("{name}.json")),
        serde_json::to_vec_pretty(value).unwrap(),
    )
    .unwrap();
}

fn values(
    catalog: &PresetCatalogHandle,
    kind: ProfileKind,
    name: &str,
    keys: &[&str],
    artifact: &str,
) -> Value {
    for key in keys {
        assert!(
            ares_core::option_definition(key).is_some(),
            "source-defined option {key} must have a registered owner"
        );
    }
    let actual = catalog.preset_values(kind, &[name.to_owned()], keys);
    observe(
        artifact,
        &json!({
            "identity_source": "PresetBundle.cpp:484-554; PresetCollection constructor",
            "name": name, "keys": keys,
            "actual": actual.as_ref().map(|value| serde_json::to_value(value).unwrap()).map_err(ToString::to_string)
        }),
    );
    serde_json::to_value(actual.unwrap()).unwrap()
}

fn import(catalog: &mut PresetCatalogHandle, mut preset: Value) {
    preset["from"] = json!("user");
    preset["instantiation"] = json!("true");
    preset["version"] = json!("2.5.0.0");
    let id = match preset["type"].as_str().unwrap() {
        "machine" => "printer_settings_id",
        "process" => "print_settings_id",
        "filament" => "filament_settings_id",
        _ => unreachable!(),
    };
    preset[id] = preset["name"].clone();
    catalog
        .add_import(preset.to_string().as_bytes(), "preset.json")
        .unwrap();
}

fn machine() -> PresetCatalogHandle {
    let mut catalog = PresetCatalogHandle::new();
    import(
        &mut catalog,
        json!({
            "type": "machine", "name": "Single", "printer_technology": "FFF",
            "nozzle_diameter": ["0.4"], "single_extruder_multi_material": "0"
        }),
    );
    catalog
}

#[test]
fn source_defined_process_default_has_initialized_vectors_and_empty_identity() {
    let actual = values(
        &PresetCatalogHandle::new(),
        ProfileKind::Process,
        "Default Setting",
        &[
            "print_settings_id",
            "inherits",
            "compatible_printers_condition",
            "layer_height",
            "bridge_speed",
        ],
        "process-default",
    );
    assert_eq!(
        actual,
        json!([{
            "print_settings_id": "", "inherits": "", "compatible_printers_condition": "",
            "layer_height": "0.2", "bridge_speed": ["25"]
        }])
    );
}

#[test]
fn source_defined_filament_default_preserves_one_empty_id_and_nil_overrides() {
    let actual = values(
        &PresetCatalogHandle::default(),
        ProfileKind::Filament,
        "Default Filament",
        &[
            "filament_settings_id",
            "inherits",
            "filament_diameter",
            "filament_retraction_length",
            "filament_ironing_speed",
        ],
        "filament-default",
    );
    assert_eq!(
        actual,
        json!([{
            "filament_settings_id": [""], "inherits": "", "filament_diameter": ["1.75"],
            "filament_retraction_length": ["nil"], "filament_ironing_speed": ["nil"]
        }])
    );
}

#[test]
fn source_defined_printer_default_retains_definition_defaults_after_constructor_touches() {
    let actual = values(
        &PresetCatalogHandle::new(),
        ProfileKind::Machine,
        "Default Printer",
        &[
            "printer_settings_id",
            "printer_model",
            "printer_variant",
            "inherits",
            "thumbnails",
            "default_print_profile",
            "default_filament_profile",
            "nozzle_diameter",
        ],
        "printer-default",
    );
    assert_eq!(
        actual,
        json!([{
            "printer_settings_id": "", "printer_model": "", "printer_variant": "", "inherits": "",
            "thumbnails": "48x48/PNG,300x300/PNG", "default_print_profile": "",
            "default_filament_profile": [], "nozzle_diameter": ["0.4"]
        }])
    );
}

#[test]
fn defaults_do_not_manufacture_generic_pla_or_wrong_kind_names() {
    let catalog = PresetCatalogHandle::new();
    for (kind, name) in [
        (ProfileKind::Filament, "Generic PLA @System"),
        (ProfileKind::Filament, "Default Setting"),
        (ProfileKind::Process, "Default Printer"),
        (ProfileKind::Machine, "Default Filament"),
    ] {
        let error = catalog
            .preset_values(kind, &[name.to_owned()], &["inherits"])
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("preset '{name}' is not in the catalog"))
        );
    }
    observe(
        "unmanufactured-names",
        &json!({"generic_pla_alias": false, "cross_kind_lookup": false}),
    );
}

#[test]
fn default_lookup_keeps_unknown_option_diagnostics() {
    let error = PresetCatalogHandle::new()
        .preset_values(
            ProfileKind::Filament,
            &["Default Filament".to_owned()],
            &["ares_unknown_default_option"],
        )
        .unwrap_err();
    observe(
        "unknown-default-option",
        &json!({"error": error.to_string()}),
    );
    assert!(
        error
            .to_string()
            .contains("unknown option 'ares_unknown_default_option'")
    );
}

#[test]
fn ordinary_name_collision_is_resolved_before_builtin_default() {
    let mut catalog = PresetCatalogHandle::new();
    import(
        &mut catalog,
        json!({"type": "filament", "name": "Default Filament", "filament_diameter": ["2.85"]}),
    );
    assert_eq!(
        values(
            &catalog,
            ProfileKind::Filament,
            "Default Filament",
            &["filament_diameter"],
            "ordinary-before-default"
        ),
        json!([{"filament_diameter": ["2.85"]}])
    );
}

#[test]
fn adding_imports_preserves_builtin_lookup_without_installing_defaults() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "filament", "name": "Real PLA", "filament_type": ["PLA"]}),
    );
    for (kind, name) in [
        (ProfileKind::Process, "Default Setting"),
        (ProfileKind::Filament, "Default Filament"),
        (ProfileKind::Machine, "Default Printer"),
    ] {
        assert_eq!(
            values(
                &catalog,
                kind,
                name,
                &["inherits"],
                &format!("preserved-{name}")
            ),
            json!([{"inherits": ""}])
        );
    }
    let installed = catalog.installed_filaments(&[]).unwrap();
    observe(
        "installed-default-isolation",
        &json!({"installed": installed}),
    );
    assert!(!installed.iter().any(|name| name.starts_with("Default ")));
    let printers = serde_json::to_value(catalog.printers()).unwrap();
    assert_eq!(printers.as_array().unwrap().len(), 1);
    assert_eq!(printers[0]["name"], "Single");
    assert_eq!(printers[0]["vendor"], "user");
}

#[test]
fn one_shot_catalog_constructs_source_defined_defaults() {
    let actual = ares_core::filter_compatible_presets(
        "{\"vendors\":[],\"imports\":[]}",
        "Default Printer",
        Some("Default Setting"),
    );
    observe(
        "one-shot-defaults",
        &json!({"actual": actual.as_ref().map(|value| serde_json::to_value(value).unwrap()).map_err(ToString::to_string)}),
    );
    assert!(actual.is_ok());
}
