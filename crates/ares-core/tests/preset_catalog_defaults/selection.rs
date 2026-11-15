//! Source-derived selection expectations: Preset.hpp:834-858 and
//! PresetBundle.cpp:8517-8673, with real public catalog queries.

use super::{import, machine, observe, values};
use ares_core::{PresetCatalogHandle, ProfileKind};
use serde_json::{Value, json};

fn select(
    catalog: &PresetCatalogHandle,
    (printer, process): (&str, Option<&str>),
    current: &[&str],
    installed: &[&str],
    artifact: &str,
) -> Vec<String> {
    let current: Vec<_> = current.iter().map(|name| (*name).to_owned()).collect();
    let installed: Vec<_> = installed.iter().map(|name| (*name).to_owned()).collect();
    let actual = catalog.select_compatible_filaments((printer, process), &current, &installed);
    observe(
        artifact,
        &json!({"printer": printer, "process": process, "current": current,
        "installed": installed, "actual": actual.as_ref().map_err(ToString::to_string)}),
    );
    actual.unwrap()
}

fn system_filament(catalog: &mut PresetCatalogHandle) {
    let vendor = json!({
        "id": "MaterialVendor", "index": {"name": "MaterialVendor",
            "filament_list": [{"sub_path": "material.json"}]},
        "files": {"material.json": {"type": "filament", "name": "Hidden PLA",
            "from": "system", "instantiation": "true", "filament_type": ["PLA"],
            "filament_diameter": ["1.75"]}}
    });
    catalog.add_vendor(vendor.to_string().as_bytes()).unwrap();
}

#[test]
fn source_defined_filament_fallback_resolves_in_empty_material_catalog() {
    let catalog = machine();
    assert_eq!(
        select(
            &catalog,
            ("Single", None),
            &["Absent"],
            &[],
            "empty-material-selection"
        ),
        ["Default Filament"]
    );
    assert_eq!(
        values(
            &catalog,
            ProfileKind::Filament,
            "Default Filament",
            &["filament_settings_id"],
            "selected-default-values"
        ),
        json!([{"filament_settings_id": [""]}])
    );
}

#[test]
fn source_defined_empty_initial_slots_choose_builtin_filament() {
    assert_eq!(
        select(
            &machine(),
            ("Single", None),
            &[],
            &[],
            "empty-initial-slots"
        ),
        ["Default Filament"]
    );
}

#[test]
fn source_defined_filament_fallback_does_not_choose_incompatible_first_record() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "process", "name": "Active", "layer_height": "0.2"}),
    );
    import(
        &mut catalog,
        json!({"type": "filament", "name": "First incompatible",
        "compatible_prints_condition": "layer_height > 0.3"}),
    );
    assert_eq!(
        select(
            &catalog,
            ("Single", Some("Active")),
            &["First incompatible"],
            &[],
            "process-incompatible-fallback"
        ),
        ["Default Filament"]
    );
}

#[test]
fn source_defined_default_resolves_when_only_system_candidate_is_hidden() {
    let mut catalog = machine();
    system_filament(&mut catalog);
    assert_eq!(
        select(
            &catalog,
            ("Single", None),
            &["Absent"],
            &[],
            "hidden-only-fallback"
        ),
        ["Default Filament"]
    );
    assert_eq!(
        values(
            &catalog,
            ProfileKind::Machine,
            "Default Printer",
            &["inherits"],
            "default-after-vendor"
        ),
        json!([{"inherits": ""}])
    );
}

#[test]
fn compatible_hidden_current_is_retained_independently_of_default_suppression() {
    let mut catalog = machine();
    system_filament(&mut catalog);
    assert_eq!(
        select(
            &catalog,
            ("Single", None),
            &["Hidden PLA"],
            &[],
            "hidden-current-retained"
        ),
        ["Hidden PLA"]
    );
}

#[test]
fn source_defined_explicit_current_default_survives_real_candidates() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "filament", "name": "Real PLA"}),
    );
    assert_eq!(
        select(
            &catalog,
            ("Single", None),
            &["Default Filament"],
            &[],
            "explicit-current-default"
        ),
        ["Default Filament"]
    );
}

#[test]
fn suppressed_default_does_not_steal_real_candidate_or_tie_order() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "filament", "name": "Z PLA", "filament_type": ["PLA"]}),
    );
    import(
        &mut catalog,
        json!({"type": "filament", "name": "A PLA", "filament_type": ["PLA"]}),
    );
    assert_eq!(
        select(
            &catalog,
            ("Single", None),
            &["Absent"],
            &[],
            "real-candidate-tie"
        ),
        ["A PLA"]
    );
}

