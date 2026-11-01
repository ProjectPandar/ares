//! `printer_models` grouping and `similar_printer_preset` resolution:
//! `PresetComboBoxes.cpp:1243-1251` (system models deduped into a
//! `std::map<wxString>`) and `PresetBundle.cpp:3750-3782` (model/variant
//! resolution).

use std::fs;
use std::path::Path;

use ares_core::{CurrentPrinter, PresetCatalogHandle};
use serde::Deserialize;
use serde_json::{Value, json};

fn vendor(name: &str, lists: Value, files: Value) -> Value {
    let mut index = json!({"name": name});
    if let Some(entries) = lists.get("machine_list") {
        index["machine_list"] = entries.clone();
    }
    json!({"index": index, "files": files})
}

fn entry(sub_path: &str) -> Value {
    json!({"sub_path": sub_path})
}

fn machine(sub_path: &str, name: &str, model: &str, variant: &str) -> (String, Value) {
    let mut fields = json!({
        "type": "machine", "name": name, "from": "system", "instantiation": "true",
        "nozzle_diameter": [0.4]
    });
    if !model.is_empty() {
        fields["printer_model"] = json!(model);
    }
    if !variant.is_empty() {
        fields["printer_variant"] = json!(variant);
    }
    (sub_path.to_owned(), fields)
}

fn acme_vendor() -> Value {
    let files = [
        machine("base.json", "fdm_acme_common", "", ""),
        machine(
            "stinger_04.json",
            "LH Stinger 0.4 nozzle",
            "LH Stinger",
            "0.4",
        ),
        machine(
            "stinger_06.json",
            "LH Stinger 0.6 nozzle",
            "LH Stinger",
            "0.6",
        ),
        machine(
            "stinger_mmu.json",
            "LH Stinger MMU 0.4 nozzle",
            "LH Stinger",
            "0.4",
        ),
        machine("harrier_04.json", "Harrier 0.4 nozzle", "Harrier", "0.4"),
        machine("template.json", "fdm_acme_template", "ACME", ""),
    ]
    .into_iter()
    .collect::<serde_json::Map<_, _>>();
    let mut files = Value::Object(files);
    files["template.json"]["instantiation"] = json!("false");
    files["base.json"]["instantiation"] = json!("false");
    vendor(
        "Acme",
        json!({
            "machine_list": [
                entry("base.json"), entry("stinger_04.json"), entry("stinger_06.json"),
                entry("stinger_mmu.json"), entry("harrier_04.json"), entry("template.json")
            ]
        }),
        files,
    )
}

fn stinger_04_with_alias() -> Value {
    let mut v = acme_vendor();
    v["files"]["stinger_04.json"]["alias"] = json!("My Stinger");
    v
}

fn handle(vendor_json: &Value) -> PresetCatalogHandle {
    let mut handle = PresetCatalogHandle::new();
    handle
        .add_vendor(vendor_json.to_string().as_bytes())
        .unwrap();
    handle
}

fn models(handle: &PresetCatalogHandle) -> Vec<Value> {
    handle
        .printer_models()
        .iter()
        .map(|group| json!({"model": group.model}))
        .collect()
}

#[derive(Deserialize)]
struct Entry {
    sub_path: String,
}

#[derive(Deserialize)]
struct MachineLists {
    machine_model_list: Vec<Entry>,
    machine_list: Vec<Entry>,
}

/// The raw OrcaSlicer vendor's machine files as `{index, files}` keyed by
/// `sub_path`, the `@ares-slicer/profiles` `loadVendor` shape.
fn raw_vendor(id: &str) -> PresetCatalogHandle {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OrcaSlicer/resources/profiles");
    let index = fs::read_to_string(root.join(format!("{id}.json"))).unwrap();
    let lists: MachineLists = serde_json::from_str(&index).unwrap();
    let files = lists
        .machine_model_list
        .into_iter()
        .chain(lists.machine_list)
        .map(|Entry { sub_path }| {
            let file = fs::read_to_string(root.join(id).join(&sub_path)).unwrap();
            (sub_path, serde_json::from_str::<Value>(&file).unwrap())
        })
        .collect::<serde_json::Map<_, _>>();
    let index: Value = serde_json::from_str(&index).unwrap();
    let index = json!({"name": index["name"], "machine_model_list": index["machine_model_list"], "machine_list": index["machine_list"]});
    handle(&json!({"index": index, "files": files}))
}

