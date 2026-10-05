//! Native expectations for the browser preset-import E2E
//! (`web/e2e/presetImport.spec.ts`): the same `.orca_printer` bundle and the
//! same OrcaSlicer vendor files the browser loads from `@ares-slicer/profiles`
//! go through `parse_preset_bundle_json` / `filter_compatible_presets_json`.
//! The E2E proves the browser wiring; upstream parity of the compatibility
//! decision is covered by the ares-core profile tests.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const BUNDLE: &str = "My X1C 0.4 nozzle.orca_printer";
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

/// `{index, files}` as `@ares-slicer/profiles` `loadVendor(id)` returns it:
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
    format!(r#"{{"index":{index},"files":{{{files}}}}}"#)
}

#[test]
fn browser_compatible_presets_expectations() {
    let bundle = fs::read(repo_root().join("tests/presets").join(BUNDLE)).unwrap();
    let imports = format!(
        "[{}]",
        ares_wasm::parse_preset_bundle_json(&bundle, BUNDLE).unwrap()
    );
    let library = vendor_json("OrcaFilamentLibrary");
    let catalog = |vendors: &str| format!(r#"{{"vendors":[{vendors}],"imports":{imports}}}"#);

    // Without the BBL vendor installed the imported printer's parent is missing.
    let missing =
        ares_wasm::filter_compatible_presets_json(&catalog(&library), PRINTER, None).unwrap_err();
    assert!(missing.contains("parent profile"), "{missing}");

    // PresetBundle.cpp:2216-2234: OrcaFilamentLibrary loads before the other vendors.
    let installed = catalog(&format!("{library},{}", vendor_json("BBL")));
    let without_process =
        ares_wasm::filter_compatible_presets_json(&installed, PRINTER, None).unwrap();
    let with_process =
        ares_wasm::filter_compatible_presets_json(&installed, PRINTER, Some(PROCESS)).unwrap();
    assert!(without_process.contains(PROCESS), "{without_process}");
    assert!(
        with_process.contains("Bambu PLA Basic @BBL X1C - Mine"),
        "{with_process}"
    );

    let artifacts = repo_root().join("target/web-e2e");
    fs::create_dir_all(&artifacts).unwrap();
    fs::write(
        artifacts.join("expected-compatible.json"),
        format!(
            r#"{{"bundle":{},"printer":{},"process":{},"missingVendorError":{},"withoutProcess":{without_process},"withProcess":{with_process}}}"#,
            serde_json::to_string(BUNDLE).unwrap(),
            serde_json::to_string(PRINTER).unwrap(),
            serde_json::to_string(PROCESS).unwrap(),
            serde_json::to_string(&missing).unwrap(),
        ),
    )
    .unwrap();
}
