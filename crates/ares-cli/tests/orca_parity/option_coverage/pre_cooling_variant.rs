//! PrintConfig.hpp:1926,1929: nullable integer pre-cooling members.
use crate::{
    golden,
    runner::{application, native_stream_case},
};
use serde_json::json;
use std::fs;

const KEYS: [&str; 2] = [
    "filament_pre_cooling_temperature",
    "filament_pre_cooling_temperature_nc",
];

fn cohort(text: &str) -> Vec<Option<&str>> {
    KEYS.iter()
        .map(|key| {
            let prefix = format!("; {key} = ");
            let mut lines = text.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next();
            assert!(
                lines.next().is_none(),
                "at most one pre-cooling footer line"
            );
            line
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, expected: &[Option<&str>]) {
    let (reference, _) = native_stream_case::qualify(
        (
            "pre-cooling-variant",
            &[
                "PrintConfig.cpp:8879-8886,8908-8916,10154-10159",
                "PrintConfig.cpp:11862-11882,11971-11977",
                "PrintConfig.hpp:1926,1929; Config.hpp:1085-1100",
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
    assert_eq!(
        native_section, expected,
        "Native nullable pre-cooling cohort"
    );
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
    fs::write(reference.root.join("labelled-section-comparison.json"),serde_json::to_vec_pretty(&json!({"native_section":native_section,"api_section":api_section,"cli_section":cli_section,"section_matches":native_section==api_section && native_section==cli_section,"native_activity":activity,"input_sha256":sha,"api_cli_raw_equal":api==cli_bytes,"whole_output_parity_claimed":false})).unwrap()).unwrap();
    assert_eq!(
        native_section, api_section,
        "labelled API pre-cooling cohort"
    );
    assert_eq!(
        native_section, cli_section,
        "labelled CLI pre-cooling cohort"
    );
}

#[test]
fn pre_cooling_variant_absent_defaults() {
    check(
        "absent-defaults",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/variant-retraction/l59.3mf"
        )),
        "908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d",
        &[
            Some("; filament_pre_cooling_temperature = 0,0"),
            Some("; filament_pre_cooling_temperature_nc = 0,0"),
        ],
    );
}

#[test]
fn pre_cooling_variant_distinct_columns() {
    check(
        "distinct",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/pre-cooling-variant/distinct.3mf"
        )),
        "d738fb48a4dbd450afe210eb8cee0bc226481b933158650efdd249701ad59bed",
        &[
            Some("; filament_pre_cooling_temperature = 132,154"),
            Some("; filament_pre_cooling_temperature_nc = 172,194"),
        ],
    );
}

#[test]
fn pre_cooling_variant_mixed_nil_zero() {
    check(
        "mixed",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/pre-cooling-variant/mixed.3mf"
        )),
        "561ca738f30133e99af9b162ceb2e83586d1068edfa09b2e5ddf2d914d6ca387",
        &[
            Some("; filament_pre_cooling_temperature = 0,nil"),
            Some("; filament_pre_cooling_temperature_nc = nil,0"),
        ],
    );
}

#[test]
fn pre_cooling_variant_all_nil_absence() {
    check(
        "all-nil",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/pre-cooling-variant/all-nil.3mf"
        )),
        "3c94201801b3c2d42bc953d8639d78d6e11e202502e5f92fa353c83063aa555c",
        &[None, Some("; filament_pre_cooling_temperature_nc = 0,0")],
    );
}
