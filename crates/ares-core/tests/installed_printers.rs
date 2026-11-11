//! Installed printers (`AppConfig` vendors section) and the setup guide's model and filament data:
//! `Preset::set_visible_from_appconfig` (`Preset.cpp:967-995`),
//! `PrinterPresetCollection::find_system_preset_by_model_and_variant` (`Preset.cpp:4569-4581`),
//! `VendorProfile::PrinterModel` (`PresetBundle.cpp:4712-4821`) and
//! `GuideFrame::BuildProfileJson` (`WebGuideDialog.cpp:1385-1460`).
//! Failure inventory: an installed model's variant missing from the listing; another model's or
//! an uninstalled variant's printer listed; an imported printer or a printer without a model or
//! variant hidden; a catalog without an installed set filtered; the model listing not following
//! visibility; `system_printer` picking another preset or skipping hidden ones; guide models with
//! wrong variants or materials; filament tags naming printers outside `compatible_printers`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use ares_core::{GuideFilament, GuideModel, InstalledPrinters, PresetCatalogHandle, guide_models};
use serde_json::{Value, json};

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

fn catalog(vendors: &[Value]) -> PresetCatalogHandle {
    let mut catalog = PresetCatalogHandle::new();
    for vendor in vendors {
        catalog
            .add_vendor(&serde_json::to_vec(vendor).unwrap())
            .unwrap();
    }
    catalog
}

fn bbl_machines() -> Value {
    raw_vendor("BBL", &["machine_model_list", "machine_list"])
}

fn installed(vendor: &str, model: &str, variants: &[&str]) -> InstalledPrinters {
    BTreeMap::from([(
        vendor.to_owned(),
        BTreeMap::from([(
            model.to_owned(),
            variants
                .iter()
                .map(|variant| (*variant).to_owned())
                .collect(),
        )]),
    )])
}

/// `printers()` names, in listing order.
fn names(catalog: &PresetCatalogHandle) -> Vec<String> {
    catalog
        .printers()
        .into_iter()
        .map(|printer| printer.name)
        .collect()
}

/// The machine model files of `BBL.json`, read the way `VendorProfile::PrinterModel` reads them:
/// `(model id, name, nozzle variants, default materials)`, without the models that have no nozzle
/// variant (`PresetBundle.cpp:4819-4820`).
fn bbl_model_files() -> Vec<(String, String, Vec<String>, Vec<String>)> {
    let index: Value =
        serde_json::from_str(&fs::read_to_string(profiles().join("BBL.json")).unwrap()).unwrap();
    index["machine_model_list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let sub_path = entry["sub_path"].as_str().unwrap();
            let file: Value = serde_json::from_str(
                &fs::read_to_string(profiles().join("BBL").join(sub_path)).unwrap(),
            )
            .unwrap();
            let split = |key: &str| -> Vec<String> {
                file[key]
                    .as_str()
                    .unwrap_or_default()
                    .split(';')
                    .filter(|item| !item.is_empty())
                    .map(str::to_owned)
                    .collect()
            };
            let mut variants = Vec::new();
            for variant in split("nozzle_diameter") {
                if !variants.contains(&variant) {
                    variants.push(variant);
                }
            }
            let materials: BTreeSet<String> = split("default_materials").into_iter().collect();
            (
                entry["name"].as_str().unwrap().to_owned(),
                file["name"].as_str().unwrap().to_owned(),
                variants,
                materials.into_iter().collect(),
            )
        })
        .filter(|(_, _, variants, _)| !variants.is_empty())
        .collect()
}

