//! PrintConfig.cpp:10153-10161: six nullable float filament variant members.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

const KEYS: [&str; 6] = [
    "filament_preheat_temperature_delta",
    "filament_ramming_travel_time",
    "filament_ramming_travel_time_nc",
    "filament_ramming_volumetric_speed",
    "filament_ramming_volumetric_speed_nc",
    "filament_retract_length_nc",
];
const SELECTED: &[&str] = &[
    "; filament_self_index = 1,2",
    "; nozzle_volume_type = High Flow",
    "; nozzle_temperature_initial_layer = 217,237",
];

fn cohort(text: &str) -> Vec<Option<&str>> {
    KEYS.iter()
        .map(|key| {
            let prefix = format!("; {key} = ");
            let mut lines = text.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next();
            assert!(lines.next().is_none(), "at most one footer line for {key}");
            line
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, premise: &[&str], expected: &[Option<&str>]) {
    let (reference, _) = native_stream_case::qualify(
        (
            "remaining-filament-variants",
            &[
                "PrintConfig.cpp:8869-8937,8971-8977,10153-10161",
                "PrintConfig.cpp:11431-11457,11862-11994",
                "PrintConfig.hpp:1925,1927-1931,2129",
            ],
        ),
        label,
        input,
        sha,
        false,
    );
    for line in premise {
        assert!(
            reference.native.lines().any(|actual| actual == *line),
            "Native layout premise: {line}"
        );
    }
    let native_section = cohort(&reference.native);
    assert_eq!(native_section, expected, "Native six-key cohort");
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
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|b| golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), b));
    let whole_cli = cli_bytes
        .as_ref()
        .map_err(|e| format!("CLI: {e}"))
        .and_then(|b| golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), b));
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
    fs::write(reference.root.join("labelled-section-comparison.json"), serde_json::to_vec_pretty(&json!({
        "native_section": native_section, "api_section": api_section, "cli_section": cli_section,
        "section_matches": native_section == api_section && native_section == cli_section,
        "native_premise": premise, "input_sha256": sha, "api_cli_raw_equal": api == cli_bytes,
        "whole_output_parity_claimed": false,
    })).unwrap()).unwrap();
    assert_eq!(native_section, api_section, "labelled API six-key cohort");
    assert_eq!(native_section, cli_section, "labelled CLI six-key cohort");
}

#[test]
fn ender_four_slot_defaults() {
    check(
        "ender-defaults",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/auto-brim-object-adhesion/auto_brim-pla.3mf"
        )),
        "ec2304a186a04bc1dcd0a7ad2497003f0c209fafd78eb5a66c90f044fc9ddb31",
        &["; filament_self_index = 1,1,1,1"],
        &[
            Some("; filament_preheat_temperature_delta = 0,0,0,0"),
            Some("; filament_ramming_travel_time = 0,0,0,0"),
            Some("; filament_ramming_travel_time_nc = 0,0,0,0"),
            Some("; filament_ramming_volumetric_speed = -1,-1,-1,-1"),
            Some("; filament_ramming_volumetric_speed_nc = -1,-1,-1,-1"),
            Some("; filament_retract_length_nc = 10,10,10,10"),
        ],
    );
}

#[test]
fn mk4_two_slot_defaults() {
    check(
        "mk4-defaults",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/mk4-layer-change/mk4base.3mf"
        )),
        "d0a4dfdb7f89120719b380a2995687a81a7e699025b8f3734349480bdebdc046",
        &["; filament_self_index = 1,1"],
        &[
            Some("; filament_preheat_temperature_delta = 0,0"),
            Some("; filament_ramming_travel_time = 0,0"),
            Some("; filament_ramming_travel_time_nc = 0,0"),
            Some("; filament_ramming_volumetric_speed = -1,-1"),
            Some("; filament_ramming_volumetric_speed_nc = -1,-1"),
            Some("; filament_retract_length_nc = 10,10"),
        ],
    );
}

#[test]
fn distinct_source_columns() {
    check(
        "distinct",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/remaining-filament-variants/distinct.3mf"
        )),
        "6160c4189456184da68af273937e448f210a06719ade2059cb92f4e7da133aeb",
        SELECTED,
        &[
            Some("; filament_preheat_temperature_delta = -2.5,-4.5"),
            Some("; filament_ramming_travel_time = 2.5,4.5"),
            Some("; filament_ramming_travel_time_nc = 6.5,8.5"),
            Some("; filament_ramming_volumetric_speed = 22.5,24.5"),
            Some("; filament_ramming_volumetric_speed_nc = 32.5,34.5"),
            Some("; filament_retract_length_nc = 12.5,14.5"),
        ],
    );
}

#[test]
fn mixed_nil_and_defined() {
    check(
        "mixed",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/remaining-filament-variants/mixed.3mf"
        )),
        "9749d50fb885d3c01ac6e65d5756d462718f2762152402d30c682389670fbef0",
        SELECTED,
        &[
            Some("; filament_preheat_temperature_delta = 0,nil"),
            Some("; filament_ramming_travel_time = nil,0"),
            Some("; filament_ramming_travel_time_nc = 0,nil"),
            Some("; filament_ramming_volumetric_speed = nil,-1"),
            Some("; filament_ramming_volumetric_speed_nc = -1,nil"),
            Some("; filament_retract_length_nc = nil,0"),
        ],
    );
}

#[test]
fn all_nil_omission() {
    check(
        "all-nil",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/remaining-filament-variants/all-nil.3mf"
        )),
        "2c8613b2e60e14dc4bb2afc84059d2d1c52d17c1c9ce04d8d13d91099c8b27d0",
        SELECTED,
        &[
            Some("; filament_preheat_temperature_delta = 0,0"),
            None,
            None,
            None,
            None,
            None,
        ],
    );
}
