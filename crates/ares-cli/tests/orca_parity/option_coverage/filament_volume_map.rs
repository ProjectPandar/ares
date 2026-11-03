//! PrintConfig.cpp:3118; ToolOrdering.cpp:1752–1776,1892–1908,3054–3084; Print.cpp:4656–4745.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

const MAP_LINES: [(&str, &str); 3] = [
    ("; filament_map = ", "1,1"),
    ("; filament_volume_map = ", "0,0"),
    ("; filament_nozzle_map = ", "0,0"),
];

fn map_lines(text: &str) -> Vec<&str> {
    MAP_LINES
        .iter()
        .map(|&(prefix, _)| {
            let mut lines = text.lines().filter(|line| line.starts_with(prefix));
            let line = lines.next().unwrap();
            assert!(lines.next().is_none(), "one complete map line per key");
            line
        })
        .collect()
}

pub(crate) fn check_labelled(
    label: &str,
    input: &[u8],
    expected_sha: &str,
    expected: [(&str, &str); 3],
) {
    let (reference, _) = native_stream_case::qualify(
        (
            "filament-volume-map",
            &[
                "PrintConfig.cpp:3118",
                "ToolOrdering.cpp:1752-1776,1892-1908,3054-3084",
                "MultiNozzleUtils.cpp:316-369; FilamentGroupUtils.cpp:346-358",
                "Print.cpp:4656-4745; GCode.cpp:8338-8351",
            ],
        ),
        label,
        input,
        expected_sha,
        false,
    );
    let native_section = map_lines(&reference.native);
    let expected_section: Vec<_> = expected
        .iter()
        .map(|(prefix, value)| format!("{prefix}{value}"))
        .collect();
    assert_eq!(
        native_section, expected_section,
        "Native static map triplet"
    );
    let activity = crate::paint_color_activity::activity(reference.native.as_bytes());
    let model = activity["positive_model_XYE_by_tool"].as_object().unwrap();
    assert_eq!(model.len(), 1);
    assert!(model["1"].as_u64().unwrap() > 0);

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
            "input_sha256": expected_sha,
            "api": whole_api,
            "cli": whole_cli,
            "cli_exit": cli.status.code(),
            "whole_output_parity_claimed": false,
            "oracle": "STATIC_FILAMENT_MAP_TRIPLET"
        }))
        .unwrap(),
    )
    .unwrap();
    let api = api.unwrap();
    assert!(cli.status.success());
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI equality");
    let api_section = map_lines(std::str::from_utf8(&api).unwrap());
    let cli_section = map_lines(std::str::from_utf8(&cli_bytes).unwrap());
    fs::write(
        reference.root.join("labelled-section-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "oracle": "STATIC_FILAMENT_MAP_TRIPLET",
            "whole_output_parity_claimed": false,
            "native_section": native_section,
            "api_section": api_section,
            "cli_section": cli_section,
            "section_matches": native_section == api_section && native_section == cli_section,
            "native_activity": activity,
            "input_sha256": expected_sha,
            "api_cli_raw_equal": api == cli_bytes
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_section, api_section, "labelled static map triplet");
    assert_eq!(
        native_section, cli_section,
        "labelled CLI static map triplet"
    );
}

#[test]
fn volume_map_nonzero_initial_tool_labelled_section() {
    check_labelled(
        "nonzero-initial-tool",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/nonzero-initial-tool.3mf"
        )),
        "44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a",
        MAP_LINES,
    );
}

#[test]
fn volume_map_unused_hotter_filament_labelled_section() {
    check_labelled(
        "unused-hotter-filament",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/unused-hotter-filament.3mf"
        )),
        "893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0",
        MAP_LINES,
    );
}
