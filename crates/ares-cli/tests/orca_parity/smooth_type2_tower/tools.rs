//! Active Type2 tool changes: IDEX without front-edge priming and SEMM with/without it.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn orca_parity_smooth_type2_tower_tools_and_priming() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let mut failures = Vec::new();
    for (label, vendor, printer, semm, priming) in [
        ("idex", "Snapmaker", "Snapmaker J1 (0.4 nozzle)", "0", "0"),
        ("semm", "Creality", "Creality Ender-3 0.4 nozzle", "1", "0"),
        (
            "semm-asymmetric",
            "Creality",
            "Creality Ender-3 0.4 nozzle",
            "1",
            "0",
        ),
        (
            "semm-infill",
            "Creality",
            "Creality Ender-3 0.4 nozzle",
            "1",
            "0",
        ),
        (
            "semm-infill-copies",
            "Creality",
            "Creality Ender-3 0.4 nozzle",
            "1",
            "0",
        ),
        (
            "semm-priming",
            "Creality",
            "Creality Ender-3 0.4 nozzle",
            "1",
            "1",
        ),
    ] {
        let profiles = VendorProfiles::load(
            &runner::repo_root().join("OrcaSlicer/resources/profiles"),
            vendor,
        )
        .unwrap();
        let selection = crate::select_printer(&profiles, vendor, printer).unwrap();
        let mut machine = profiles.machine(&selection.printer).unwrap();
        machine.insert("wipe_tower_type".into(), json!("type2"));
        machine.insert("single_extruder_multi_material".into(), json!(semm));
        machine.insert("filament_map_mode".into(), json!("Manual"));
        let map = if semm == "1" { ["1", "1"] } else { ["1", "2"] };
        machine.insert("filament_map".into(), json!(map));
        let mut process = profiles.process(&selection.process).unwrap();
        crate::normalize_process_defaults(&machine, &mut process);
        for (key, value) in [
            ("enable_prime_tower", json!("1")),
            ("timelapse_type", json!("1")),
            ("wipe_tower_wall_type", json!("rib")),
            ("prime_tower_width", json!("40")),
            ("wipe_tower_x", json!(["70"])),
            ("wipe_tower_y", json!(["150"])),
            ("outer_wall_filament_id", json!("1")),
            ("inner_wall_filament_id", json!("2")),
            ("single_extruder_multi_material_priming", json!(priming)),
        ] {
            process.insert(key.into(), value);
        }
        let asymmetric = matches!(
            label,
            "semm-asymmetric" | "semm-infill" | "semm-infill-copies"
        );
        let infill = if label.starts_with("semm-infill") {
            "1"
        } else {
            "0"
        };
        if asymmetric {
            machine.insert("purge_in_prime_tower".into(), json!("1"));
            process.insert(
                "flush_volumes_matrix".into(),
                json!(["0", "200", "110", "0"]),
            );
            process.insert("flush_multiplier".into(), json!(["0.75"]));
            process.insert("flush_into_infill".into(), json!(infill));
        }
        let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
        crate::normalize_filament_defaults(&mut filaments);
        for (filament, color) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
            filament.insert("filament_colour".into(), json!([color]));
            filament.insert("filament_is_support".into(), json!(["0"]));
        }
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("type2-tools/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &crate::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let exported = runner
            .bind_filament_map_mode(exported, &json!("Manual"))
            .unwrap();
        for (key, value) in [
            ("enable_prime_tower", json!("1")),
            ("timelapse_type", json!("1")),
            ("wipe_tower_type", json!("type2")),
            ("single_extruder_multi_material", json!(semm)),
            ("single_extruder_multi_material_priming", json!(priming)),
            ("outer_wall_filament_id", json!("1")),
            ("inner_wall_filament_id", json!("2")),
            ("filament_map", json!(map)),
            ("wipe_tower_x", json!(["70"])),
            ("wipe_tower_y", json!(["150"])),
        ] {
            runner::application::verify_value(&exported.project, key, &value).unwrap();
        }
        if asymmetric {
            for (key, value) in [
                ("purge_in_prime_tower", json!("1")),
                ("flush_volumes_matrix", json!(["0", "200", "110", "0"])),
                ("flush_multiplier", json!(["0.75"])),
                ("flush_into_infill", json!(infill)),
            ] {
                runner::application::verify_value(&exported.project, key, &value).unwrap();
            }
        }
        let exported = if label == "semm-infill-copies" {
            runner::type2_copies::second_copy(exported)
        } else {
            exported
        };
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        if label == "semm-infill-copies" {
            // This reference labels both instances "copy 0". Verify actual
            // extrusion at both translated cube contours, not that label.
            assert!(reference.contains("G1 X90.31 Y114.69 E."));
            assert!(reference.contains("G1 X120.31 Y114.69 E."));
        }
        assert!(reference.contains(";TYPE:Prime tower\n"), "{label}");
        assert_eq!(
            reference.contains("; CP PRIMING START\n"),
            priming == "1",
            "{label}"
        );
        assert!(
            reference
                .lines()
                .filter(|line| *line == "T0" || *line == "T1")
                .count()
                > 3,
            "{label}"
        );
        if label.starts_with("semm-infill") {
            assert!(reference.contains("; PURGING FINISHED\n"));
        }
        let outcome = crate::compare_case(&sliced);
        if label.starts_with("semm-infill") {
            let actual =
                std::fs::read_to_string(outcome.artifacts.as_ref().unwrap().join("ares.gcode"))
                    .unwrap();
            assert_eq!(
                actual.matches("; PURGING FINISHED\n").count(),
                reference.matches("; PURGING FINISHED\n").count(),
                "{label}: real wiping/normal passes"
            );
            if label == "semm-infill-copies" {
                assert!(actual.contains("G1 X90.31 Y114.69 E."));
                assert!(actual.contains("G1 X120.31 Y114.69 E."));
            }
        }
        if outcome.status != "PASS" {
            failures.push(format!("{label}: {} {}", outcome.status, outcome.detail));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
