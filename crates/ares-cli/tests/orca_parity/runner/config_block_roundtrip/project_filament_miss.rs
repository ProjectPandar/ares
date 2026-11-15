use super::super::{OrcaRunner, application, cache_identity, profile_entry, repo_root};
use super::project_catalog_supply::{native_with_args, produce};
use serde_json::json;

#[test]
fn project_filament_refresh_vendor_miss_keeps_imported_slot_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input =
        std::fs::read(repo_root().join("tests/parity/arachne-alternate-extra-wall/spiral-off.3mf"))
            .unwrap();
    assert_eq!(
        cache_identity::raw_digest(&input),
        "acac29bab7d4a4c56e34acd312b004aff8d0a8da0023247bae96964e68732ee5"
    );
    let settings = application::settings(&input).unwrap();
    assert_eq!(
        settings["filament_settings_id"],
        json!(["Creality Generic PLA"])
    );
    assert_eq!(settings["filament_flow_ratio"], json!(["0.98"]));
    assert_eq!(settings["filament_diameter"], json!(["1.75"]));
    let project = runner.work.join("input.3mf");
    std::fs::write(&project, &input).unwrap();
    let supply = produce(&runner);
    // 2.5.0 PrintConfig.cpp:13160-13165 defaults to error-only logging; utils.cpp:160-192
    // no longer forces info for release builds. Request warnings for OrcaSlicer.cpp:2806;
    // PresetBundle.cpp:480's vendor-miss error alone does not cover that CLI warning.
    let references = (0..2)
        .map(|index| {
            native_with_args(
                &runner,
                &project,
                Some(&supply.join("native")),
                &format!("native-{index}"),
                &["--debug", "2"],
            )
        })
        .collect::<Vec<_>>();
    for reference in &references {
        let text = std::str::from_utf8(reference).unwrap();
        assert!(text.contains("; filament_settings_id = \"Creality Generic PLA\"\n"));
        assert!(text.contains("; filament_flow_ratio = 0.98\n"));
        assert!(text.contains("; filament_diameter = 1.75\n"));
        assert!(!text.contains("; compatible_process_expression_group = "));
        assert!(!text.contains("; pellet_flow_coefficient = "));
    }
    let normalized = references
        .iter()
        .map(|bytes| {
            crate::golden::normalize_one_generator_line(bytes, crate::golden::GeneratorKind::Orca)
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(normalized[0], normalized[1], "Native repeat");
    let warning = "CLI: system filament preset 'Creality Generic PLA' not resolved (No vendor lists the preset); filament 1 keeps its values";
    let logs = std::fs::read_dir(&runner.work)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|suffix| suffix == "log")
        })
        .map(|entry| std::fs::read_to_string(entry.path()).unwrap())
        .collect::<Vec<_>>();
    assert!(
        logs.iter().any(|text| text.contains(warning)),
        "Native miss warning; {:?}",
        runner.work
    );
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
    let actual = result.unwrap_or_else(|error| {
        panic!(
            "public optional filament refresh: {error}; {:?}",
            runner.work
        )
    });
    std::fs::write(runner.work.join("ares.gcode"), &actual).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&references[0], &actual)
        .unwrap_or_else(|error| panic!("whole output: {error}; {:?}", runner.work));
}
