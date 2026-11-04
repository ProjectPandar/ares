//! PrintConfig.cpp:11862–11882: shared gather preserves the front payload.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

fn initial_retract(text: &str) -> Vec<&str> {
    let (_, layer) = text.split_once(";BEFORE_LAYER_CHANGE\n").unwrap();
    let (before_fan, _) = layer.split_once(";_SET_FAN_SPEED_CHANGING_LAYER").unwrap();
    before_fan
        .lines()
        .filter(|line| line.starts_with("G1 E"))
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, expected: &[&str]) {
    let (reference, _) = native_stream_case::qualify(
        (
            "variant-retraction",
            &[
                "PrintConfig.cpp:11862-11882,11971-11977",
                "Config.hpp:767-815",
                "GCodeWriter.cpp:1457-1503",
            ],
        ),
        label,
        input,
        sha,
        false,
    );
    for line in [
        "; nozzle_temperature_initial_layer = 217,237",
        "; filament_self_index = 1,2",
        "; nozzle_volume_type = High Flow",
    ] {
        assert!(
            reference.native.lines().any(|actual| actual == line),
            "variant premise: {line}"
        );
    }
    let native_section = initial_retract(&reference.native);
    assert_eq!(native_section, expected, "Native initial retract");
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
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|b| golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), b));
    let whole_cli = cli_bytes
        .as_ref()
        .map_err(|e| format!("CLI: {e}"))
        .and_then(|b| golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), b));
    fs::write(reference.root.join("strict-whole-comparison.json"), serde_json::to_vec_pretty(&json!({"api": whole_api,"cli": whole_cli,"cli_exit": cli.status.code(),"input_sha256": sha,"whole_output_parity_claimed": false})).unwrap()).unwrap();
    let api = api.unwrap();
    assert!(cli.status.success());
    let cli_bytes = cli_bytes.unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI equality");
    let api_section = initial_retract(std::str::from_utf8(&api).unwrap());
    let cli_section = initial_retract(std::str::from_utf8(&cli_bytes).unwrap());
    fs::write(reference.root.join("labelled-section-comparison.json"),serde_json::to_vec_pretty(&json!({"native_section":native_section,"api_section":api_section,"cli_section":cli_section,"section_matches":native_section==api_section && native_section==cli_section,"native_activity":activity,"input_sha256":sha,"api_cli_raw_equal":api==cli_bytes,"whole_output_parity_claimed":false})).unwrap()).unwrap();
    assert_eq!(native_section, api_section, "labelled API initial retract");
    assert_eq!(native_section, cli_section, "labelled CLI initial retract");
}

#[test]
fn variant_retraction_l59() {
    check(
        "l59",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/l59.3mf"
        )),
        "908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d",
        &["G1 E-4 F3600"],
    );
}
#[test]
fn variant_retraction_nil() {
    check(
        "nil",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/nil.3mf"
        )),
        "05e95fd83ee91714eef81b863e07ca73514338a7f7a3b07ffb4114a4534ab299",
        &["G1 E-4 F3600"],
    );
}
#[test]
fn variant_retraction_zero() {
    check(
        "zero",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/zero.3mf"
        )),
        "3b9ee30c63a85c668fc81091425ed81975de711cc83b87bb69a19b673648dc0a",
        &[],
    );
}
#[test]
fn variant_retraction_nonzero() {
    check(
        "nonzero",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/nonzero.3mf"
        )),
        "8b54e454012680741b3885ff919e8e15e44c152385f578ca2b4a6d73f1342898",
        &["G1 E-2.5 F2220"],
    );
}
