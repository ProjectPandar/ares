//! `GCodeProcessor.cpp:4237–4546::process_VG1`: public full-byte RED.
use crate::{
    CaseInputs, VendorProfiles,
    runner::{OrcaRunner, application, repo_root, vg1_inputs::bind_two_filaments},
};
use serde_json::json;
fn check(printer: &str, virtual_moves: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let selection = crate::select_printer(&profiles, "BBL", printer).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let filament = profiles.filament(&selection.filaments[0]).unwrap();
    let mut filaments = vec![filament.clone(), filament];
    crate::normalize_filament_defaults(&mut filaments);
    let virtual_segment = if virtual_moves {
        ";VG1 E4 F523\n;VG1 E4 F261\n; VFLUSH_START\n;VG1 E34.9231 F523\n; VFLUSH_END\n"
    } else {
        ""
    };
    let end = format!(
        "; VG1_TEST_BEGIN\nM83\nG92 E0\nG1 X10 Y10 F1200\nT1 H-1\n{virtual_segment}SYNC T5\nM1020 S1\nT0 H-1\nM1020 S0\nG1 X20 Y10 F1200\n; VG1_TEST_END\n{}",
        machine["machine_end_gcode"].as_str().unwrap()
    );
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.insert("machine_end_gcode".into(), json!(end));
    overrides.insert("disable_m73".into(), json!("0"));
    overrides.insert("filament_map".into(), json!(["1", "1"]));
    let label = format!(
        "vg1/{printer}/{}",
        if virtual_moves { "positive" } else { "none" }
    );
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
    let exported = bind_two_filaments(exported);
    for (key, value) in [
        ("machine_end_gcode", json!(end)),
        ("disable_m73", json!("0")),
        ("filament_map", json!(["1", "1"])),
        ("filament_colour", json!(["#F2754E", "#0000FF"])),
        ("flush_volumes_matrix", json!(["0", "140", "140", "0"])),
        ("flush_volumes_vector", json!(["140", "140", "140", "140"])),
    ] {
        application::verify_value(&exported.project, key, &value).unwrap();
    }
    let settings = application::settings(&exported.project).unwrap();
    assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 2);
    let case = runner.slice_case(&exported).unwrap();
    let reference = std::str::from_utf8(&case.reference).unwrap();
    assert!(reference.contains("; VG1_TEST_BEGIN\n"));
    assert!(reference.contains("T1 H-1\n"));
    assert!(reference.contains("M1020 S1\n"));
    assert!(reference.contains("T0 H-1\n"));
    assert!(reference.contains("M1020 S0\n"));
    assert!(reference.contains("; VG1_TEST_END\n"));
    if virtual_moves {
        assert!(reference.contains(virtual_segment));
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
fn vg1_x1c_dual_filament_transition_matches_native() {
    check("Bambu Lab X1 Carbon 0.4 nozzle", true);
}
#[test]
fn vg1_x1c_dual_filament_without_virtual_moves_matches_native() {
    check("Bambu Lab X1 Carbon 0.4 nozzle", false);
}
