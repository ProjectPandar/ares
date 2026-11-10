//! `GCode.cpp:4086-4091` deferred print time placeholders, replaced by
//! `GCodeProcessor.cpp:1385-1407` in every template rendered after them.

use crate::runner::{CaseInputs, OrcaRunner, application, repo_root};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

const PLACEHOLDERS: &str = "d={print_time_day} h={print_time_hour} m={print_time_minute} \
     s={print_time_sec} total={print_time_total_sec} length={used_filament_length}";

#[test]
fn print_time_sec_is_the_seconds_remainder_in_every_template() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 V3 SE 0.2 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("layer_height", "0.14"),
        ("line_width", "0.22"),
        ("outer_wall_line_width", "0.22"),
        ("inner_wall_line_width", "0.22"),
    ] {
        process.insert(key.into(), json!(value));
    }
    let start = machine["machine_start_gcode"].as_str().unwrap().to_owned();
    let end = machine["machine_end_gcode"].as_str().unwrap().to_owned();
    let templates = [
        ("file_start_gcode", format!("; FILE_START {PLACEHOLDERS}")),
        (
            "machine_start_gcode",
            format!("; MACHINE_START {PLACEHOLDERS}\n{start}"),
        ),
        (
            "machine_end_gcode",
            format!("{end}\n; MACHINE_END {PLACEHOLDERS}"),
        ),
    ];
    for (key, value) in &templates {
        machine.insert((*key).into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "deferred-time-placeholders/cube",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for (key, value) in &templates {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    let case = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&case.reference).unwrap();
    for marker in ["; FILE_START ", "; MACHINE_START ", "; MACHINE_END "] {
        let line = reference
            .lines()
            .find(|line| line.starts_with(marker))
            .unwrap_or_else(|| panic!("Native renders {marker}"));
        let minute = line
            .split_whitespace()
            .find_map(|field| field.strip_prefix("m="))
            .and_then(|minute| minute.parse::<u32>().ok())
            .unwrap_or_else(|| panic!("Native replaces the minutes in {line}"));
        assert!(
            minute > 0,
            "the seconds remainder must differ from the total: {line}"
        );
    }
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
