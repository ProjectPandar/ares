//! The print preset a printer selection holds: `PresetCollection::update_compatible` with
//! `PresetSelectCompatibleType::Always` (`Preset.hpp:877-885`, `Preset.cpp:3965-4010`),
//! choosing by `PresetBundle::update_compatible`'s `PreferedPrintProfileMatch`
//! (`PresetBundle.cpp:8522-8572`, `:8635-8639`) in `first_compatible_idx` order
//! (`Preset.hpp:831-855`).
//! Failure inventory: a compatible current print replaced; the printer's `default_print_profile`
//! ignored without a current print; an incompatible current print not mapped to the compatible
//! print of the same alias, or mapped by layer height instead; the layer height of the current
//! print not preferred over the default profile when no alias matches; a user print matched by
//! its parent's alias instead of its own whole name; a catalog without normal prints failing
//! to resolve its source-defined built-in default.

use std::fs;
use std::path::Path;

use ares_core::{PresetCatalogHandle, ProfileKind};
use serde_json::{Value, json};

const KLIPPER: &str = "MyKlipper 0.4 nozzle";
const A1_MINI: &str = "Bambu Lab A1 mini 0.4 nozzle";
const PRESET_LISTS: [&str; 3] = ["machine_model_list", "machine_list", "process_list"];

fn profiles() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OrcaSlicer/resources/profiles")
}

/// OrcaSlicer's vendor `id` as the `@ares-slicer/profiles` `{id, index, files}` envelope, with the
/// files of the given lists.
fn raw_vendor(id: &str, lists: &[&str]) -> Value {
    let index: Value =
        serde_json::from_str(&fs::read_to_string(profiles().join(format!("{id}.json"))).unwrap())
            .unwrap();
    let mut files = serde_json::Map::new();
    let mut kept = json!({"name": index["name"]});
    for list in lists {
        kept[*list] = index[*list].clone();
        for entry in index[*list].as_array().unwrap() {
            let sub_path = entry["sub_path"].as_str().unwrap();
            let file = fs::read_to_string(profiles().join(id).join(sub_path)).unwrap();
            files.insert(sub_path.to_owned(), serde_json::from_str(&file).unwrap());
        }
    }
    json!({"id": id, "index": kept, "files": files})
}

fn catalog(vendor: &str, lists: &[&str]) -> PresetCatalogHandle {
    let mut catalog = PresetCatalogHandle::new();
    catalog
        .add_vendor(&serde_json::to_vec(&raw_vendor(vendor, lists)).unwrap())
        .unwrap();
    catalog
}

/// The compatible prints of `printer`, in collection order.
fn compatible(catalog: &PresetCatalogHandle, printer: &str) -> Vec<String> {
    let filtered = serde_json::to_value(catalog.filter_compatible(printer, None).unwrap()).unwrap();
    serde_json::from_value(filtered["processes"].clone()).unwrap()
}

fn value(catalog: &PresetCatalogHandle, kind: ProfileKind, name: &str, key: &str) -> String {
    let values = serde_json::to_value(
        catalog
            .preset_values(kind, &[name.to_owned()], &[key])
            .unwrap(),
    )
    .unwrap();
    values[0][key].as_str().unwrap().to_owned()
}

fn default_print(catalog: &PresetCatalogHandle, printer: &str) -> String {
    value(
        catalog,
        ProfileKind::Machine,
        printer,
        "default_print_profile",
    )
}

fn layer_height(catalog: &PresetCatalogHandle, print: &str) -> String {
    value(catalog, ProfileKind::Process, print, "layer_height")
}

/// A system print's alias: the name before its `@` (`PresetBundle.cpp:4943-4944`).
fn alias(name: &str) -> &str {
    name.split('@').next().unwrap().trim()
}

#[test]
fn without_a_current_print_the_printer_default_print_profile_is_chosen() {
    let catalog = catalog("Custom", &PRESET_LISTS);
    let default = default_print(&catalog, KLIPPER);
    assert!(compatible(&catalog, KLIPPER).contains(&default));
    assert_eq!(
        catalog.compatible_print(KLIPPER, None).unwrap(),
        Some(default)
    );
}