#[test]
fn installed_system_candidate_still_wins_normal_ranking() {
    let mut catalog = machine();
    system_filament(&mut catalog);
    assert_eq!(
        select(
            &catalog,
            ("Single", None),
            &["Absent"],
            &["Hidden PLA"],
            "installed-system-winner"
        ),
        ["Hidden PLA"]
    );
}

#[test]
fn source_defined_no_process_winner_returns_resolvable_default_setting() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "process", "name": "Blocked", "compatible_printers_condition": "nozzle_diameter[0] > 1"}),
    );
    let actual = catalog.compatible_print("Single", Some("Blocked"));
    observe(
        "process-default-selection",
        &json!({"actual": actual.as_ref().map_err(ToString::to_string)}),
    );
    assert_eq!(actual.unwrap().as_deref(), Some("Default Setting"));
    assert_eq!(
        values(
            &catalog,
            ProfileKind::Process,
            "Default Setting",
            &["print_settings_id"],
            "selected-process-values"
        ),
        json!([{"print_settings_id": ""}])
    );
}

#[test]
fn suppressed_process_default_does_not_steal_real_winner() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "process", "name": "Real process", "layer_height": "0.2"}),
    );
    assert_eq!(
        catalog.compatible_print("Single", None).unwrap().as_deref(),
        Some("Real process")
    );
}

#[test]
fn source_defined_explicit_default_process_is_retained() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "process", "name": "Real process", "layer_height": "0.2"}),
    );
    let actual = catalog
        .compatible_print("Single", Some("Default Setting"))
        .unwrap();
    observe(
        "explicit-current-default-process",
        &json!({"actual": actual}),
    );
    assert_eq!(actual.as_deref(), Some("Default Setting"));
}

#[test]
fn source_defined_default_fallback_preserves_fixed_four_and_surplus_slots() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "machine", "name": "Fixed four", "printer_model": "Snapmaker U1",
        "printer_technology": "FFF", "nozzle_diameter": ["0.4", "0.4", "0.4", "0.4"], "single_extruder_multi_material": "0"}),
    );
    assert_eq!(
        select(
            &catalog,
            ("Fixed four", None),
            &["Absent"],
            &[],
            "fixed-four-defaults"
        ),
        vec!["Default Filament"; 4]
    );
    assert_eq!(
        select(
            &catalog,
            ("Fixed four", None),
            &["Absent"; 6],
            &[],
            "surplus-default-slots"
        ),
        vec!["Default Filament"; 6]
    );
}

#[test]
fn source_defined_semm_fallback_keeps_one_slot() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "machine", "name": "SEMM", "printer_technology": "FFF",
        "nozzle_diameter": ["0.4"], "single_extruder_multi_material": "1"}),
    );
    assert_eq!(
        select(&catalog, ("SEMM", None), &[], &[], "semm-default-slot"),
        ["Default Filament"]
    );
}

#[test]
fn unrelated_missing_parents_remain_strict_and_reported() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "filament", "name": "Orphan", "inherits": "Absent parent"}),
    );
    let filtered =
        serde_json::to_value(catalog.filter_compatible("Single", None).unwrap()).unwrap();
    observe("missing-parent-control", &filtered);
    assert_eq!(
        filtered["missing_parents"],
        json!([{"kind": "filament", "name": "Orphan", "parent": "Absent parent"}])
    );
    assert!(
        catalog
            .preset_values(ProfileKind::Filament, &["Orphan".to_owned()], &["inherits"])
            .is_err()
    );
    assert!(catalog.filter_compatible("Absent printer", None).is_err());
    assert!(
        catalog
            .filter_compatible("Single", Some("Absent process"))
            .is_err()
    );
}

#[test]
fn expression_failure_diagnostics_do_not_become_default_lookup_repairs() {
    let mut catalog = machine();
    import(
        &mut catalog,
        json!({"type": "filament", "name": "Invalid expression", "compatible_printers_condition": "("}),
    );
    let actual: Value =
        serde_json::to_value(catalog.filter_compatible("Single", None).unwrap()).unwrap();
    observe("expression-diagnostic-control", &actual);
    assert!(
        actual["filaments"]
            .as_array()
            .unwrap()
            .contains(&json!("Invalid expression"))
    );
    assert_eq!(actual["diagnostics"][0]["name"], "Invalid expression");
    assert_eq!(actual["diagnostics"].as_array().unwrap().len(), 1);
}
