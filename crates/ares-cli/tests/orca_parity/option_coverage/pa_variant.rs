//! PrintConfig.cpp:10199–10205: six PA fields share filament variant gathering.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

const KEYS: [&str; 6] = [
    "enable_pressure_advance",
    "pressure_advance",
    "adaptive_pressure_advance",
    "adaptive_pressure_advance_model",
    "adaptive_pressure_advance_overhangs",
    "adaptive_pressure_advance_bridges",
];

fn cohort(text: &str) -> Vec<&str> {
    KEYS.iter()
        .map(|key| {
            let prefix = format!("; {key} = ");
            let mut lines = text.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next().unwrap();
            assert!(lines.next().is_none(), "one footer line per PA key");
            line
        })
        .collect()
}

fn pa_commands(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| !line.starts_with(';') && line.contains("Override pressure advance value"))
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, expected: &[&str]) {
    let (reference, _) = native_stream_case::qualify(
        (
            "pa-variant",
            &[
                "PrintConfig.cpp:10199-10205,11862-11882,11971-11977",
                "GCode.cpp:11330-11340",
                "GCodeWriter.cpp:599-637",
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
    let native_section = cohort(&reference.native);
    assert_eq!(native_section, expected, "Native selected PA cohort");
    let native_commands = pa_commands(&reference.native);
    if label == "static-enable" {
        assert!(!native_commands.is_empty());
        assert!(
            native_commands
                .iter()
                .all(|line| line.starts_with("M900 K0.044")
                    && line.ends_with("; Override pressure advance value")),
            "Native selects source column 3 PA: {native_commands:?}"
        );
    } else {
        assert!(native_commands.is_empty(), "selected PA enable is false");
    }
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
    let api_section = cohort(std::str::from_utf8(&api).unwrap());
    let cli_section = cohort(std::str::from_utf8(&cli_bytes).unwrap());
    let api_commands = pa_commands(std::str::from_utf8(&api).unwrap());
    let cli_commands = pa_commands(std::str::from_utf8(&cli_bytes).unwrap());
    fs::write(reference.root.join("labelled-section-comparison.json"),serde_json::to_vec_pretty(&json!({"native_section":native_section,"api_section":api_section,"cli_section":cli_section,"section_matches":native_section==api_section && native_section==cli_section,"native_commands":native_commands,"api_commands":api_commands,"cli_commands":cli_commands,"commands_match":native_commands==api_commands && native_commands==cli_commands,"native_activity":activity,"input_sha256":sha,"api_cli_raw_equal":api==cli_bytes,"whole_output_parity_claimed":false})).unwrap()).unwrap();
    assert_eq!(native_section, api_section, "labelled API PA cohort");
    assert_eq!(native_section, cli_section, "labelled CLI PA cohort");
    assert_eq!(native_commands, api_commands, "API selected PA commands");
    assert_eq!(native_commands, cli_commands, "CLI selected PA commands");
}

#[test]
fn pa_variant_short_vectors() {
    check(
        "short-vectors",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/l59.3mf"
        )),
        "908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d",
        &[
            "; enable_pressure_advance = 0,0",
            "; pressure_advance = 0.02,0.02",
            "; adaptive_pressure_advance = 0,0",
            r#"; adaptive_pressure_advance_model = "0,0,0\n0,0,0";"0,0,0\n0,0,0""#,
            "; adaptive_pressure_advance_overhangs = 0,0",
            "; adaptive_pressure_advance_bridges = 0,0",
        ],
    );
}

#[test]
fn pa_variant_distinct_columns() {
    check(
        "distinct",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/pa-variant/distinct.3mf"
        )),
        "efa0226219fe97efb1e23609ab2679dfd005d6d2fca2f070cdd9a5e77f209b9f",
        &[
            "; enable_pressure_advance = 0,0",
            "; pressure_advance = 0.022,0.044",
            "; adaptive_pressure_advance = 1,0",
            r#"; adaptive_pressure_advance_model = "0.021,1,1000\n0.022,2,2000";"0.041,1,1000\n0.042,2,2000""#,
            "; adaptive_pressure_advance_overhangs = 0,1",
            "; adaptive_pressure_advance_bridges = 0.062,0.084",
        ],
    );
}

#[test]
fn pa_variant_static_enable() {
    check(
        "static-enable",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/pa-variant/static-enable.3mf"
        )),
        "adc2f063ecb66f92c4a25cbbbaa43208c114cdfe059ed9558336c82c3361ceed",
        &[
            "; enable_pressure_advance = 0,1",
            "; pressure_advance = 0.022,0.044",
            "; adaptive_pressure_advance = 0,0",
            r#"; adaptive_pressure_advance_model = "0.021,1,1000\n0.022,2,2000";"0.041,1,1000\n0.042,2,2000""#,
            "; adaptive_pressure_advance_overhangs = 0,1",
            "; adaptive_pressure_advance_bridges = 0.062,0.084",
        ],
    );
}
