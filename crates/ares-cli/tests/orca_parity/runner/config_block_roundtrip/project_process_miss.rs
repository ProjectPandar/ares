use super::super::{OrcaRunner, application, cache_identity, profile_entry, repo_root};
use super::project_catalog_supply::{native, produce};
use super::project_preset_refresh::replace_settings;
use serde_json::json;

const UNLISTED: &str = "0.20mm Unlisted @H2D";

/// `load_project_preset` (`OrcaSlicer.cpp:3182-3185`): a process whose system preset no vendor
/// lists keeps the project's values, the keys it lacks take the collection default, and the
/// printer and filaments still take theirs from their system presets.
#[test]
fn project_process_refresh_miss_keeps_project_values_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input =
        std::fs::read(repo_root().join("tests/parity/master-extruder-id/stock-right-only.3mf"))
            .unwrap();
    let mut settings = application::settings(&input).unwrap();
    assert_eq!(
        settings["print_settings_id"],
        json!("0.20mm Standard @BBL H2D")
    );
    assert_eq!(settings["inherits_group"], json!(["", "", "", ""]));
    for key in [
        "top_color_penetration_layers",
        "extruder_clearance_dist_to_rod",
    ] {
        assert!(!settings.contains_key(key), "{key}");
    }
    settings.insert("print_settings_id".to_owned(), json!(UNLISTED));
    // The legal variant: master extruder 1 rejects the TPU on the left nozzle.
    settings.insert("master_extruder_id".to_owned(), json!("2"));
    let input = replace_settings(&input, &settings);
    let project = runner.work.join("input.3mf");
    std::fs::write(&project, &input).unwrap();
    let supply = produce(&runner);
    let reference = native(&runner, &project, Some(&supply.join("native")), "native");
    let text = std::str::from_utf8(&reference).unwrap();
    // The process takes the collection default (its system preset holds 5); the printer and
    // filament still take their system values.
    assert!(text.contains("; top_color_penetration_layers = 0\n"));
    assert!(text.contains("; extruder_clearance_dist_to_rod = 50\n"));
    assert!(text.contains("; support_cooling_filter = 1\n"));

    let catalog_json = std::fs::read_to_string(supply.join("catalog.json")).unwrap();
    let selection_json = json!({"from_project": true}).to_string();
    let overrides = ares_core::ConfigOverrides::default();
    let result = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles_validated(
            &input,
            ares_core::ProfileSliceRequest {
                filename: "project.3mf",
                catalog_json: &catalog_json,
                selection_json: &selection_json,
                plate: None,
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ));
    profile_entry::write_json(
        &runner.work.join("public-result.json"),
        &match &result {
            Ok(bytes) => json!({"sha256": cache_identity::raw_digest(bytes)}),
            Err(error) => json!({"error": error.to_string()}),
        },
    );
    let actual = result
        .unwrap_or_else(|error| panic!("public process refresh miss: {error}; {:?}", runner.work));
    std::fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|error| panic!("whole output: {error}; {:?}", runner.work));
}
