//! Bundle import and compatibility filtering:
//! `PresetBundle.cpp:1304-1549` (`import_presets`/`import_json_presets`) and
//! `Preset.cpp:771-851` (`is_compatible_with_printer`/`is_compatible_with_print`,
//! library exclusions `:3705-3736`, extras `:843-850`).

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

fn catalog_json() -> String {
    json!({
        "vendors": [
            vendor(
                "OrcaFilamentLibrary",
                json!({"filament_list": [entry("lib_base.json"), entry("lib_pla.json")]}),
                json!({
                    "lib_base.json": {
                        "type": "filament",
                        "name": "fdm_filament_pla",
                        "instantiation": "false",
                        "filament_id": "GFL00"
                    },
                    "lib_pla.json": {
                        "type": "filament",
                        "name": "Generic PLA @System",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "GFL01",
                        "compatible_printers": [],
                        "filament_diameter": [1.75]
                    }
                })
            ),
            vendor(
                "Acme",
                json!({
                    "machine_list": [
                        entry("machine_base.json"),
                        entry("machine_m1.json"),
                        entry("machine_m2.json")
                    ],
                    "process_list": [
                        entry("process_base.json"),
                        entry("process_std.json"),
                        entry("process_small.json"),
                        entry("process_broken.json"),
                        entry("process_template.json")
                    ],
                    "filament_list": [
                        entry("filament_matched.json"),
                        entry("filament_other.json"),
                        entry("filament_cond.json"),
                        entry("filament_cond_print.json"),
                        entry("filament_prints.json"),
                        entry("filament_generic_clone.json")
                    ]
                }),
                json!({
                    "machine_base.json": {
                        "type": "machine",
                        "name": "fdm_acme_common",
                        "instantiation": "false",
                        "printer_model": "ACME",
                        "printer_notes": "ACME"
                    },
                    "machine_m1.json": {
                        "type": "machine",
                        "name": "Acme M1 0.4 nozzle",
                        "inherits": "fdm_acme_common",
                        "from": "system",
                        "instantiation": "true",
                        "nozzle_diameter": [0.4],
                        "printer_model": "ACME_M1"
                    },
                    "machine_m2.json": {
                        "type": "machine",
                        "name": "Acme M2 0.8 nozzle",
                        "inherits": "fdm_acme_common",
                        "from": "system",
                        "instantiation": "true",
                        "nozzle_diameter": [0.8],
                        "printer_model": "ACME_M2"
                    },
                    "process_base.json": {
                        "type": "process",
                        "name": "fdm_process_common",
                        "instantiation": "false",
                        "layer_height": 0.2
                    },
                    "process_std.json": {
                        "type": "process",
                        "name": "0.20mm Standard @AcmeM1",
                        "inherits": "fdm_process_common",
                        "from": "system",
                        "instantiation": "true",
                        "compatible_printers": ["Acme M1 0.4 nozzle"]
                    },
                    "process_small.json": {
                        "type": "process",
                        "name": "0.10mm Detail @AcmeM1",
                        "inherits": "fdm_process_common",
                        "from": "system",
                        "instantiation": "true",
                        "compatible_printers": ["Acme M2 0.8 nozzle"]
                    },
                    "process_broken.json": {
                        "type": "process",
                        "name": "0.10mm Broken @AcmeM1",
                        "inherits": "fdm_process_common",
                        "from": "system",
                        "instantiation": "true",
                        "compatible_printers_condition": "nozzle_diameter[0] ==="
                    },
                    "process_template.json": {
                        "type": "process",
                        "name": "fdm_hidden_template",
                        "inherits": "fdm_process_common",
                        "instantiation": "false"
                    },
                    "filament_matched.json": {
                        "type": "filament",
                        "name": "Acme PLA @Acme M1",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "ACF01",
                        "compatible_printers": ["Acme M1 0.4 nozzle"],
                        "compatible_prints": ["0.20mm Standard @AcmeM1"]
                    },
                    "filament_other.json": {
                        "type": "filament",
                        "name": "Acme PLA @Acme M2",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "ACF02",
                        "compatible_printers": ["Acme M2 0.8 nozzle"]
                    },
                    "filament_cond.json": {
                        "type": "filament",
                        "name": "Acme Conditional @Acme",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "ACF03",
                        "compatible_printers": [],
                        "compatible_printers_condition": "nozzle_diameter[0] == 0.4"
                    },
                    "filament_cond_print.json": {
                        "type": "filament",
                        "name": "Acme CondPrint @Acme",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "ACF06",
                        "compatible_printers": [],
                        "compatible_prints_condition": "layer_height == 0.2"
                    },
                    "filament_prints.json": {
                        "type": "filament",
                        "name": "Acme Picky @Acme M1",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "ACF04",
                        "compatible_prints": ["0.10mm Draft @AcmeM1"]
                    },
                    "filament_generic_clone.json": {
                        "type": "filament",
                        "name": "Generic PLA @Acme M1",
                        "inherits": "fdm_filament_pla",
                        "from": "system",
                        "instantiation": "true",
                        "filament_id": "ACF05",
                        "compatible_printers": ["Acme M1 0.4 nozzle"]
                    }
                })
            )
        ],
        "imports": []
    })
    .to_string()
}

