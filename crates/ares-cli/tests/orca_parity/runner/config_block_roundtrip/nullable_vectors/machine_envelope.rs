//! GCode.cpp:5407–5491: normal-pair limits over used physical extruders.

use crate::runner::{application, native_api_cli::sha, native_stream_case, repo_root};
use std::fs;

fn envelope(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with(';'))
        .skip_while(|line| !line.starts_with("M201 "))
        .take_while(|line| {
            matches!(
                line.split_ascii_whitespace().next(),
                Some("M201" | "M203" | "M204" | "M205" | "M566")
            )
        })
        .collect()
}

fn config_token<'a>(output: &'a str, key: &str) -> &'a str {
    let prefix = format!("; {key} = ");
    output
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .next_back()
        .unwrap()
}

fn check(physical: u8, acceleration: &str, speed: &str) {
    let source = fs::read(repo_root().join("tests/parity/nozzle-flush-dataset/D0.3mf")).unwrap();
    assert_eq!(
        sha(&source),
        "6d576ce9d4835dfae9ec5edacf9c215d7d6893ce097606f06f987db98261ff34"
    );
    let project = super::mapped_nozzle::mapped_project(&source, physical);
    let (reference, _) = native_stream_case::qualify(
        (
            "machine-envelope-used-extruders",
            &["GCode.cpp:5407-5491", "PrintConfig.cpp:10270-10287"],
        ),
        &format!("physical-{physical}"),
        &project,
        &sha(&project),
        false,
    );
    let native = envelope(&reference.native);
    assert!(native.contains(&acceleration));
    assert!(native.contains(&speed));
    assert_eq!(native.len(), 4);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "{cli:?}");
    let cli_bytes = fs::read(cli_path).unwrap();
    assert_eq!(api, cli_bytes, "API/CLI raw equality");
    let api_text = std::str::from_utf8(&api).unwrap();
    for key in ["filament_map", "filament_map_2", "filament_nozzle_map"] {
        assert_eq!(
            config_token(&reference.native, key),
            config_token(api_text, key),
            "{key}: witness must not depend on map repair"
        );
    }
    assert_eq!(
        native,
        envelope(api_text),
        "used-extruder machine envelope; artifacts {:?}",
        reference.root
    );
}

#[test]
fn physical_head_two_selects_used_normal_pair_limits() {
    check(
        2,
        "M201 X20000 Y20000 Z500 E5000",
        "M203 X1000 Y1000 Z20 E120",
    );
}

#[test]
fn physical_head_one_ignores_unused_head_limits() {
    check(
        1,
        "M201 X20000 Y20000 Z500 E30000",
        "M203 X1000 Y1000 Z20 E30",
    );
}
