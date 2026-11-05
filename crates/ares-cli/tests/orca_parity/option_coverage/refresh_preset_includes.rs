//! PresetBundle.cpp:7627-7640: the refreshed system filament carries its includes;
//! PrintConfig.cpp:11986-11992: the variant gather remaps `filament_self_index`.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

fn footer_lines<'a>(text: &'a str, keys: &[&str]) -> Vec<&'a str> {
    keys.iter()
        .map(|key| {
            let prefix = format!("; {key} = ");
            let mut lines = text.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next().unwrap();
            assert!(lines.next().is_none(), "one footer line for {key}");
            line
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, keys: &[&str], expected: &[&str]) {
    let (reference, _) = native_stream_case::qualify(
        (
            "refresh-preset-includes",
            &[
                "PresetBundle.cpp:7627-7640",
                "OrcaSlicer.cpp:2800-2807",
                "PrintApply.cpp:1436-1455",
                "PrintConfig.cpp:11986-11992",
            ],
        ),
        label,
        input,
        sha,
        false,
    );
    let native_section = footer_lines(&reference.native, keys);
    assert_eq!(native_section, expected, "Native refreshed filament footer");
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
            "input_sha256": sha,
            "api": whole_api,
            "cli": whole_cli,
            "cli_exit": cli.status.code(),
            "whole_output_parity_claimed": false,
        }))
        .unwrap(),
    )
    .unwrap();
    let api = api.unwrap();
    assert!(cli.status.success());
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI equality");
    let api_section = footer_lines(std::str::from_utf8(&api).unwrap(), keys);
    let cli_section = footer_lines(std::str::from_utf8(&cli_bytes).unwrap(), keys);
    fs::write(
        reference.root.join("labelled-section-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "native_section": native_section,
            "api_section": api_section,
            "cli_section": cli_section,
            "section_matches": native_section == api_section && native_section == cli_section,
            "input_sha256": sha,
            "api_cli_raw_equal": api == cli_bytes,
            "whole_output_parity_claimed": false,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        native_section, api_section,
        "labelled API refreshed filament footer"
    );
    assert_eq!(
        native_section, cli_section,
        "labelled CLI refreshed filament footer"
    );
}

#[test]
fn x1c_included_dual_variant_template_labelled_section() {
    check(
        "x1c-detour0",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/avoid-crossing-external-frame/x1c-detour0.3mf"
        )),
        "f44f09c6b26391787d5877cfafd094bb50cb452d727298d46cd6ab5d45ffa04e",
        &[
            "filament_extruder_variant",
            "filament_preheat_temperature_delta",
            "filament_self_index",
        ],
        &[
            "; filament_extruder_variant = \"Direct Drive Standard\"",
            "; filament_preheat_temperature_delta = 10",
            "; filament_self_index = 1",
        ],
    );
}

#[test]
fn ender3_two_variant_filaments_self_index_labelled_section() {
    check(
        "l59",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/l59.3mf"
        )),
        "908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d",
        &["filament_self_index"],
        &["; filament_self_index = 1,2"],
    );
}