#[test]
fn installing_one_model_lists_exactly_its_installed_variants() {
    let mut catalog = catalog(&[bbl_machines()]);
    let all = catalog.printers();
    let mini: Vec<_> = all
        .iter()
        .filter(|printer| printer.printer_model == "Bambu Lab A1 mini")
        .collect();
    assert!(mini.len() > 1, "the A1 mini has several nozzle presets");

    let every_variant: Vec<&str> = mini
        .iter()
        .map(|printer| printer.printer_variant.as_str())
        .collect();
    catalog.set_installed_printers(installed("BBL", "Bambu Lab A1 mini", &every_variant));
    assert_eq!(
        names(&catalog),
        mini.iter()
            .map(|printer| printer.name.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        catalog
            .printer_models()
            .into_iter()
            .map(|group| group.model)
            .collect::<Vec<_>>(),
        ["Bambu Lab A1 mini"]
    );

    // AppConfig::get_variant is per variant: an installed model shows only its installed nozzles.
    catalog.set_installed_printers(installed("BBL", "Bambu Lab A1 mini", &[every_variant[0]]));
    assert_eq!(names(&catalog), [mini[0].name.clone()]);

    // A vendor id other than the printer's owning vendor installs nothing.
    catalog.set_installed_printers(installed("Acme", "Bambu Lab A1 mini", &every_variant));
    assert!(catalog.printers().is_empty());
    assert!(catalog.printer_models().is_empty());
}

#[test]
fn a_catalog_without_an_installed_set_lists_every_printer() {
    let catalog = catalog(&[bbl_machines()]);
    let models: BTreeSet<String> = catalog
        .printers()
        .into_iter()
        .map(|printer| printer.printer_model)
        .collect();
    assert!(models.len() > 5, "{models:?}");
}

fn acme_vendor() -> Value {
    json!({
        "id": "Acme",
        "index": {
            "name": "Acme",
            "machine_list": [
                {"name": "fdm_acme_common", "sub_path": "machine/fdm_acme_common.json"},
                {"name": "Acme A1 0.4 nozzle", "sub_path": "machine/a1_04.json"},
                {"name": "Acme A1 0.6 nozzle", "sub_path": "machine/a1_06.json"},
                {"name": "Acme B1", "sub_path": "machine/b1.json"},
                {"name": "Acme Loose", "sub_path": "machine/loose.json"}
            ],
            "filament_list": [
                {"name": "fdm_filament_common", "sub_path": "filament/fdm_filament_common.json"},
                {"name": "Acme PLA", "sub_path": "filament/pla.json"},
                {"name": "Acme PETG A1", "sub_path": "filament/petg.json"}
            ]
        },
        "files": {
            "machine/fdm_acme_common.json": {
                "type": "machine", "name": "fdm_acme_common", "from": "system",
                "instantiation": "false", "nozzle_diameter": ["0.4"]
            },
            "machine/a1_04.json": {
                "type": "machine", "name": "Acme A1 0.4 nozzle", "inherits": "fdm_acme_common",
                "from": "system", "instantiation": "true",
                "printer_model": "Acme A1", "printer_variant": "0.4"
            },
            "machine/a1_06.json": {
                "type": "machine", "name": "Acme A1 0.6 nozzle", "inherits": "fdm_acme_common",
                "from": "system", "instantiation": "true",
                "printer_model": "Acme A1", "printer_variant": "0.6", "nozzle_diameter": ["0.6"]
            },
            "machine/b1.json": {
                "type": "machine", "name": "Acme B1", "inherits": "fdm_acme_common",
                "from": "system", "instantiation": "true",
                "printer_model": "Acme B1", "printer_variant": "0.4"
            },
            "machine/loose.json": {
                "type": "machine", "name": "Acme Loose", "inherits": "fdm_acme_common",
                "from": "system", "instantiation": "true", "printer_model": "Acme Loose"
            },
            "filament/fdm_filament_common.json": {
                "type": "filament", "name": "fdm_filament_common", "from": "system",
                "instantiation": "false", "filament_type": ["PLA"], "filament_vendor": ["Acme"]
            },
            "filament/pla.json": {
                "type": "filament", "name": "Acme PLA", "inherits": "fdm_filament_common",
                "from": "system", "instantiation": "true"
            },
            "filament/petg.json": {
                "type": "filament", "name": "Acme PETG A1", "inherits": "fdm_filament_common",
                "from": "system", "instantiation": "true", "filament_type": ["PETG"],
                "compatible_printers": ["Acme A1 0.6 nozzle", "Acme B1", "Not A Printer"]
            }
        }
    })
}

#[test]
fn printers_without_a_model_or_variant_and_imported_printers_stay_visible() {
    let mut catalog = catalog(&[acme_vendor()]);
    catalog
        .add_import(
            &serde_json::to_vec(&json!({
                "type": "machine", "name": "My A1", "inherits": "Acme A1 0.4 nozzle",
                "from": "User", "instantiation": "true", "version": "1.9.0.0", "printer_model": "Acme A1", "printer_variant": "0.4"
            }))
            .unwrap(),
            "My A1.json",
        )
        .unwrap();
    catalog.set_installed_printers(BTreeMap::new());
    // Preset.cpp:971 (no vendor) and :976-977 (an empty printer_variant) keep the visibility.
    assert_eq!(names(&catalog), ["Acme Loose", "My A1"]);
    assert_eq!(
        catalog
            .printer_models()
            .into_iter()
            .map(|group| group.model)
            .collect::<Vec<_>>(),
        ["Acme Loose"]
    );

    catalog.set_installed_printers(installed("Acme", "Acme A1", &["0.6"]));
    assert_eq!(
        names(&catalog),
        ["Acme A1 0.6 nozzle", "Acme Loose", "My A1"]
    );
}

#[test]
fn system_printer_finds_the_first_system_preset_of_a_model_and_variant() {
    let mut catalog = catalog(&[acme_vendor()]);
    catalog.set_installed_printers(BTreeMap::new());
    // Preset.cpp:4573-4579 ignores visibility and user presets; an empty variant matches any.
    assert_eq!(
        catalog.system_printer("Acme A1", "0.6").as_deref(),
        Some("Acme A1 0.6 nozzle")
    );
    assert_eq!(
        catalog.system_printer("Acme A1", "").as_deref(),
        Some("Acme A1 0.4 nozzle")
    );
    assert_eq!(catalog.system_printer("Acme A1", "0.8"), None);
    assert_eq!(catalog.system_printer("", "0.4"), None);

    let bbl = self::catalog(&[bbl_machines()]);
    assert_eq!(
        bbl.system_printer("Bambu Lab A1 mini", "0.4").as_deref(),
        Some("Bambu Lab A1 mini 0.4 nozzle")
    );
}

#[test]
fn guide_models_read_every_model_of_a_vendor_with_its_variants_and_materials() {
    let vendor = raw_vendor("BBL", &["machine_model_list"]);
    let models: Vec<GuideModel> =
        guide_models(&serde_json::to_vec(&json!([vendor])).unwrap()).unwrap();
    let expected = bbl_model_files();
    assert_eq!(models.len(), expected.len());
    for (model, (id, name, variants, materials)) in models.iter().zip(&expected) {
        assert_eq!(model.vendor, "BBL");
        assert_eq!(&model.model, id);
        assert_eq!(&model.name, name);
        assert_eq!(&model.variants, variants, "{id}");
        assert_eq!(&model.default_materials, materials, "{id}");
    }
    let mini = models
        .iter()
        .find(|model| model.model == "Bambu Lab A1 mini")
        .unwrap();
    assert!(mini.variants.len() > 1, "{:?}", mini.variants);
}

#[test]
fn guide_filaments_tag_each_filament_with_its_compatible_models_and_variants() {
    let catalog = catalog(&[acme_vendor()]);
    let filaments: Vec<GuideFilament> = catalog.guide_filaments();
    // WebGuideDialog.cpp:1417-1460: vendor and type from the resolved config, and one
    // (printer_model, printer_variant) tag per compatible system printer with a model.
    assert_eq!(
        filaments,
        [
            GuideFilament {
                name: "Acme PETG A1".to_owned(),
                vendor: "Acme".to_owned(),
                filament_type: "PETG".to_owned(),
                models: vec![
                    ("Acme A1".to_owned(), "0.6".to_owned()),
                    ("Acme B1".to_owned(), "0.4".to_owned()),
                ],
            },
            GuideFilament {
                name: "Acme PLA".to_owned(),
                vendor: "Acme".to_owned(),
                filament_type: "PLA".to_owned(),
                models: Vec::new(),
            },
        ]
    );
}

#[test]
fn visibility_reads_the_variant_a_printer_inherits() {
    // `Anycubic Kobra 0.4 nozzle` sets no `printer_variant`; its parent `fdm_machine_common` does.
    let mut catalog = catalog(&[raw_vendor(
        "Anycubic",
        &["machine_model_list", "machine_list"],
    )]);
    catalog.set_installed_printers(installed("Anycubic", "Anycubic Kobra", &["0.4"]));
    let printers: Vec<(String, String, String)> = catalog
        .printers()
        .into_iter()
        .map(|printer| (printer.name, printer.printer_model, printer.printer_variant))
        .collect();
    assert_eq!(
        printers,
        [(
            "Anycubic Kobra 0.4 nozzle".to_owned(),
            "Anycubic Kobra".to_owned(),
            "0.4".to_owned()
        )]
    );
}

#[test]
fn installed_filaments_seed_only_the_visible_printers_default_materials() {
    let mut catalog = catalog(&[raw_vendor(
        "BBL",
        &["machine_model_list", "machine_list", "filament_list"],
    )]);
    let every = catalog.installed_filaments(&[]).unwrap();
    let mini = catalog
        .printers()
        .into_iter()
        .filter(|printer| printer.printer_model == "Bambu Lab A1 mini")
        .map(|printer| printer.printer_variant)
        .collect::<Vec<_>>();
    catalog.set_installed_printers(installed(
        "BBL",
        "Bambu Lab A1 mini",
        &mini.iter().map(String::as_str).collect::<Vec<_>>(),
    ));
    let (_, _, _, materials) = bbl_model_files()
        .into_iter()
        .find(|(model, ..)| model == "Bambu Lab A1 mini")
        .unwrap();
    let names: BTreeSet<String> = catalog
        .guide_filaments()
        .into_iter()
        .map(|filament| filament.name)
        .collect();
    // PresetBundle.cpp:3767: only visible printers add their model's loaded default materials.
    let expected: Vec<String> = materials
        .into_iter()
        .filter(|name| names.contains(name))
        .collect();
    assert!(!expected.is_empty());
    assert_eq!(catalog.installed_filaments(&[]).unwrap(), expected);
    assert!(every.len() > expected.len(), "{every:?}");
}
