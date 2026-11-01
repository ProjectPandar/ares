//! Native expectations for the browser preset-import E2E
//! (`web/e2e/presetImport.spec.ts`): the same `.orca_printer` bundle, user
//! filament preset and OrcaSlicer vendor files the browser loads from
//! `@ares-slicer/profiles` go through the `PresetCatalog` handle, built as the
//! app builds it: vendors in load order, then the imports.
//! The E2E proves the browser wiring; upstream parity of the compatibility
//! decision is covered by the ares-core profile tests.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const BUNDLE: &str = "My X1C 0.4 nozzle.orca_printer";
/// Inherits a Creality system filament; the Creality vendor stays uninstalled.
const FILAMENT: &str = "My Creality PLA.json";
const PRINTER: &str = "My X1C 0.4 nozzle";
const PROCESS: &str = "0.20mm Standard @BBL X1C - Mine";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Deserialize)]
struct Entry {
    sub_path: String,
}

#[derive(Deserialize)]
struct VendorLists {
    #[serde(default)]
    machine_model_list: Vec<Entry>,
    #[serde(default)]
    process_list: Vec<Entry>,
    #[serde(default)]
    filament_list: Vec<Entry>,
    #[serde(default)]
    machine_list: Vec<Entry>,
}

/// `{id, index, files}` as `@ares-slicer/profiles` `loadVendor(id)` returns it:
/// the vendor index plus every listed file keyed by `sub_path`, values as in
/// the source.
fn vendor_json(id: &str) -> String {
    let root = repo_root().join("OrcaSlicer/resources/profiles");
    let index = fs::read_to_string(root.join(format!("{id}.json"))).unwrap();
    let lists: VendorLists = serde_json::from_str(&index).unwrap();
    let files = [
        lists.machine_model_list,
        lists.process_list,
        lists.filament_list,
        lists.machine_list,
    ]
    .into_iter()
    .flatten()
    .map(|Entry { sub_path }| {
        let file = fs::read_to_string(root.join(id).join(&sub_path)).unwrap();
        format!("{}:{file}", serde_json::to_string(&sub_path).unwrap())
    })
    .collect::<Vec<_>>()
    .join(",");
    format!(
        r#"{{"id":{},"index":{index},"files":{{{files}}}}}"#,
        serde_json::to_string(id).unwrap()
    )
}

#[test]
fn browser_compatible_presets_expectations() {
    let presets = repo_root().join("tests/presets");
    let imports = [BUNDLE, FILAMENT].map(|name| (fs::read(presets.join(name)).unwrap(), name));
    // PresetBundle.cpp:2216-2234: OrcaFilamentLibrary loads before the other vendors.
    let catalog = |vendors: &[&str]| {
        let mut catalog = ares_wasm::PresetCatalog::new();
        for id in vendors {
            catalog.add_vendor_json(vendor_json(id).as_bytes()).unwrap();
        }
        for (bytes, name) in &imports {
            catalog.add_import_json(bytes, name).unwrap();
        }
        catalog
    };

    // Without the BBL vendor installed the imported printer's parent is missing.
    let missing = catalog(&["OrcaFilamentLibrary"])
        .filter_compatible_json(PRINTER, None)
        .unwrap_err();
    assert!(missing.contains("is missing parent"), "{missing}");

    let installed = catalog(&["OrcaFilamentLibrary", "BBL"]);
    let without_process = installed.filter_compatible_json(PRINTER, None).unwrap();
    let with_process = installed
        .filter_compatible_json(PRINTER, Some(PROCESS))
        .unwrap();
    assert!(without_process.contains(PROCESS), "{without_process}");
    // Preset.cpp:1685-1691: the Creality-based user filament is skipped, not filtered.
    assert!(
        with_process.contains(r#""missing_parents":[{"kind":"filament","name":"My Creality PLA","parent":"Creality Generic PLA"}]"#),
        "{with_process}"
    );
    assert!(
        with_process.contains("Bambu PLA Basic @BBL X1C - Mine"),
        "{with_process}"
    );

    let artifacts = repo_root().join("target/web-e2e");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(
        artifacts.join("expected-compatible.json"),
        format!(
            r#"{{"bundle":{},"filament":{},"printer":{},"process":{},"missingVendorError":{},"withoutProcess":{without_process},"withProcess":{with_process}}}"#,
            serde_json::to_string(BUNDLE).unwrap(),
            serde_json::to_string(FILAMENT).unwrap(),
            serde_json::to_string(PRINTER).unwrap(),
            serde_json::to_string(PROCESS).unwrap(),
            serde_json::to_string(&missing).unwrap(),
        ),
    )
    .unwrap();
}
