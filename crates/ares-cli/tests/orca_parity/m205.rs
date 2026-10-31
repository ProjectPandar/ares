//! Source-owned M205 mutations and minimum-rate consumers (GCodeProcessor.cpp:
//! 5236–5265, 3987–3989, 4347–4349, 5726–5740). Failure inventory is recorded
//! before production in docs/architecture/gcode-processor-m205-machine-limits.md.

use crate::runner::{OrcaRunner, stages};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const PROJECT: &[u8] = include_bytes!("../../../../tests/parity/orca_cli_ender3.3mf");

fn input(flavor: &str, rates: [&str; 2], setter: &str) -> Vec<u8> {
    let start = format!(
        "; M205_WITNESS_BEGIN\nG90\nM83\nG92 E0\nM204 P600 R600 T600\n{setter}\n\
         G1 X10 Y10 Z0.3 F120\nG1 Y90 F60\nG1 X90 E0.4 F60\n\
         G1 Y10 E-0.4 F60\nG1 E-2 F60\nG1 E2 F60\nG1 X10 Y10 F60\n\
         ; M205_WITNESS_END"
    );
    let changes = json!({
        "machine_start_gcode": start, "gcode_flavor": flavor, "disable_m73": "0",
        "machine_min_extruding_rate": [rates[0], rates[0]],
        "machine_min_travel_rate": [rates[1], rates[1]],
        "machine_max_jerk_x": ["10", "10"], "machine_max_jerk_y": ["12", "12"],
        "machine_max_junction_deviation": ["0.04", "0.04"]
    });
    let mut archive = zip::ZipArchive::new(Cursor::new(PROJECT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for (key, value) in changes.as_object().unwrap() {
                settings[key] = value.clone();
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn check(name: &str, flavor: &str, rates: [&str; 2], setter: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("m205-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("{name}: artifacts {directory:?}");
    let project = input(flavor, rates, setter);
    let settings = crate::runner::application::settings(&project).unwrap();
    assert_eq!(
        settings["machine_min_extruding_rate"],
        json!([rates[0], rates[0]])
    );
    assert_eq!(
        settings["machine_min_travel_rate"],
        json!([rates[1], rates[1]])
    );
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, &project).unwrap();
    let output = directory.join("orca");
    fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            output.to_str().unwrap().to_owned(),
            input_path.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let reference = stages::read(&output.join("plate_1.gcode"), stages::Stage::Slice).unwrap();
    let processed = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_processed(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &processed.gcode).unwrap();
    let reference_text = std::str::from_utf8(&reference).unwrap();
    let actual_text = std::str::from_utf8(&processed.gcode).unwrap();
    assert!(reference_text.contains("; M205_WITNESS_BEGIN\n"));
    assert!(reference_text.contains("; M205_WITNESS_END\n"));
    if !setter.is_empty() {
        assert!(reference_text.contains(setter));
    } else {
        assert!(
            !reference_text.lines().any(|line| {
                line.starts_with("M205 ")
                    && line
                        .split_whitespace()
                        .skip(1)
                        .any(|word| word.starts_with('S') || word.starts_with('T'))
            }),
            "configuration witness must not be masked by an emitted S/T setter"
        );
    }
    assert!(reference_text.contains("M73 P"));
    let ordered = crate::golden::compare_ordered_bytes_generator_only(&reference, &processed.gcode);
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "name": name, "flavor": flavor, "rates": rates, "setter": setter,
        "ordered_bytes": ordered,
        "native_m73": reference_text.lines().filter(|line| line.starts_with("M73 ")).collect::<Vec<_>>(),
        "ares_m73": actual_text.lines().filter(|line| line.starts_with("M73 ")).collect::<Vec<_>>(),
        "native_footer_time": reference_text.lines().filter(|line| line.starts_with("; estimated printing time")).collect::<Vec<_>>(),
        "ares_footer_time": actual_text.lines().filter(|line| line.starts_with("; estimated printing time")).collect::<Vec<_>>()
    })).unwrap()).unwrap();
    ordered.unwrap_or_else(|difference| panic!("{name}: {difference}; {directory:?}"));
}

#[test]
fn m205_config_minimum_rates() {
    check("config-minimum-rates", "marlin", ["6", "12"], "");
}
#[test]
fn m205_start_minimum_rates() {
    check("start-minimum-rates", "marlin", ["0", "0"], "M205 S6 T12");
}
#[test]
fn m205_start_x_inherits_y() {
    check("start-x-inherits-y", "marlin", ["0", "0"], "M205 X1");
}
#[test]
fn m205_start_junction_deviation() {
    check(
        "start-junction-deviation",
        "marlin2",
        ["0", "0"],
        "M205 J0.013",
    );
}
#[test]
fn m205_zero_control() {
    check("zero-control", "marlin", ["0", "0"], "");
}
#[test]
fn m205_rrf_configuration_reset() {
    check("rrf-configuration-reset", "reprapfirmware", ["6", "12"], "");
}
