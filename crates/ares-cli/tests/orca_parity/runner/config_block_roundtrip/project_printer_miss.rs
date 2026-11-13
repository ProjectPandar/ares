use super::super::{OrcaRunner, application, repo_root};
use super::project_catalog_supply::{native, produce};
use super::project_preset_refresh::replace_settings;
use super::project_process_miss::slice_matches_native;
use serde_json::json;

const UNLISTED: &str = "Bambu Lab H2D Unlisted 0.4 nozzle";

/// `load_project_preset` (`OrcaSlicer.cpp:3182-3185`): a printer whose system preset no vendor
/// lists keeps the project's values, the keys it lacks take the collection default, and the
/// process and filaments still take theirs from their system presets.
#[test]
fn project_printer_refresh_miss_keeps_project_values_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input =
        std::fs::read(repo_root().join("tests/parity/master-extruder-id/stock-right-only.3mf"))
            .unwrap();
    let mut settings = application::settings(&input).unwrap();
    assert_eq!(
        settings["printer_settings_id"],
        json!("Bambu Lab H2D 0.4 nozzle")
    );
    assert_eq!(settings["inherits_group"], json!(["", "", "", ""]));
    for key in [
        "top_color_penetration_layers",
        "extruder_clearance_dist_to_rod",
    ] {
        assert!(!settings.contains_key(key), "{key}");
    }
    assert_eq!(
        settings["print_compatible_printers"],
        json!(["Bambu Lab H2D 0.4 nozzle"])
    );
    settings.insert("printer_settings_id".to_owned(), json!(UNLISTED));
    // A reprocessed project's process must list its printer (OrcaSlicer.cpp:2955-2962).
    settings.insert(
        "print_compatible_printers".to_owned(),
        json!(["Bambu Lab H2D 0.4 nozzle", UNLISTED]),
    );
    // The legal variant: master extruder 1 rejects the TPU on the left nozzle.
    settings.insert("master_extruder_id".to_owned(), json!("2"));
    let input = replace_settings(&input, &settings);
    let project = runner.work.join("input.3mf");
    std::fs::write(&project, &input).unwrap();
    let supply = produce(&runner);
    let reference = native(&runner, &project, Some(&supply.join("native")), "native");
    let text = std::str::from_utf8(&reference).unwrap();
    // The printer takes the collection defaults (its system preset holds 50 and 1); the process
    // and filaments still take their system values.
    assert!(text.contains("; extruder_clearance_dist_to_rod = 40\n"));
    assert!(text.contains("; support_cooling_filter = 0\n"));
    assert!(text.contains("; top_color_penetration_layers = 5\n"));
    assert!(text.contains("; filament_pre_cooling_temperature = 210,0\n"));

    slice_matches_native(&runner, &input, &supply, &reference, "printer");
}
