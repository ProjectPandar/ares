use crate::{OrcaRunner, runner::native_api_cli::NativeCase};
use serde_json::{Value, json};
use std::{fs, io::Cursor};
use zip::write::SimpleFileOptions;

const SINGLE: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");
const DUAL: &[u8] =
    include_bytes!("../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf");

#[test]
fn reprapfirmware_second_layer_sets_temperature_without_tool_or_wait() {
    compare(
        SINGLE,
        "reprapfirmware",
        &["210"],
        &["G10 S210 ; set nozzle temperature"],
    );
}

#[test]
fn reprapfirmware_second_layer_names_both_temperature_tools_with_p() {
    compare(
        DUAL,
        "reprapfirmware",
        &["210", "215"],
        &[
            "G10 S210 P0 ; set nozzle temperature",
            "G10 S215 P1 ; set nozzle temperature",
        ],
    );
}

#[test]
fn reprapfirmware_second_layer_leaves_equal_nozzle_target_unchanged() {
    compare(SINGLE, "reprapfirmware", &["220"], &[]);
}

#[test]
fn marlin_second_layer_sets_temperature_with_m104() {
    compare(
        SINGLE,
        "marlin",
        &["210"],
        &["M104 S210 ; set nozzle temperature"],
    );
}

fn project(base: &[u8], flavor: &str, ordinary: &[&str]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(base)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() != "Metadata/project_settings.config" {
            output.raw_copy_file(entry).unwrap();
            continue;
        }
        let mut settings: Value = serde_json::from_reader(entry).unwrap();
        settings["gcode_flavor"] = json!(flavor);
        settings["machine_start_gcode"] = json!("G90\nM83");
        settings["enable_prime_tower"] = json!("0");
        settings["ooze_prevention"] = json!("0");
        settings["nozzle_temperature_initial_layer"] = json!(vec!["220"; ordinary.len()]);
        settings["nozzle_temperature"] = json!(ordinary);
        settings["cool_plate_temp_initial_layer"] = json!(vec!["55"; ordinary.len()]);
        settings["cool_plate_temp"] = json!(vec!["50"; ordinary.len()]);
        output
            .start_file(
                "Metadata/project_settings.config",
                SimpleFileOptions::DEFAULT,
            )
            .unwrap();
        serde_json::to_writer(&mut output, &settings).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn compare(base: &[u8], flavor: &str, ordinary: &[&str], expected: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(base, flavor, ordinary);
    let case = NativeCase::slice(&runner, &project, "second-layer-nozzle-");
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(case.root.join("ares.gcode"), &actual).unwrap();
    assert_eq!(
        second_layer_thermal_commands(&case.native),
        expected,
        "Native at {:?}",
        case.root
    );
    assert_eq!(
        second_layer_thermal_commands(std::str::from_utf8(&actual).unwrap()),
        expected,
        "Ares at {:?}",
        case.root,
    );
}

fn second_layer_thermal_commands(gcode: &str) -> Vec<&str> {
    let second_layer = gcode.split(";LAYER_CHANGE\n").nth(2).unwrap();
    let (transition, _) = second_layer
        .split_once("M140 S50 ; set bed temperature\n")
        .expect("second layer changes the bed temperature");
    transition
        .lines()
        .filter(|line| {
            line.starts_with("M104 ")
                || line.starts_with("M109 ")
                || line.starts_with("M116")
                || line.starts_with("G10 S")
        })
        .collect()
}
