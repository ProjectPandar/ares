//! Labelled footer comparison for the filament variant layout and selection slices.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

type FooterLines<'a> = &'a [(&'a str, &'a str)];

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

pub(super) fn check(
    case: (&str, &[&str]),
    label: &str,
    input: &[u8],
    sha: &str,
    (premise, expected): (FooterLines<'_>, FooterLines<'_>),
) {
    let (reference, _) = native_stream_case::qualify(case, label, input, sha, false);
    for (key, value) in premise {
        assert!(
            reference
                .native
                .lines()
                .any(|line| line == format!("; {key} = {value}")),
            "{key} premise"
        );
    }
    let keys: Vec<_> = expected.iter().map(|(key, _)| *key).collect();
    let native_section = footer_lines(&reference.native, &keys);
    let expected_section: Vec<_> = expected
        .iter()
        .map(|(key, value)| format!("; {key} = {value}"))
        .collect();
    assert_eq!(native_section, expected_section, "Native variant layout");
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
    let api_section = footer_lines(std::str::from_utf8(&api).unwrap(), &keys);
    let cli_section = footer_lines(std::str::from_utf8(&cli_bytes).unwrap(), &keys);
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
    assert_eq!(native_section, api_section, "labelled API variant layout");
    assert_eq!(native_section, cli_section, "labelled CLI variant layout");
}
