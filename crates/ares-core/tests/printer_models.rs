//! `printer_models` grouping and `similar_printer_preset` resolution:
//! `PresetComboBoxes.cpp:1243-1251` (system models deduped by first
//! occurrence) and `PresetBundle.cpp:3750-3782` (model/variant resolution).

use ares_core::PresetCatalogHandle;
use serde_json::{Value, json};

fn vendor(name: &str, lists: Value, files: Value) -> Value {
    let mut index = json!({"name": name});
    for key in ["machine_list"] {
        if let Some(entries) = lists.get(key) {
            index[key] = entries.clone();
        }
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
        machine("stinger_04.json", "LH Stinger 0.4 nozzle", "LH Stinger", "0.4"),
        machine("stinger_06.json", "LH Stinger 0.6 nozzle", "LH Stinger", "0.6"),
        machine("stinger_mmu.json", "LH Stinger MMU 0.4 nozzle", "LH Stinger", "0.4"),
        machine("harrier_04.json", "Harrier 0.4 nozzle", "Harrier", "0.4"),
        machine("template.json", "fdm_acme_template", "ACME", ""),
    ]
    .into_iter()
    .collect::<serde_json::Map<_, _>>();
    let mut files = Value::Object(files);
    files["template.json"]["instantiation"] = json!("false");
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

#[test]
fn printer_models_dedupes_system_printers_by_first_occurrence() {
    // PresetComboBoxes.cpp:1243-1251: system printer presets display
    // `printer_model`; a set dedupes, so each model appears once in
    // collection order. Templates (instantiation "false") and missing-parent
    // printers never enter the listing.
    let mut h = handle(&acme_vendor());
    // Add an orphan machine to prove missing parents stay out.
    let orphan = json!({
        "type": "machine", "name": "Acme Orphan", "inherits": "fdm_missing",
        "instantiation": "true", "printer_model": "Orphan"
    });
    h.add_import(orphan.to_string().as_bytes(), "orphan.json").unwrap();
    assert_eq!(
        models(&h),
        vec![
            json!({"model": "LH Stinger"}),
            json!({"model": "Harrier"}),
        ]
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
    h.add_import(user.to_string().as_bytes(), "my.json").unwrap();
    assert_eq!(
        models(&h),
        vec![json!({"model": "LH Stinger"}), json!({"model": "Harrier"})]
    );
}

#[test]
fn similar_resolves_by_exact_alias_match() {
    // PresetBundle.cpp:3768-3776: prefer_printer = the current preset's
    // alias with the old variant replaced by the requested one.
    let h = handle(&stinger_04_with_alias());
    // With an explicit alias, the substituted alias matches the 0.6 preset's
    // own alias (which resolves to the bare model name).
    assert_eq!(
        h.similar_printer_preset("LH Stinger", "0.6", "My Stinger"),
        Some("LH Stinger 0.6 nozzle".to_owned())
    );
}

#[test]
fn similar_resolves_by_variant_then_first() {
    let h = handle(&acme_vendor());
    // No alias: prefer_printer substitution misses; the first preset whose
    // printer_variant matches wins (`:3777-3780`).
    assert_eq!(
        h.similar_printer_preset("LH Stinger", "0.6", "LH Stinger 0.4 nozzle"),
        Some("LH Stinger 0.6 nozzle".to_owned())
    );
    // No variant match either (MMU-only model): the first candidate in name
    // order wins (`:3781`).
    assert_eq!(
        h.similar_printer_preset("LH Stinger", "0.8", "LH Stinger 0.4 nozzle"),
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
    h.add_import(user.to_string().as_bytes(), "user_harrier.json").unwrap();
    assert_eq!(
        h.similar_printer_preset("Harrier", "", "Harrier 0.4 nozzle"),
        Some("Harrier 0.4 nozzle".to_owned())
    );
    assert_eq!(
        h.similar_printer_preset("No Such Model", "0.4", "Harrier 0.4 nozzle"),
        None
    );
}

#[test]
fn similar_missing_model_returns_none() {
    // `:3755-3756`: no candidates → nullptr upstream → None here.
    let h = handle(&acme_vendor());
    assert_eq!(h.similar_printer_preset("Ghost", "0.4", "LH Stinger 0.4 nozzle"), None);
}
