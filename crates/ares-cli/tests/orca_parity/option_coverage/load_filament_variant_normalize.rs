//! PrintConfig.cpp:11431-11457: per-variant filament options are laid out to the
//! `filament_self_index` slots at 3MF load (PrintConfig.cpp:10094-10096) and after the CLI
//! defaults completion (OrcaSlicer.cpp:4250-4259).
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

const KEYS: [&str; 4] = [
    "activate_air_filtration",
    "during_print_exhaust_fan_speed",
    "nozzle_temperature",
    "complete_print_exhaust_fan_speed",
];

fn footer_lines(text: &str) -> Vec<&str> {
    KEYS.iter()
        .map(|key| {
            let prefix = format!("; {key} = ");
            let mut lines = text.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next().unwrap();
            assert!(lines.next().is_none(), "one footer line for {key}");
            line
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, self_index: &str, expected: [&str; 4]) {
    let (reference, _) = native_stream_case::qualify(
        (
            "load-filament-variant-normalize",
            &[
                "PrintConfig.cpp:11431-11457",
                "PrintConfig.cpp:10094-10096",
                "OrcaSlicer.cpp:4250-4259",
            ],
        ),
        label,
        input,
        sha,
        false,
    );
    assert!(
        reference
            .native
            .lines()
            .any(|line| line == format!("; filament_self_index = {self_index}")),
        "filament_self_index premise"
    );
    let native_section = footer_lines(&reference.native);
    let expected_section: Vec<_> = KEYS
        .iter()
        .zip(expected)
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
    let api_section = footer_lines(std::str::from_utf8(&api).unwrap());
    let cli_section = footer_lines(std::str::from_utf8(&cli_bytes).unwrap());
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

#[test]
fn ender3_four_slot_project_singletons_labelled_section() {
    check(
        "auto-brim-pla",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/auto-brim-object-adhesion/auto_brim-pla.3mf"
        )),
        "ec2304a186a04bc1dcd0a7ad2497003f0c209fafd78eb5a66c90f044fc9ddb31",
        "1,1,1,1",
        ["0,0,0,0", "60,60,60,60", "220,220,220,220", "80,80,80,80"],
    );
}

#[test]
fn ender3_four_slot_two_tools_labelled_section() {
    check(
        "tools2-minimum10",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/skirt-minlength-multitool/tools2-minimum10.3mf"
        )),
        "cd9460b1c11e7dff39c5f1f4ffb60e9558489efd1edf7e3abb90ea3ff2a7caba",
        "1,1,1,1",
        ["0,0,0,0", "60,60,60,60", "220,220,220,220", "80,80,80,80"],
    );
}

#[test]
fn mk4_two_slot_project_singletons_labelled_section() {
    check(
        "mk4base",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/mk4-layer-change/mk4base.3mf"
        )),
        "d0a4dfdb7f89120719b380a2995687a81a7e699025b8f3734349480bdebdc046",
        "1,1",
        ["0,0", "60,60", "220,220", "80,80"],
    );
}

#[test]
fn ender3_four_slot_absent_keys_take_laid_out_defaults_labelled_section() {
    check(
        "absent-variant-keys",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/load-filament-variant-normalize/absent-variant-keys.3mf"
        )),
        "a8e7658679fafb80e11a73ca7b5025519e2621ad97362e34a85282f5e6f203c5",
        "1,1,1,1",
        ["0,0,0,0", "60,60,60,60", "220,220,220,220", "80,80,80,80"],
    );
}
