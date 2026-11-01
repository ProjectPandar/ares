//! `PresetCatalogHandle::printers()` and the processed profile-entry variant:
//! selectable printer listing (`PresetComboBoxes.cpp:1170-1186` visibility,
//! `Preset.cpp:1663` load-time instantiation) and
//! `slice_with_profiles_processed` sharing the `record_moves` path
//! (`project_slice.rs slice_source`).

use serde_json::{Value, json};

fn vendor(name: &str, lists: Value, files: Value) -> Value {
    let mut index = json!({"name": name});
    for key in ["machine_list", "filament_list"] {
        if let Some(entries) = lists.get(key) {
            index[key] = entries.clone();
        }
    }
    json!({"id": name, "index": index, "files": files})
}

fn entry(sub_path: &str) -> Value {
    json!({"sub_path": sub_path})
}

fn acme_vendor() -> Value {
    vendor(
        "Acme",
        json!({
            "machine_list": [
                entry("machine_template.json"),
                entry("machine_base.json"),
                entry("machine_a1.json"),
                entry("machine_broken.json")
            ],
            "filament_list": [entry("filament_base.json"), entry("filament_pla.json")]
        }),
        json!({
            "machine_template.json": {
                "type": "machine", "name": "fdm_acme_common",
                "instantiation": "false", "printer_model": "ACME"
            },
            "machine_base.json": {
                "type": "machine", "name": "fdm_acme_a1_base", "instantiation": "false"
            },
            "machine_a1.json": {
                "type": "machine", "name": "Acme A1 0.4 nozzle",
                "inherits": "fdm_acme_a1_base", "from": "system",
                "instantiation": "true", "printer_model": "A1",
                "printer_variant": "0.4", "nozzle_diameter": [0.4]
            },
            "machine_broken.json": {
                "type": "machine", "name": "Acme Orphan", "inherits": "fdm_missing_base",
                "instantiation": "true", "printer_model": "A1", "printer_variant": "0.6"
            },
            "filament_base.json": {
                "type": "filament", "name": "fdm_filament_common", "instantiation": "false"
            },
            "filament_pla.json": {
                "type": "filament", "name": "Acme PLA", "inherits": "fdm_filament_common",
                "instantiation": "true", "filament_diameter": [1.75]
            }
        }),
    )
}

fn bbl_vendor() -> Value {
    vendor(
        "BBL",
        json!({"machine_list": [entry("machine_x1c.json")]}),
        json!({
            "machine_x1c.json": {
                "type": "machine", "name": "Bambu Lab X1 Carbon 0.4 nozzle",
                "from": "system", "instantiation": "true", "printer_model": "X1C",
                "printer_variant": "0.4", "nozzle_diameter": [0.4]
            }
        }),
    )
}

#[test]
fn printers_lists_selectable_machines_grouped_by_vendor() {
    // PresetComboBoxes.cpp:1170-1186 iterates the collection in load order
    // and skips invisible presets; load-time visibility is
    // `instantiation != "false"` (Preset.cpp:1663), and printers with
    // unresolvable parents stay out of the collection (Preset.cpp:1685-1691).
    let mut handle = ares_core::PresetCatalogHandle::new();
    handle
        .add_vendor(acme_vendor().to_string().as_bytes())
        .unwrap();
    handle
        .add_vendor(bbl_vendor().to_string().as_bytes())
        .unwrap();
    let import = json!({
        "type": "machine", "name": "My A1", "inherits": "Acme A1 0.4 nozzle",
        "from": "User", "instantiation": "true", "version": "1.9.0.0",
        "printer_model": "A1", "printer_variant": "0.8"
    });
    handle
        .add_import(import.to_string().as_bytes(), "my_a1.json")
        .unwrap();

    let printers: Vec<Value> = handle
        .printers()
        .iter()
        .map(|printer| {
            json!({
                "vendor": printer.vendor,
                "name": printer.name,
                "printer_model": printer.printer_model,
                "printer_variant": printer.printer_variant,
            })
        })
        .collect();

    assert_eq!(
        printers,
        vec![
            json!({
                "vendor": "Acme", "name": "Acme A1 0.4 nozzle",
                "printer_model": "A1", "printer_variant": "0.4"
            }),
            json!({
                "vendor": "BBL", "name": "Bambu Lab X1 Carbon 0.4 nozzle",
                "printer_model": "X1C", "printer_variant": "0.4"
            }),
            json!({
                "vendor": "user", "name": "My A1",
                "printer_model": "A1", "printer_variant": "0.8"
            }),
        ],
        "templates, missing-parent printers, and load order"
    );
}

#[test]
fn printers_do_not_resolve_vendor_parents_from_user_imports() {
    // Vendor-scope inheritance resolves within the vendor bundle (and the
    // filament base library for filaments, PresetBundle.cpp:4885-4918); a
    // user import cannot satisfy a system machine's parent, so the orphan
    // stays out of the listing.
    let mut handle = ares_core::PresetCatalogHandle::new();
    handle
        .add_vendor(acme_vendor().to_string().as_bytes())
        .unwrap();
    let parent = json!({
        "type": "machine", "name": "fdm_missing_base", "instantiation": "false",
        "version": "1.9.0.0"
    });
    handle
        .add_import(parent.to_string().as_bytes(), "missing_base.json")
        .unwrap();
    let names = handle
        .printers()
        .iter()
        .map(|printer| printer.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["Acme A1 0.4 nozzle".to_owned()]);
}
