//! Preset.cpp:524-541 -> PrintConfig.cpp:10630-10685: normalize physical
//! extruder metadata before process variant columns are selected.
use crate::runner::{application, native_api_cli};

const J1: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/parity/acceleration-jerk-variants/endpoint-contact.3mf"
));
const J1_SHA: &str = "b1c6d193f0b990473cc857fdbde50a4ddfc7a739aff30bf3ce3810529a0e9529";

fn config_lines(output: &[u8], expected: &[(&str, &str)]) -> Vec<String> {
    let output = std::str::from_utf8(output).unwrap();
    expected
        .iter()
        .map(|(key, _)| {
            let prefix = format!("; {key} = ");
            let mut lines = output.lines().filter(|line| line.starts_with(&prefix));
            let line = lines.next().unwrap();
            assert!(lines.next().is_none(), "one config line for {key}");
            line.to_owned()
        })
        .collect()
}

fn check(label: &str, input: &[u8], sha: &str, semm: &str, expected: &[(&str, &str)]) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(native_api_cli::sha(input), sha);
    let case = native_api_cli::NativeCase::slice(&runner, input, label);
    assert_eq!(
        config_lines(
            case.native.as_bytes(),
            &[("single_extruder_multi_material", semm)]
        ),
        [format!("; single_extruder_multi_material = {semm}")],
        "Native physical-extruder branch premise"
    );
    let native = config_lines(case.native.as_bytes(), expected);
    let literal: Vec<_> = expected
        .iter()
        .map(|(key, value)| format!("; {key} = {value}"))
        .collect();
    assert_eq!(native, literal, "Native normalization premise");
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
        std::fs::write(case.root.join("ares.api.gcode"), bytes).unwrap();
    }
    let path = case.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "CLI slicing failed: {:?}", cli.stderr);
    let api = api.unwrap();
    let cli = std::fs::read(path).unwrap();
    assert_eq!(
        (config_lines(&api, expected), config_lines(&cli, expected)),
        (native.clone(), native),
        "API and CLI normalization lines must both equal runtime Native"
    );
}

#[test]
fn j1_two_physical_nozzles_expand_extruder_variant_list() {
    check(
        "preset-normalize-j1-list-",
        J1,
        J1_SHA,
        "0",
        &[(
            "extruder_variant_list",
            "\"Direct Drive Standard\";\"Direct Drive Standard\"",
        )],
    );
}

#[test]
fn j1_process_columns_select_second_physical_extruder() {
    check(
        "preset-normalize-j1-process-",
        J1,
        J1_SHA,
        "0",
        &[("print_extruder_id", "1,2")],
    );
}

#[test]
fn ender3_one_nozzle_preserves_two_supported_variants() {
    check(
        "preset-normalize-ender3-control-",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/process-variant-columns/ender3-standard-nozzle.3mf"
        )),
        "036316b8d1fd464e9209a6efb85b71396aa197eb8cb0798ead31eec7a18133ca",
        "1",
        &[
            ("nozzle_diameter", "0.4"),
            (
                "extruder_variant_list",
                "\"Direct Drive Standard,Direct Drive High Flow\"",
            ),
            ("print_extruder_id", "1"),
        ],
    );
}

#[test]
fn mk4_two_filaments_keep_one_physical_nozzle() {
    check(
        "preset-normalize-mk4-control-",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/mk4-layer-change/mk4base.3mf"
        )),
        "d0a4dfdb7f89120719b380a2995687a81a7e699025b8f3734349480bdebdc046",
        "1",
        &[("nozzle_diameter", "0.4"), ("print_extruder_id", "1,1")],
    );
}

#[test]
fn x1c_two_filaments_do_not_become_two_physical_extruders() {
    check(
        "preset-normalize-x1c-control-",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bbl-pressure-advance-precedence/bbl-stock-pa-control.3mf"
        )),
        "1d56a96dfa6bb2d86ef8a2d29be12ee0ea395e25d96172ee9a07a87f70456b14",
        "1",
        &[
            ("nozzle_diameter", "0.4"),
            (
                "extruder_variant_list",
                "\"Direct Drive Standard,Direct Drive High Flow\"",
            ),
            ("print_extruder_id", "1"),
        ],
    );
}