#[test]
fn printer_models_follow_the_system_group_key_order() {
    // PresetComboBoxes.cpp:1150, 1247-1249: the "System" group is a
    // std::map<wxString> keyed by printer_model, appended in key order
    // (:1324-1327, 1427). wxString::compare is std::basic_string::compare of
    // the wide representation (wxWidgets src/common/string.cpp), i.e. code
    // point order. Beta 345efec64 resources/profiles/BBL.json:7-64 lists
    // these 14 models in that same order (A2L:17-18, H2C:21-22).
    // The machine_list order puts H2D/H2D Pro after X1E.
    let expected = [
        "Bambu Lab A1",
        "Bambu Lab A1 mini",
        "Bambu Lab A2L",
        "Bambu Lab H2C",
        "Bambu Lab H2D",
        "Bambu Lab H2D Pro",
        "Bambu Lab H2S",
        "Bambu Lab P1P",
        "Bambu Lab P1S",
        "Bambu Lab P2S",
        "Bambu Lab X1",
        "Bambu Lab X1 Carbon",
        "Bambu Lab X1E",
        "Bambu Lab X2D",
    ];
    let models = raw_vendor("BBL").printer_models();
    assert_eq!(
        models
            .iter()
            .map(|group| group.model.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn printer_models_dedupe_system_printers_by_model() {
    // PresetComboBoxes.cpp:1243-1251: system printer presets display
    // `printer_model`; a set dedupes, so each model appears once, in map
    // key order. Templates (instantiation "false") and missing-parent
    // printers never enter the listing.
    let mut h = handle(&acme_vendor());
    // Add an orphan machine to prove missing parents stay out.
    let orphan = json!({
        "type": "machine", "name": "Acme Orphan", "inherits": "fdm_missing",
        "instantiation": "true", "printer_model": "Orphan"
    });
    h.add_import(orphan.to_string().as_bytes(), "orphan.json")
        .unwrap();
    assert_eq!(
        models(&h),
        vec![json!({"model": "Harrier"}), json!({"model": "LH Stinger"})]
    );
}

#[test]
fn printer_models_exclude_user_printers() {
    // Upstream lists user printers in their own section by name; only
    // `is_system` presets enter the model list (`:1244`).
    let mut h = handle(&acme_vendor());
    let user = json!({
        "type": "machine", "name": "My Stinger Custom", "inherits": "LH Stinger 0.4 nozzle",
        "from": "User", "instantiation": "true", "version": "1.9.0.0",
        "printer_model": "LH Stinger", "printer_variant": "0.8"
    });
    h.add_import(user.to_string().as_bytes(), "my.json")
        .unwrap();
    assert_eq!(
        models(&h),
        vec![json!({"model": "Harrier"}), json!({"model": "LH Stinger"})]
    );
}

#[test]
fn similar_resolves_by_exact_alias_match() {
    // PresetBundle.cpp:3768-3776: prefer_printer = the current preset's
    // alias with the old variant replaced by the requested one.
    let h = handle(&stinger_04_with_alias());
    // The explicit alias has no variant substring, so the substitution
    // cannot hit and resolution falls through to the variant match.
    assert_eq!(
        h.similar_printer_preset(
            "LH Stinger",
            "0.6",
            CurrentPrinter {
                name: "LH Stinger 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        Some("LH Stinger 0.6 nozzle".to_owned())
    );
    // A derived alias (the bare name) contains the old variant; the
    // substitution hits the 0.6 preset exactly (`:3768-3776`).
    let h2 = handle(&acme_vendor());
    assert_eq!(
        h2.similar_printer_preset(
            "LH Stinger",
            "0.6",
            CurrentPrinter {
                name: "LH Stinger MMU 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        Some("LH Stinger 0.6 nozzle".to_owned())
    );
}

#[test]
fn similar_resolves_by_variant_then_first() {
    let h = handle(&acme_vendor());
    // No alias: prefer_printer substitution misses; the first preset whose
    // printer_variant matches wins (`:3777-3780`).
    assert_eq!(
        h.similar_printer_preset(
            "LH Stinger",
            "0.6",
            CurrentPrinter {
                name: "LH Stinger 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        Some("LH Stinger 0.6 nozzle".to_owned())
    );
    // No variant match either (MMU-only model): the first candidate in name
    // order wins (`:3781`).
    assert_eq!(
        h.similar_printer_preset(
            "LH Stinger",
            "0.8",
            CurrentPrinter {
                name: "LH Stinger 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        Some("LH Stinger 0.4 nozzle".to_owned())
    );
}

#[test]
fn similar_with_empty_variant_considers_system_only() {
    // `:3757-3759`: an empty requested variant skips non-system presets;
    // the current preset's own variant is then the fallback matcher.
    let mut h = handle(&acme_vendor());
    let user = json!({
        "type": "machine", "name": "User Harrier 0.6", "inherits": "Harrier 0.4 nozzle",
        "from": "User", "instantiation": "true", "version": "1.9.0.0",
        "printer_model": "Harrier", "printer_variant": "0.6"
    });
    h.add_import(user.to_string().as_bytes(), "user_harrier.json")
        .unwrap();
    assert_eq!(
        h.similar_printer_preset(
            "Harrier",
            "",
            CurrentPrinter {
                name: "Harrier 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        Some("Harrier 0.4 nozzle".to_owned())
    );
    assert_eq!(
        h.similar_printer_preset(
            "No Such Model",
            "0.4",
            CurrentPrinter {
                name: "Harrier 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        None
    );
}

#[test]
fn similar_missing_model_returns_none() {
    // `:3755-3756`: no candidates → nullptr upstream → None here.
    let h = handle(&acme_vendor());
    assert_eq!(
        h.similar_printer_preset(
            "Ghost",
            "0.4",
            CurrentPrinter {
                name: "LH Stinger 0.4 nozzle",
                printer_variant: "0.4"
            }
        ),
        None
    );
}

#[test]
fn similar_with_current_outside_the_catalog_matches_the_passed_variant() {
    // A 3MF project printer is not a catalog preset: its alias cannot be
    // looked up, so the exact-alias step (`:3765-3772`) is skipped and the
    // caller's `printer_variant` stands in for `printer_variant_old`
    // (`:3755`, `:3773-3780`). `tests/parity/orca_cli_ender3.3mf` carries
    // printer_variant "0.4"; BBL "Bambu Lab X1 Carbon" ships 0.2/0.4/0.6/0.8
    // nozzle presets, and "0.2 nozzle" is first in name order (`:3781`).
    let current = CurrentPrinter {
        name: "Creality Ender-3 0.4 nozzle",
        printer_variant: "0.4",
    };
    assert_eq!(
        raw_vendor("BBL").similar_printer_preset("Bambu Lab X1 Carbon", "", current),
        Some("Bambu Lab X1 Carbon 0.4 nozzle".to_owned())
    );
    let h = handle(&acme_vendor());
    assert_eq!(
        h.similar_printer_preset(
            "LH Stinger",
            "",
            CurrentPrinter {
                name: "Ghost 0.6",
                printer_variant: "0.6"
            }
        ),
        Some("LH Stinger 0.6 nozzle".to_owned())
    );
    // `:3751-3754`: an empty model falls back to the current preset's,
    // which an outside preset cannot supply.
    assert_eq!(
        h.similar_printer_preset(
            "",
            "",
            CurrentPrinter {
                name: "Ghost 0.6",
                printer_variant: "0.6"
            }
        ),
        None
    );
}
