//! Active Type2 tool changes: IDEX without front-edge priming and SEMM with/without it.

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::json;

#[test]
fn smooth_type2_tower_tools_and_priming() {
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
            "semm-infill-copies-no-brim",
            "Creality",
            "Creality Ender-3 0.4 nozzle",
            "1",
            "0",
        ),
        (
            "semm-ramming-short",
            "Creality",
            "Creality Ender-3 0.4 nozzle",
            "1",
            "0",
        ),
        (
            "semm-ramming-long",
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
        let copies = label.starts_with("semm-infill-copies");
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
        if copies {
            let start = format!(
                "{}\n; COPY_CENTER {{first_layer_center_no_wipe_tower[0]}},{{first_layer_center_no_wipe_tower[1]}}",
                machine["machine_start_gcode"].as_str().unwrap(),
            );
            machine.insert("machine_start_gcode".into(), json!(start));
        }
        let mut process = profiles.process(&selection.process).unwrap();
        crate::normalize_process_defaults(&machine, &mut process);
        if label == "semm-infill-copies-no-brim" {
            process.insert("brim_type".into(), json!("no_brim"));
        }
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
        let asymmetric = label == "semm-asymmetric" || label.starts_with("semm-infill");
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
        let ramming = label.starts_with("semm-ramming-");
        let tube = if label.ends_with("short") { "4" } else { "10" };
        let ramming_machine = [
            ("enable_filament_ramming", json!("1")),
            ("high_current_on_filament_swap", json!("1")),
            ("cooling_tube_retraction", json!("35")),
            ("cooling_tube_length", json!(tube)),
            ("parking_pos_retraction", json!("80")),
            ("extra_loading_move", json!("5")),
        ];
        let ramming_filament = [
            ("filament_ramming_parameters", json!(["120 100 6 6 6 6"])),
            ("filament_cooling_moves", json!(["2"])),
            ("filament_cooling_initial_speed", json!(["2"])),
            ("filament_cooling_final_speed", json!(["4"])),
            ("filament_stamping_distance", json!(["3"])),
            ("filament_stamping_loading_speed", json!(["20"])),
            ("filament_loading_speed", json!(["28"])),
            ("filament_loading_speed_start", json!(["3"])),
            ("filament_unloading_speed", json!(["90"])),
            ("filament_unloading_speed_start", json!(["100"])),
        ];
        if ramming {
            for (key, value) in &ramming_machine {
                machine.insert((*key).into(), value.clone());
            }
            for filament in &mut filaments {
                filament.extend(
                    ramming_filament
                        .iter()
                        .map(|(key, value)| ((*key).into(), value.clone())),
                );
            }
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
        if label == "semm-infill-copies-no-brim" {
            runner::application::verify_value(&exported.project, "brim_type", &json!("no_brim"))
                .unwrap();
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
        if ramming {
            for (key, value) in &ramming_machine {
                runner::application::verify_value(&exported.project, key, value).unwrap();
            }
            for (key, value) in &ramming_filament {
                let both = json!([value[0].clone(), value[0].clone()]);
                runner::application::verify_value(&exported.project, key, &both).unwrap();
            }
        }
        let exported = if copies {
            runner::object_copy_second_print_instance::second_copy(exported)
        } else {
            exported
        };
        let sliced = runner.slice_case(&exported).unwrap();
        let reference = std::str::from_utf8(&sliced.reference).unwrap();
        if copies {
            // This reference labels both instances "copy 0". Verify actual
            // extrusion at both translated cube contours, not that label.
            assert!(reference.contains("G1 X90.31 Y114.69 E."));
            assert!(reference.contains("G1 X120.31 Y114.69 E."));
            let center_x = if label.ends_with("no-brim") { 95 } else { 110 };
            assert!(reference.contains(&format!("; COPY_CENTER {center_x},110\n")));
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
        if ramming {
            for phase in [
                "; Ramming start\n",
                "; Retract(unload)\n",
                "; Cooling\n",
                "; Cooling park\n",
                "; CP TOOLCHANGE LOAD\n",
                "M907 E750\n",
            ] {
                assert!(reference.contains(phase), "{label}: missing {phase:?}");
            }
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
            if copies {
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
