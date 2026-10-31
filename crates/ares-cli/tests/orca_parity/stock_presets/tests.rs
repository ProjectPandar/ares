use super::{OrcaRunner, VendorProfiles, export};
use crate::runner::{application, repo_root};
use serde_json::json;

#[test]
fn stock_prusa_miniis_preserves_overhang_reverse_and_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "Prusa").unwrap();
    let selection = crate::select_printer(&profiles, "Prusa", "Prusa MINIIS 0.25 nozzle").unwrap();
    let exported = export(
        &runner,
        &profiles,
        &selection,
        &repo_root().join("tests/parity/cube10.stl"),
    )
    .unwrap();
    for (key, value) in [
        ("wall_generator", "arachne"),
        ("overhang_reverse", "1"),
        ("overhang_reverse_threshold", "50%"),
        ("detect_overhang_wall", "1"),
    ] {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    let outcome = crate::compare_exported(&runner, &exported);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn stock_kobra3_max_preserves_first_layer_bounds_and_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Anycubic",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Anycubic", "Anycubic Kobra 3 Max 0.4 nozzle").unwrap();
    let exported = export(
        &runner,
        &profiles,
        &selection,
        &repo_root().join("tests/parity/cube10.stl"),
    )
    .unwrap();
    application::verify_value(&exported.project, "skirt_loops", &json!("0")).unwrap();
    application::verify_value(&exported.project, "skirt_height", &json!("1")).unwrap();
    let outcome = crate::compare_exported(&runner, &exported);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn stock_ender3_preserves_thin_walls_and_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let process = profiles.process(&selection.process).unwrap();
    assert_eq!(process["detect_thin_wall"], json!("1"));
    let exported = export(
        &runner,
        &profiles,
        &selection,
        &repo_root().join("tests/parity/cube10.stl"),
    )
    .unwrap();
    application::verify_value(&exported.project, "detect_thin_wall", &json!("1")).unwrap();
    let outcome = crate::compare_exported(&runner, &exported);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
