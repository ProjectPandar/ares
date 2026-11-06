//! OrcaSlicer.cpp:2787-2791,3606,4250-4259: original keys precede full defaults.
//! PrintConfig.cpp:8928-8937 and Preset.cpp:325 distinguish NC defaults from nil.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

const KEYS: &[&str] = &[
    "filament_preheat_temperature_delta",
    "filament_ramming_travel_time",
    "filament_ramming_travel_time_nc",
    "filament_ramming_volumetric_speed",
    "filament_ramming_volumetric_speed_nc",
    "filament_retract_length_nc",
];

fn cohort(text: &str) -> Vec<Option<&str>> {
    KEYS.iter()
        .map(|key| {
            let prefix = format!("; {key} = ");
            let mut lines = text.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next();
            assert!(lines.next().is_none(), "one footer line for {key}");
            line
        })
        .collect()
}

#[test]
fn x1c_included_preheat_and_nc_retract() {
    let input = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/parity/avoid-crossing-external-frame/x1c-detour0.3mf"
    ));
    let sha = "f44f09c6b26391787d5877cfafd094bb50cb452d727298d46cd6ab5d45ffa04e";
    let (reference, _) = native_stream_case::qualify(
        (
            "project-key-presence-late-defaults",
            &[
                "OrcaSlicer.cpp:2787-2791,3606,4250-4259",
                "PrintConfig.cpp:8928-8937",
                "Preset.cpp:325",
                "PresetBundle.cpp:7630-7649",
            ],
        ),
        "x1c-preheat-and-nc",
        input,
        sha,
        false,
    );
    for line in [
        "; filament_extruder_variant = \"Direct Drive Standard\"",
        "; filament_self_index = 1",
    ] {
        assert!(reference.native.lines().any(|actual| actual == line));
    }
    let expected = [
        Some("; filament_preheat_temperature_delta = 10"),
        Some("; filament_ramming_travel_time = 0"),
        Some("; filament_ramming_travel_time_nc = 0"),
        Some("; filament_ramming_volumetric_speed = -1"),
        Some("; filament_ramming_volumetric_speed_nc = -1"),
        Some("; filament_retract_length_nc = 14"),
    ];
    let native_section = cohort(&reference.native);
    assert_eq!(native_section, expected, "Native included-template cohort");
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
    }
    let path = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&path);
    let whole_api = api
        .as_ref()
        .map_err(|error| format!("API: {error:?}"))
        .and_then(|bytes| {
            golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), bytes)
        });
    let whole_cli = cli_bytes
        .as_ref()
        .map_err(|error| format!("CLI: {error}"))
        .and_then(|bytes| {
            golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), bytes)
        });
    fs::write(
        reference.root.join("strict-whole-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "api": whole_api, "cli": whole_cli, "cli_exit": cli.status.code(),
            "input_sha256": sha, "whole_output_parity_claimed": false,
        }))
        .unwrap(),
    )
    .unwrap();
    let api = api.unwrap();
    assert!(cli.status.success());
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI equality");
    let api_section = cohort(std::str::from_utf8(&api).unwrap());
    let cli_section = cohort(std::str::from_utf8(&cli_bytes).unwrap());
    fs::write(
        reference.root.join("labelled-section-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "native_section": native_section, "api_section": api_section, "cli_section": cli_section,
            "section_matches": native_section == api_section && native_section == cli_section,
            "input_sha256": sha, "api_cli_raw_equal": api == cli_bytes,
            "whole_output_parity_claimed": false,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        native_section, api_section,
        "labelled API late-default cohort"
    );
    assert_eq!(
        native_section, cli_section,
        "labelled CLI late-default cohort"
    );
}
