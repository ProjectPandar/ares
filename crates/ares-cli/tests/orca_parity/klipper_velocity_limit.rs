//! `GCodeProcessor.cpp:5287-5298::process_SET_VELOCITY_LIMIT` public
//! full-output case: Klipper `ACCEL` above `machine_max_acceleration_extruding`
//! must still set the unclamped travel acceleration (Klipper has no separate
//! travel limit, `GCodeWriter::supports_separate_travel_acceleration`), which
//! the M73 / estimated-time lines expose.
use crate::{
    CaseInputs, VendorProfiles,
    runner::{OrcaRunner, application, repo_root},
};
use serde_json::json;

#[test]
fn klipper_velocity_limit_accel_above_print_limit_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality K1 (0.4 nozzle)").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let end = format!(
        "; VELOCITY_LIMIT_TEST_BEGIN\nSET_VELOCITY_LIMIT ACCEL=6000\nG1 X10 Y10 F12000\nG1 X200 Y200 F12000\nG1 X10 Y200 F12000\n; VELOCITY_LIMIT_TEST_END\n{}",
        machine["machine_end_gcode"].as_str().unwrap()
    );
    let print_limit = json!(["1000", "1000"]);
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.insert("machine_end_gcode".into(), json!(end));
    overrides.insert(
        "machine_max_acceleration_extruding".into(),
        print_limit.clone(),
    );
    overrides.insert("disable_m73".into(), json!("0"));
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "klipper-velocity-limit/accel-above-print-limit",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    for (key, value) in [
        ("machine_end_gcode", json!(end)),
        ("machine_max_acceleration_extruding", print_limit),
        ("disable_m73", json!("0")),
        ("gcode_flavor", json!("klipper")),
    ] {
        application::verify_value(&exported.project, key, &value).unwrap();
    }
    let case = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&case.reference).unwrap();
    assert!(reference.contains("; VELOCITY_LIMIT_TEST_BEGIN\nSET_VELOCITY_LIMIT ACCEL=6000\n"));
    assert!(reference.lines().any(|line| line.starts_with("M73 ")));
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
