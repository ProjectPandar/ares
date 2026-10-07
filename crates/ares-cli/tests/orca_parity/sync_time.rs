//! `GCodeProcessor.cpp:5430–5436::process_SYNC` public full-output RED/controls.
use crate::{
    CaseInputs, VendorProfiles,
    runner::{OrcaRunner, application, repo_root},
};
use serde_json::json;

fn check(firmware: &str, delay: Option<&str>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (vendor, printer, flavor) = match firmware {
        "klipper" => ("Creality", "Creality K1 (0.4 nozzle)", "klipper"),
        "marlin" => ("Creality", "Creality Ender-3 0.4 nozzle", "marlin"),
        "bbl" => ("BBL", "Bambu Lab X2D 0.4 nozzle", "marlin"),
        _ => unreachable!(),
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), vendor).unwrap();
    let selection = crate::select_printer(&profiles, vendor, printer).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let sync = delay.map_or_else(String::new, |delay| format!("SYNC T{delay}\n"));
    let end = format!(
        "; SYNC_TIME_TEST_BEGIN\nG1 X10 Y10 F1200\n{sync}G1 X20 Y10 F1200\n; SYNC_TIME_TEST_END\n{}",
        machine["machine_end_gcode"].as_str().unwrap()
    );
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.insert("machine_end_gcode".into(), json!(end));
    overrides.insert("disable_m73".into(), json!("0"));
    let label = format!("sync-time/{firmware}/{}", delay.unwrap_or("none"));
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
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
        ("disable_m73", json!("0")),
        ("gcode_flavor", json!(flavor)),
    ] {
        application::verify_value(&exported.project, key, &value).unwrap();
    }
    let case = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&case.reference).unwrap();
    assert!(reference.contains("; SYNC_TIME_TEST_BEGIN\nG1 X10 Y10 F1200\n"));
    assert!(reference.contains("G1 X20 Y10 F1200\n"));
    assert!(reference.contains("; SYNC_TIME_TEST_END\n"));
    if let Some(delay) = delay {
        assert!(
            reference
                .lines()
                .any(|line| line == format!("SYNC T{delay}"))
        );
    }
    assert!(reference.lines().any(|line| line.starts_with("M73 ")));
    let outcome = crate::compare_case(&case);
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn sync_time_klipper_positive_delay_matches_native() {
    check("klipper", Some("5"));
}
#[test]
fn sync_time_marlin_positive_delay_matches_native() {
    check("marlin", Some("5"));
}
#[test]
fn sync_time_bbl_positive_delay_matches_native() {
    check("bbl", Some("5"));
}
#[test]
fn sync_time_klipper_without_sync_matches_native() {
    check("klipper", None);
}
#[test]
fn sync_time_marlin_without_sync_matches_native() {
    check("marlin", None);
}
#[test]
fn sync_time_bbl_without_sync_matches_native() {
    check("bbl", None);
}
#[test]
fn sync_time_klipper_zero_delay_matches_native() {
    check("klipper", Some("0"));
}
#[test]
fn sync_time_marlin_zero_delay_matches_native() {
    check("marlin", Some("0"));
}
#[test]
fn sync_time_bbl_zero_delay_matches_native() {
    check("bbl", Some("0"));
}
