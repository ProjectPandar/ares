//! PrintConfig.cpp:10236–10266 and 11661–11883: printer variant-one rates.

use crate::runner::{application, native_stream_case};
use std::fs;

fn rate_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| {
            line.starts_with("; hotend_cooling_rate = ")
                || line.starts_with("; hotend_heating_rate = ")
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str) {
    let (reference, _) = native_stream_case::qualify(
        (
            "hotend-rate-variant",
            &["PrintConfig.cpp:10236-10266", "PrintConfig.cpp:11661-11883"],
        ),
        label,
        input,
        sha,
        false,
    );
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
    assert!(cli.status.success());
    let cli_bytes = fs::read(&cli_path).unwrap();
    fs::write(reference.root.join("ares.cli.gcode"), &cli_bytes).unwrap();
    assert_eq!(api, cli_bytes, "API/CLI raw equality");
    let native = rate_lines(&reference.native);
    let api = rate_lines(std::str::from_utf8(&api).unwrap());
    assert_eq!(native, api, "Native/Ares printer variant-one rate values");
}

#[test]
fn all_nil_rates_preserve_native_variant_one_values() {
    check(
        "all-nil",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/hotend-rate-variant/all-nil.3mf"
        )),
        "5359c330b164846548dc9d7fa1f33b519675d88c663f9de64f969d55e08de107",
    );
}

#[test]
fn distinct_rates_preserve_native_variant_one_values() {
    check(
        "distinct",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/hotend-rate-variant/distinct.3mf"
        )),
        "e1bf499c5e0e4a7560653321027f99a3255744ce566b42ffdc142572e872522f",
    );
}

#[test]
fn mixed_rates_preserve_native_variant_one_values() {
    check(
        "mixed",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/hotend-rate-variant/mixed.3mf"
        )),
        "651f7db71a1368265e7d2e10dc5fa2500e98f286a1d9f6cebd50e95d96741677",
    );
}

#[test]
fn retained_b33_witness_preserves_native_variant_one_values() {
    check(
        "witness",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/hotend-rate-variant/witness.3mf"
        )),
        "12fb1bea5dea710405d8e620fb916dcaefa239b693d985da8c71100413aafd3d",
    );
}
