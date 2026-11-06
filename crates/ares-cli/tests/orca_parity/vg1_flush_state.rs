//! `GCodeProcessor.cpp:4237–4546::process_VG1`: public full-byte RED.
use crate::{
    CaseInputs, VendorProfiles,
    runner::{OrcaRunner, application, repo_root, vg1_two_filament_inputs::bind_two_filaments},
};
use serde_json::json;
#[path = "vg1_flush_state/state_oracle.rs"]
mod state_oracle;

#[path = "vg1_flush_state/volume_oracle.rs"]
mod volume_oracle;

const VIRTUAL_SEGMENT: &str =
    ";VG1 E4 F523\n;VG1 E4 F261\n; VFLUSH_START\n;VG1 E34.9231 F523\n; VFLUSH_END\n";

#[derive(Clone, Copy)]
enum Oracle {
    Bytes,
    State,
    Volumes,
}

fn check(printer: &str, virtual_segment: &str, oracle: Oracle) {
    let verify_state = matches!(oracle, Oracle::State);
    let virtual_moves = !virtual_segment.is_empty();
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
    let state_probe = if verify_state {
        "M82\n; VG1_STATE_PROBE\nG1 X25 Y10 E45\nM83\n"
    } else {
        ""
    };
    let end = format!(
        "; VG1_TEST_BEGIN\nM83\nG92 E0\nG1 X10 Y10 F1200\nT1 H-1\n{virtual_segment}{state_probe}SYNC T5\nM1020 S1\nT0 H-1\nM1020 S0\nG1 X20 Y10 F1200\n; VG1_TEST_END\n{}",
        machine["machine_end_gcode"].as_str().unwrap()
    );
    let mut overrides = crate::smoke_case_overrides(&machine, &process);
    overrides.insert("machine_end_gcode".into(), json!(end));
    overrides.insert("disable_m73".into(), json!("0"));
    overrides.insert("filament_map".into(), json!(["1", "1"]));
    let label = format!(
        "vg1_flush_state/{printer}/{}",
        if verify_state {
            "state"
        } else if virtual_moves {
            "positive"
        } else {
            "none"
        }
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
    if virtual_moves && !virtual_segment.lines().any(|line| line.starts_with("G1 ")) {
        assert!(reference.contains(virtual_segment));
    }
    if virtual_segment.lines().any(|line| line.starts_with("G1 ")) {
        let mut reference_lines = reference.lines();
        for expected in virtual_segment.lines() {
            assert!(
                reference_lines.any(|line| line == expected),
                "missing ordered fixture line: {expected}"
            );
        }
    }
    assert!(reference.lines().any(|line| line.starts_with("M73 ")));
    match oracle {
        Oracle::State => {
            state_oracle::check(&case.project);
            return;
        }
        Oracle::Volumes => {
            volume_oracle::check(&case.project);
            return;
        }
        Oracle::Bytes => {}
    }
    let outcome = crate::compare_case(&case);
    if virtual_segment.starts_with("VG1 ") {
        crate::option_coverage::multitool_ramming_footer::check(&outcome);
    }
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn vg1_x1c_dual_filament_transition_matches_native() {
    check(
        "Bambu Lab X1 Carbon 0.4 nozzle",
        VIRTUAL_SEGMENT,
        Oracle::Bytes,
    );
}
#[test]
fn vg1_x1c_dual_filament_without_virtual_moves_matches_native() {
    check("Bambu Lab X1 Carbon 0.4 nozzle", "", Oracle::Bytes);
}

#[test]
fn vg1_x1c_virtual_state_and_vertex_effects_match_native() {
    if crate::native_environment::binary("ARES_PROCGCODE_BIN").is_some() {
        check(
            "Bambu Lab X1 Carbon 0.4 nozzle",
            VIRTUAL_SEGMENT,
            Oracle::State,
        );
    }
}

#[test]
fn vg1_x1c_zero_displacement_matches_native() {
    check(
        "Bambu Lab X1 Carbon 0.4 nozzle",
        ";VG1 E0 F1200\n",
        Oracle::Bytes,
    );
}

#[test]
fn vg1_x1c_ordinary_and_spaced_comments_are_not_commands() {
    check(
        "Bambu Lab X1 Carbon 0.4 nozzle",
        ";G1 E80 F523\n; VG1 E80 F523\n",
        Oracle::Bytes,
    );
}

#[test]
fn vg1_x1c_direct_command_matches_native() {
    check(
        "Bambu Lab X1 Carbon 0.4 nozzle",
        "VG1 E4 F523\nVG1 E4 F261\n; VFLUSH_START\nVG1 E34.9231 F523\n; VFLUSH_END\n",
        Oracle::Bytes,
    );
}

#[test]
fn vg1_x1c_global_relative_and_absent_feedrate_state_matches_native() {
    if crate::native_environment::binary("ARES_PROCGCODE_BIN").is_some() {
        check(
            "Bambu Lab X1 Carbon 0.4 nozzle",
            ";VG1 E4 F523\nG91\nM82\n;VG1 X2 Y3 E4\nG90\n",
            Oracle::State,
        );
    }
}

#[test]
fn vg1_x1c_origin_and_inches_state_matches_native() {
    if crate::native_environment::binary("ARES_PROCGCODE_BIN").is_some() {
        check(
            "Bambu Lab X1 Carbon 0.4 nozzle",
            "G92 X5 Y5 E0\nG20\n;VG1 X2 Y3 E4 F523\n;VG1 E0 F261\nG21\n",
            Oracle::State,
        );
    }
}

fn volumes(segment: &str) {
    if crate::native_environment::binary("ARES_PROCGCODE_BIN").is_some() {
        check("Bambu Lab X1 Carbon 0.4 nozzle", segment, Oracle::Volumes);
    }
}

#[test]
fn vg1_x1c_flush_below_remaining_volume_five_maps_match_native() {
    volumes(VIRTUAL_SEGMENT);
}

#[test]
fn vg1_x1c_flush_overflow_five_maps_match_native() {
    volumes("; VFLUSH_START\n;VG1 E80 F523\n; VFLUSH_END\n");
}

#[test]
fn vg1_x1c_repeated_flush_five_maps_match_native() {
    volumes("; VFLUSH_START\n;VG1 E20 F523\n;VG1 E80 F523\n;VG1 E4 F261\n; VFLUSH_END\n");
}

#[test]
fn vg1_x1c_inactive_zero_flush_five_maps_match_native() {
    volumes(";VG1 E80 F523\n; VFLUSH_START\n;VG1 E0 F261\n; VFLUSH_END\n;VG1 E4 F523\n");
}

#[test]
fn vg1_x1c_virtual_and_physical_flush_share_remainder() {
    volumes(
        "; VFLUSH_START\n;VG1 E20 F523\n; VFLUSH_END\n; FLUSH_START\nG1 E80 F523\n; FLUSH_END\n",
    );
}

#[test]
fn vg1_x1c_no_virtual_five_maps_match_native() {
    volumes("");
}
