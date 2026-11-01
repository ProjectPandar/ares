//! Stateful PresetCatalog handle:
//! `PresetCollection::load_presets` (`Preset.cpp:1685-1691`) and
//! `PresetBundle::import_json_presets` (`PresetBundle.cpp:1521-1525`) skip a
//! preset whose non-empty `inherits` cannot be resolved; the handle mirrors
//! that by keeping such presets invisible to filtering while reporting them
//! as structured missing parents.

use ares_core::{PresetCatalogHandle, filter_compatible_presets};
use serde_json::{Value, json};

fn vendor(name: &str, lists: Value, files: Value) -> Value {
    let mut index = json!({"name": name});
    for key in ["machine_list", "process_list", "filament_list"] {
        if let Some(entries) = lists.get(key) {
            index[key] = entries.clone();
        }
    }
    json!({"id": name, "index": index, "files": files})
}

fn entry(sub_path: &str) -> Value {
    json!({"sub_path": sub_path})
}

/// A machine vendor plus a filament child inheriting the base library.
fn acme_vendor() -> Value {
    vendor(
        "Acme",
        json!({
            "machine_list": [entry("machine_base.json"), entry("machine_m1.json")],
            "process_list": [entry("process_base.json"), entry("process_std.json")],
            "filament_list": [entry("filament_pla.json")]
        }),
        json!({
            "machine_base.json": {
                "type": "machine", "name": "fdm_acme_common", "instantiation": "false",
                "printer_model": "ACME", "printer_notes": "ACME"
            },
            "machine_m1.json": {
                "type": "machine", "name": "AcmeM1", "inherits": "fdm_acme_common",
                "from": "system", "instantiation": "true",
                "printer_model": "ACME", "nozzle_diameter": [0.4]
            },
            "process_base.json": {
                "type": "process", "name": "fdm_process_common", "instantiation": "false"
            },
            "process_std.json": {
                "type": "process", "name": "0.20mm Standard @AcmeM1",
                "inherits": "fdm_process_common", "from": "system",
                "instantiation": "true", "layer_height": "0.2"
            },
            "filament_pla.json": {
                "type": "filament", "name": "Acme PLA", "inherits": "fdm_filament_common",
                "from": "system", "instantiation": "true", "filament_diameter": [1.75]
            }
        }),
    )
}

fn library_vendor() -> Value {
    vendor(
        "OrcaFilamentLibrary",
        json!({"filament_list": [entry("lib_base.json")]}),
        json!({
            "lib_base.json": {
                "type": "filament", "name": "fdm_filament_common",
                "instantiation": "false"
            }
        }),
    )
}

fn to_value(result: ares_core::FilteredPresets) -> Value {
    serde_json::to_value(result).unwrap()
}

#[test]
fn handle_matches_one_shot_catalog_filtering() {
    let mut handle = PresetCatalogHandle::new();
    handle
        .add_vendor(library_vendor().to_string().as_bytes())
        .unwrap();
    handle
        .add_vendor(acme_vendor().to_string().as_bytes())
        .unwrap();
    let via_handle = to_value(
        handle
            .filter_compatible("AcmeM1", Some("0.20mm Standard @AcmeM1"))
            .unwrap(),
    );

    let catalog = json!({
        "vendors": [library_vendor(), acme_vendor()],
        "imports": [],
    });
    let via_one_shot = to_value(
        filter_compatible_presets(
            &catalog.to_string(),
            "AcmeM1",
            Some("0.20mm Standard @AcmeM1"),
        )
        .unwrap(),
    );

    assert_eq!(via_handle, via_one_shot);
    assert!(
        via_handle["missing_parents"].as_array().unwrap().is_empty(),
        "{via_handle}"
    );
    assert!(
        via_handle["filaments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "Acme PLA"),
        "{via_handle}"
    );
}

#[test]
fn handle_reports_missing_parents_and_hides_unresolvable_presets() {
    // Preset.cpp:1685-1691: the filament child inherits fdm_filament_common,
    // which the missing library vendor would supply; the child stays
    // invisible to filtering until the parent arrives.
    let mut handle = PresetCatalogHandle::new();
    handle
        .add_vendor(acme_vendor().to_string().as_bytes())
        .unwrap();
    let filtered = to_value(handle.filter_compatible("AcmeM1", None).unwrap());
    assert!(
        !filtered["filaments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "Acme PLA"),
        "{filtered}"
    );
    assert_eq!(
        filtered["missing_parents"],
        json!([{ "kind": "filament", "name": "Acme PLA", "parent": "fdm_filament_common" }]),
        "{filtered}"
    );

    // PresetBundle.cpp:1521-1525 skips at load; once the parent is added the
    // child re-enters the collection and the report clears.
    handle
        .add_vendor(library_vendor().to_string().as_bytes())
        .unwrap();
    let resolved = to_value(handle.filter_compatible("AcmeM1", None).unwrap());
    assert!(
        resolved["missing_parents"].as_array().unwrap().is_empty(),
        "{resolved}"
    );
    assert!(
        resolved["filaments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "Acme PLA"),
        "{resolved}"
    );
}

#[test]
fn one_shot_filtering_reports_missing_parents_instead_of_failing() {
    let catalog = json!({
        "vendors": [acme_vendor()],
        "imports": [],
    });
    let filtered =
        to_value(filter_compatible_presets(&catalog.to_string(), "AcmeM1", None).unwrap());
    assert_eq!(
        filtered["missing_parents"],
        json!([{ "kind": "filament", "name": "Acme PLA", "parent": "fdm_filament_common" }]),
        "{filtered}"
    );
}

#[test]
fn handle_imports_bundles_into_the_user_scope() {
    let mut handle = PresetCatalogHandle::new();
    handle
        .add_vendor(library_vendor().to_string().as_bytes())
        .unwrap();
    handle
        .add_vendor(acme_vendor().to_string().as_bytes())
        .unwrap();

    let bundle = json!({
        "type": "filament", "name": "User PETG", "inherits": "fdm_filament_common",
        "from": "User", "instantiation": "true", "version": "1.9.0.0",
        "filament_settings_id": "GFPU1", "filament_diameter": [1.75]
    });
    handle
        .add_import(bundle.to_string().as_bytes(), "user_petg.json")
        .unwrap();

    let filtered = to_value(handle.filter_compatible("AcmeM1", None).unwrap());
    assert!(
        filtered["filaments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "User PETG"),
        "{filtered}"
    );
}
