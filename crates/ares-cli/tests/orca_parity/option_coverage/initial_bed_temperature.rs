//! GCode.cpp:3990–4005,5497–5513,5518–5545: labelled start-temperature oracle.
use crate::{
    golden,
    runner::{application, native_api_cli::sha, native_stream_case},
};
use serde_json::json;
use std::fs;

// This section oracle does not assert whole-output parity.
fn start_through_temperature(text: &str) -> &str {
    let start = text.find("; EXECUTABLE_BLOCK_START\n").unwrap();
    let mut end = start;
    for line in text[start..].split_inclusive('\n') {
        end += line.len();
        if line.starts_with("M140 ") || line.starts_with("M190 ") {
            return &text[start..end];
        }
    }
    panic!("missing first M140/M190 in start block");
}

fn rendered_scalar(text: &str, command: &str) -> String {
    text.lines()
        .find_map(|line| {
            line.strip_prefix(command).and_then(|tail| {
                tail.split_ascii_whitespace()
                    .find_map(|word| word.strip_prefix('S'))
            })
        })
        .unwrap()
        .to_owned()
}

fn check(label: &str, input: &[u8], expected_sha: &str, temperature: &str) {
    assert_eq!(sha(input), expected_sha);
    let raw = application::settings(input).unwrap();
    let template = raw["machine_start_gcode"].as_str().unwrap();
    for command in ["M140", "M190"] {
        assert!(template.contains(&format!(
            "{command} S[bed_temperature_initial_layer_single]"
        )));
    }
    let (reference, _) = native_stream_case::qualify(
        (
            "initial-bed-temperature",
            &["GCode.cpp:3990-4005,5497-5513,5518-5545"],
        ),
        label,
        input,
        expected_sha,
        false,
    );
    let activity = crate::paint_color_activity::activity(reference.native.as_bytes());
    let model = activity["positive_model_XYE_by_tool"].as_object().unwrap();
    assert_eq!(model.len(), 1);
    assert!(
        model["1"].as_u64().unwrap() > 0,
        "Native must actually print only filament1"
    );
    for command in ["M140 ", "M190 "] {
        assert_eq!(rendered_scalar(&reference.native, command), temperature);
    }
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
    // Preserve the strict whole-output first difference; the assertion is scoped.
    fs::write(reference.root.join("strict-whole-comparison.json"), serde_json::to_vec_pretty(&json!({
        "input_sha256":expected_sha,"api":whole_api,"cli":whole_cli,"cli_exit":cli.status.code(),
        "whole_output_parity_claimed":false,"oracle":"LABELLED_START_THROUGH_FIRST_M140_M190_PLUS_RENDERED_SINGLE_PLACEHOLDER"
    })).unwrap()).unwrap();
    let api = api.unwrap();
    assert!(cli.status.success());
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI equality");
    let api_text = std::str::from_utf8(&api).unwrap();
    let native_section = start_through_temperature(&reference.native);
    let api_section = start_through_temperature(api_text);
    let native_scalar = [
        rendered_scalar(&reference.native, "M140 "),
        rendered_scalar(&reference.native, "M190 "),
    ];
    let api_scalar = [
        rendered_scalar(api_text, "M140 "),
        rendered_scalar(api_text, "M190 "),
    ];
    fs::write(reference.root.join("labelled-section-comparison.json"), serde_json::to_vec_pretty(&json!({
        "oracle":"LABELLED_START_THROUGH_FIRST_M140_M190_PLUS_RENDERED_SINGLE_PLACEHOLDER",
        "whole_output_parity_claimed":false,"native_start":native_section,"api_start":api_section,
        "placeholder":"bed_temperature_initial_layer_single","native_rendered_scalar":native_scalar,
        "api_rendered_scalar":api_scalar,"native_activity":activity,
        "section_matches":native_section==api_section,"placeholder_matches":native_scalar==api_scalar
    })).unwrap()).unwrap();
    assert_eq!(
        native_section, api_section,
        "labelled start-section oracle, not whole output"
    );
    assert_eq!(
        native_scalar, api_scalar,
        "labelled rendered scalar placeholder"
    );
}

#[test]
fn initial_bed_nonzero_filament_labelled_start() {
    check(
        "nonzero-initial-tool",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/nonzero-initial-tool.3mf"
        )),
        "44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a",
        "73",
    );
}

#[test]
fn initial_bed_unused_hotter_filament_labelled_start() {
    check(
        "unused-hotter-filament",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/m140-initial-bed/unused-hotter-filament.3mf"
        )),
        "893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0",
        "63",
    );
}