fn filtered(result: &ares_core::FilteredPresets) -> Value {
    serde_json::to_value(result).unwrap()
}

#[test]
fn filter_compatible_presets_lists_names_and_diagnostics() {
    let result = ares_core::filter_compatible_presets(
        &catalog_json(),
        "Acme M1 0.4 nozzle",
        Some("0.20mm Standard @AcmeM1"),
    )
    .unwrap();
    let result = filtered(&result);

    // Preset.cpp:837-840 list match; broken condition yields true + diagnostic
    // (Preset.cpp:830-836); the M2-scoped process drops; non-instantiable
    // templates never appear.
    assert_eq!(
        result["processes"],
        json!(["0.20mm Standard @AcmeM1", "0.10mm Broken @AcmeM1"])
    );
    // Preset.cpp:837-840 list match, :828-834 condition match, :789-791
    // compatible_prints gating drops "Acme Picky" (its list targets a
    // different process), :780-787 evaluates compatible_prints_condition on
    // the process config, and :814-821 excludes the library "Generic PLA
    // @System" preset because the same-alias vendor preset "Generic PLA
    // @Acme M1" claims this printer (Preset.cpp:3705-3736). The vendor clone
    // itself stays.
    assert_eq!(
        result["filaments"],
        json!([
            "Acme PLA @Acme M1",
            "Acme Conditional @Acme",
            "Acme CondPrint @Acme",
            "Generic PLA @Acme M1"
        ])
    );
    let diagnostics = result["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["kind"], "process");
    assert_eq!(diagnostics[0]["name"], "0.10mm Broken @AcmeM1");
    assert!(diagnostics[0]["message"].is_string());
}

#[test]
fn filter_compatible_presets_without_process_keeps_printer_scope() {
    let result =
        ares_core::filter_compatible_presets(&catalog_json(), "Acme M1 0.4 nozzle", None).unwrap();
    let result = filtered(&result);

    // Without a process, filament gating by compatible_prints does not apply
    // (Preset.cpp:3364: is_compatible_with_print only runs when a print is
    // active); library filament stays excluded for this printer.
    assert_eq!(
        result["filaments"],
        json!([
            "Acme PLA @Acme M1",
            "Acme Conditional @Acme",
            "Acme CondPrint @Acme",
            "Acme Picky @Acme M1",
            "Generic PLA @Acme M1"
        ])
    );
    assert_eq!(
        result["processes"],
        json!(["0.20mm Standard @AcmeM1", "0.10mm Broken @AcmeM1"])
    );
}

#[test]
fn filter_compatible_presets_user_printer_parent_match() {
    // Preset.cpp:840 + 799-806: a user printer whose immediate `inherits` is a
    // compatible system printer keeps list-matched presets. User presets live
    // in `imports`, not `vendors`.
    let mut catalog: Value = serde_json::from_str(&catalog_json()).unwrap();
    catalog["imports"] = json!([{
        "profiles": [{
            "type": "machine",
            "name": "My Acme M1",
            "inherits": "Acme M1 0.4 nozzle",
            "instantiation": "true",
            "nozzle_diameter": [0.4],
            "printer_model": "ACME_M1"
        }]
    }]);
    let result =
        ares_core::filter_compatible_presets(&catalog.to_string(), "My Acme M1", None).unwrap();
    let result = filtered(&result);
    assert!(
        result["processes"]
            .as_array()
            .unwrap()
            .contains(&json!("0.20mm Standard @AcmeM1"))
    );
}

#[test]
fn parse_preset_bundle_accepts_single_json() {
    let preset = json!({
        "type": "process",
        "name": "Imported @Machine",
        "print_settings_id": "GP999",
        "inherits": "fdm_process_common",
        "version": "1.9.0.0",
        "layer_height": "0.2"
    });
    let parsed = ares_core::parse_preset_bundle(preset.to_string(), "preset.json").unwrap();
    let parsed: Value = serde_json::to_value(parsed).unwrap();

    assert_eq!(parsed["bundle"], Value::Null);
    let profiles = parsed["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0]["type"], "process");
    assert_eq!(profiles[0]["name"], "Imported @Machine");
    assert_eq!(profiles[0]["inherits"], "fdm_process_common");
    assert_eq!(profiles[0]["layer_height"], "0.2");
}

#[test]
fn parse_preset_bundle_rejects_malformed_json() {
    let error = ares_core::parse_preset_bundle("{not json", "preset.json")
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid"), "{error}");
}

#[test]
fn parse_preset_bundle_reads_zip_members() {
    // PresetBundle.cpp:1320-1447: .orca_* archives are ZIP files; members are
    // preset JSONs and an optional bundle_structure.json.
    let preset = json!({
        "name": "Zipped Filament",
        "filament_settings_id": "GFZ01",
        "inherits": "fdm_filament_pla",
        "version": "1.9.0.0",
        "filament_diameter": [1.75]
    });
    let metadata = json!({"id": "bundle-1", "name": "Test Bundle", "version": "1.0.0"});
    let zip = zip_bytes(&[
        ("bundle_structure.json", metadata.to_string().as_bytes()),
        ("filament/pla.json", preset.to_string().as_bytes()),
    ]);

    let parsed = ares_core::parse_preset_bundle(&zip, "test.orca_filament").unwrap();
    let parsed: Value = serde_json::to_value(parsed).unwrap();

    assert_eq!(parsed["bundle"]["id"], "bundle-1");
    assert_eq!(parsed["bundle"]["name"], "Test Bundle");
    let profiles = parsed["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1);
    // PresetBundle.cpp:1469-1482: kind derives from the settings-ID key.
    assert_eq!(profiles[0]["type"], "filament");
    assert_eq!(profiles[0]["name"], "Zipped Filament");
    assert_eq!(profiles[0]["filament_diameter"], json!([1.75]));
}

#[test]
fn parse_preset_bundle_rejects_corrupt_zip() {
    let error = ares_core::parse_preset_bundle(b"PK\x03\x04truncated", "bad.orca_printer")
        .unwrap_err()
        .to_string();
    assert!(!error.is_empty());
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut cursor);
        for (name, bytes) in entries {
            writer
                .start_file::<_, ()>(*name, zip::write::FileOptions::default())
                .unwrap();
            std::io::Write::write_all(&mut writer, bytes).unwrap();
        }
        writer.finish().unwrap();
    }
    cursor.into_inner()
}
