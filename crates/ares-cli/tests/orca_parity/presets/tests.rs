use super::VendorProfiles;
use crate::{runner, select_printer, smoke_case_overrides, smoke_overrides};

fn profiles(vendor: &str) -> VendorProfiles {
    VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        vendor,
    )
    .unwrap()
}

#[test]
fn relative_e_profile_without_reset_gets_layer_reset() {
    let machine = serde_json::from_value(serde_json::json!({
        "use_relative_e_distances": "1"
    }))
    .unwrap();
    let process = serde_json::from_value(serde_json::json!({
        "before_layer_change_gcode": ";before"
    }))
    .unwrap();

    let overrides = smoke_case_overrides(&machine, &process);

    assert_eq!(
        overrides
            .get("before_layer_change_gcode")
            .and_then(serde_json::Value::as_str),
        Some(";before\nG92 E0")
    );
}

#[test]
fn smoke_overrides_clear_cli_unsafe_bed_exclusion() {
    assert_eq!(
        smoke_overrides().get("bed_exclude_area"),
        Some(&serde_json::json!(["0x0"]))
    );
}

#[test]
fn sweep_lists_instantiated_printers_only() {
    let names = profiles("Afinia").instantiated_machine_names();

    assert!(names.iter().any(|name| name == "Afinia H+1(HS) 0.4 nozzle"));
    assert!(!names.iter().any(|name| name == "Afinia H+1(HS)"));
    assert!(!names.iter().any(|name| name == "fdm_afinia_common"));
}

#[test]
fn instantiated_printer_names_remain_sorted() {
    let names = profiles("Creality").instantiated_machine_names();
    let mut sorted = names.clone();
    sorted.sort();

    assert_eq!(names, sorted);
}

#[test]
fn selection_falls_back_to_an_explicitly_compatible_filament() {
    let profiles = profiles("Afinia");
    let selection = select_printer(&profiles, "Afinia", "Afinia H+1(HS) 0.4 nozzle").unwrap();

    assert_eq!(selection.process, "0.20mm Standard @Afinia H+1(HS)");
    assert!(selection.filaments[0].contains("PLA"));
}

#[test]
fn smoke_selects_one_filament_for_multi_extruder_printers() {
    let profiles = profiles("Snapmaker");
    let selection =
        select_printer(&profiles, "Snapmaker", "Snapmaker A250 Dual (0.4 nozzle)").unwrap();

    assert_eq!(selection.filaments.len(), 1);
}

#[test]
fn incompatible_named_default_falls_back_to_explicit_compatible_process() {
    let profiles = profiles("Artillery");
    let selection = select_printer(&profiles, "Artillery", "Artillery M1 Pro 0.2 nozzle").unwrap();

    assert!(selection.process.contains("M1 Pro 0.2 nozzle"));
    assert!(profiles.process_is_compatible(&selection.process, &selection.printer));
}

#[test]
fn selection_falls_back_to_a_compatible_process() {
    let profiles = profiles("Anker");
    let selection = select_printer(&profiles, "Anker", "Anker M5 0.2 nozzle").unwrap();

    assert!(selection.process.contains("0.2 nozzle @Anker"));
}

#[test]
fn vendor_without_filament_directory_uses_system_library() {
    let profiles = profiles("Voron");
    let selection = select_printer(&profiles, "Voron", "Voron 0.1 0.4 nozzle").unwrap();

    assert_eq!(selection.filaments, ["Generic PLA @System"]);
    assert!(profiles.filament("Generic PLA @System").is_ok());
}

#[test]
fn missing_local_filament_parent_uses_orca_defaults() {
    let profiles = profiles("Z-Bolt");
    let flattened = profiles.filament("Generic PLA @Z-Bolt 0.4 nozzle").unwrap();

    assert!(!flattened.contains_key("inherits"));
    assert_eq!(
        flattened.get("name").and_then(serde_json::Value::as_str),
        Some("Generic PLA @Z-Bolt 0.4 nozzle")
    );
}
