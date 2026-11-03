//! Beta PrintApply.cpp:1422-1430 selects stride-2 values before stride-1 metadata.
use crate::{
    golden,
    runner::native_stream_case::{check, qualify},
};
use serde_json::json;
use std::fs;

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/printer-variant-order/model-ironing-gyroid.3mf");
const SHA: &str = "a17ab11edca5ef6ec93aa202d87f75e1898526a604c954dd166b939cc94b0553";
const SPEED_E: &str = "30,30,120,120";
const UPSTREAM: &[&str] = &[
    "PrintApply.cpp:1422-1430::Print::apply",
    "PrintConfig.cpp:11661-11814::update_values_to_printer_extruders",
];

#[test]
fn printer_variant_order_machine_speed_e_matches_native() {
    let (reference, replay) = qualify(
        ("printer-variant-order-speed-e", UPSTREAM),
        "model-ironing-gyroid",
        PROJECT,
        SHA,
        false,
    );
    for native in [&reference, &replay] {
        assert_eq!(machine_speed_e(&native.native), SPEED_E);
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            PROJECT,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let path = reference.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&path)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(path).unwrap();
    let api_text = std::str::from_utf8(&api).unwrap();
    let cli_text = std::str::from_utf8(&cli_bytes).unwrap();
    fs::write(
        reference.root.join("focused-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": SHA,
            "native_speed_e": machine_speed_e(&reference.native),
            "api_speed_e": machine_speed_e(api_text),
            "cli_speed_e": machine_speed_e(cli_text),
            "cli_code": cli.status.code(),
            "api_cli_complete_bytes_equal": api == cli_bytes,
            "complete_api_comparison": golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(), &api,
            ),
            "complete_cli_comparison": golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(), &cli_bytes,
            ),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "CLI at {:?}", reference.root);
    assert!(api == cli_bytes, "complete API/CLI bytes differ");
    assert_eq!(
        machine_speed_e(api_text),
        machine_speed_e(&reference.native),
        "public API configured E-speed vector",
    );
    assert_eq!(
        machine_speed_e(cli_text),
        machine_speed_e(&reference.native),
        "public CLI configured E-speed vector",
    );
}

#[test]
fn printer_variant_order_matches_native_complete_output() {
    check(
        ("printer-variant-order-complete", UPSTREAM),
        "model-ironing-gyroid",
        PROJECT,
        SHA,
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert_eq!(machine_speed_e(&native.native), SPEED_E);
            }
        }),
    );
}

fn machine_speed_e(gcode: &str) -> &str {
    gcode
        .lines()
        .find_map(|line| line.strip_prefix("; machine_max_speed_e = "))
        .unwrap()
}
