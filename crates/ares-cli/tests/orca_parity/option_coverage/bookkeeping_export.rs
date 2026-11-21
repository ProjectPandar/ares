use super::*;
use crate::runner::CaseInputs;
use std::io::{Cursor, Read, Write};

#[test]
fn bookkeeping_short_vectors_pad_to_filament_count() {
    if !crate::runner::seeded_reference_enabled() {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("disable_m73".into(), Value::String("1".into()));
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    let machine = machine.clone();
    let process = process.clone();
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "bookkeeping/short-vectors",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            // Two filaments, so OrcaSlicer.cpp:2817-2818 resizes both
            // bookkeeping vectors to filament_count + 2 = 4 entries
            // (grow with empty strings) before the config block is
            // serialized; the stored vectors are SHORT here.
            settings["different_settings_to_system"] = serde_json::json!(["outer_wall_speed", ""]);
            settings["inherits_group"] = serde_json::json!(["", ""]);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let project = output.finish().unwrap().into_inner();
    let directory = tempfile::Builder::new()
        .prefix("bookkeeping-short-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    std::fs::write(&path, &project).unwrap();
    runner::stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            directory.to_str().unwrap().to_owned(),
            path.to_str().unwrap().to_owned(),
        ],
        runner::stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let reference = std::fs::read(directory.join("plate_1.gcode")).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    assert!(
        text.lines()
            .any(|line| line == "; different_settings_to_system = outer_wall_speed;;;"),
        "short different_settings_to_system must resize to filament_count + 2"
    );
    assert!(
        text.lines().any(|line| line == "; inherits_group = ;;;"),
        "short inherits_group must resize to filament_count + 2"
    );
    let outcome = crate::artifacts::compare(
        &crate::artifacts::root_from_env().unwrap(),
        "bookkeeping/short-vectors",
        Ok(&project),
        Ok(&reference),
        serde_json::json!({"kind": "bookkeeping_reference", "binary": std::env::var("ORCA_REPLAY_BIN").unwrap(), "seed": std::env::var("ORCA_REPLAY_SEED").unwrap(), "cpu": std::env::var("ORCA_REPLAY_CPU").unwrap()}),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn configure_different_settings(settings: &mut Value, supplied: bool) {
    if supplied {
        settings["different_settings_to_system"] =
            serde_json::json!(["outer_wall_speed", "", "", ""]);
    } else {
        settings
            .as_object_mut()
            .unwrap()
            .remove("different_settings_to_system");
    }
}

#[test]
fn bookkeeping_matches_complete_configured_filament_output() {
    if !crate::runner::seeded_reference_enabled() {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("disable_m73".into(), Value::String("1".into()));
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    for (count, supplied, absolute) in [
        (1, false, false),
        (2, false, false),
        (2, true, false),
        (2, false, true),
    ] {
        let mut machine = machine.clone();
        let mut process = process.clone();
        if absolute {
            machine.insert("use_relative_e_distances".into(), Value::String("0".into()));
            process.insert(
                "before_layer_change_gcode".into(),
                Value::String(";LAYER_E {e_position[0]}\n".into()),
            );
        }
        let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); count];
        parity::normalize_filament_defaults(&mut filaments);
        if absolute {
            for filament in &mut filaments {
                filament.insert(
                    "filament_start_gcode".into(),
                    serde_json::json!([
                        "{e_position[0]=12}\n;START_E_BEFORE_RESET {e_position[0]}\n"
                    ]),
                );
            }
        }
        let label = format!("bookkeeping/{count}/supplied-{supplied}/absolute-{absolute}");
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &parity::smoke_overrides(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..input.len() {
            let mut file = input.by_index(index).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            if file.name() == "Metadata/project_settings.config" {
                let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
                configure_different_settings(&mut settings, supplied);
                bytes = serde_json::to_vec(&settings).unwrap();
            }
            output
                .start_file(
                    file.name(),
                    zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
                )
                .unwrap();
            output.write_all(&bytes).unwrap();
        }
        let project = output.finish().unwrap().into_inner();
        let settings = runner::application::settings(&project).unwrap();
        assert_eq!(
            settings["filament_diameter"].as_array().unwrap().len(),
            count
        );
        assert_eq!(
            settings.get("different_settings_to_system").is_some(),
            supplied
        );
        let directory = tempfile::Builder::new()
            .prefix("bookkeeping-")
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let path = directory.join("input.3mf");
        std::fs::write(&path, &project).unwrap();
        runner::stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                directory.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ],
            runner::stages::Stage::Slice,
            &directory,
        )
        .unwrap();
        let reference = std::fs::read(directory.join("plate_1.gcode")).unwrap();
        if absolute {
            assert!(
                std::str::from_utf8(&reference)
                    .unwrap()
                    .contains(";START_E_BEFORE_RESET 12\nT0\nG92 E0\n")
            );
        }
        let expected = if supplied {
            "outer_wall_speed;;;".to_owned()
        } else {
            ";".repeat(count + 1)
        };
        assert!(
            std::str::from_utf8(&reference)
                .unwrap()
                .lines()
                .any(|line| line == format!("; different_settings_to_system = {expected}"))
        );
        let outcome = crate::artifacts::compare(&crate::artifacts::root_from_env().unwrap(), &label, Ok(&project), Ok(&reference), serde_json::json!({"kind": "bookkeeping_reference", "binary": std::env::var("ORCA_REPLAY_BIN").unwrap(), "seed": std::env::var("ORCA_REPLAY_SEED").unwrap(), "cpu": std::env::var("ORCA_REPLAY_CPU").unwrap()})).unwrap();
        assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    }
}