#[test]
fn a_compatible_current_print_stays_selected() {
    let catalog = catalog("Custom", &PRESET_LISTS);
    let default = default_print(&catalog, KLIPPER);
    let current = compatible(&catalog, KLIPPER)
        .into_iter()
        .rfind(|name| *name != default)
        .unwrap();
    assert_eq!(
        catalog.compatible_print(KLIPPER, Some(&current)).unwrap(),
        Some(current)
    );
}

/// The A1 mini's default print has the same layer height as the current print's alias
/// counterpart, so only the alias rule picks the counterpart.
#[test]
fn an_incompatible_current_print_maps_to_the_compatible_print_of_its_alias() {
    let catalog = catalog("BBL", &PRESET_LISTS);
    let prints = compatible(&catalog, A1_MINI);
    let current = "0.20mm Strength @BBL X1C";
    assert!(!prints.iter().any(|name| name == current));
    let expected = prints
        .iter()
        .find(|name| alias(name) == alias(current))
        .unwrap();
    let default = default_print(&catalog, A1_MINI);
    assert_ne!(*expected, default);
    assert_eq!(
        layer_height(&catalog, &default),
        layer_height(&catalog, current)
    );
    assert_eq!(
        catalog
            .compatible_print(A1_MINI, Some(current))
            .unwrap()
            .as_ref(),
        Some(expected)
    );
}

#[test]
fn without_an_alias_match_the_current_layer_height_outranks_the_default_profile() {
    let catalog = catalog("Custom", &PRESET_LISTS);
    let prints = compatible(&catalog, KLIPPER);
    let current = "0.32mm Extra Draft @MyToolChanger";
    assert!(
        !prints
            .iter()
            .any(|name| name == current || alias(name) == alias(current))
    );
    let height = layer_height(&catalog, current);
    assert_ne!(
        layer_height(&catalog, &default_print(&catalog, KLIPPER)),
        height
    );
    let expected = prints
        .iter()
        .find(|name| layer_height(&catalog, name) == height)
        .unwrap();
    assert_eq!(
        catalog
            .compatible_print(KLIPPER, Some(current))
            .unwrap()
            .as_ref(),
        Some(expected)
    );
}

#[test]
fn a_catalog_without_prints_returns_source_defined_default_setting() {
    let catalog = catalog("Custom", &["machine_model_list", "machine_list"]);
    // Preset.hpp:573-579 / first_compatible_idx: source-defined default identity.
    assert_eq!(
        catalog.compatible_print(KLIPPER, None).unwrap().as_deref(),
        Some("Default Setting")
    );
}

/// `PresetCollection::set_custom_preset_alias` (`Preset.cpp:4414-4437`): a user print keeps its
/// whole bare name as its alias, so a user copy of a system print matches by layer height, not by
/// the system print's alias.
#[test]
fn a_user_print_matches_by_layer_height_rather_than_its_parent_alias() {
    let mut catalog = catalog("BBL", &PRESET_LISTS);
    let parent = "0.20mm Strength @BBL X1C";
    let current = format!("{parent} - Mine");
    let import = json!({
        "type": "process", "name": current, "inherits": parent,
        "from": "User", "instantiation": "true", "version": "1.9.0.0"
    });
    catalog
        .add_import(import.to_string().as_bytes(), "mine.json")
        .unwrap();
    let prints = compatible(&catalog, A1_MINI);
    assert!(!prints.contains(&current));
    let default = default_print(&catalog, A1_MINI);
    assert!(
        prints
            .iter()
            .any(|name| name != &default && alias(name) == alias(parent))
    );
    assert_eq!(
        layer_height(&catalog, &default),
        layer_height(&catalog, &current)
    );
    assert_eq!(
        catalog.compatible_print(A1_MINI, Some(&current)).unwrap(),
        Some(default)
    );
}
