use super::VendorProfiles;
use crate::runner;

fn profiles(vendor: &str) -> VendorProfiles {
    VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        vendor,
    )
    .unwrap()
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
