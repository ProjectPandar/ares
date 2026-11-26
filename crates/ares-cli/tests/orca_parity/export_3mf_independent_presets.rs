use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};
use zip::{ZipWriter, write::SimpleFileOptions};

use super::export_3mf::{archive_members, synthetic_3mf};
use crate::OrcaRunner;

#[test]
fn export_3mf_replaces_only_settings_in_saved_bbl_project() {
    verify_export(
        true,
        &["--load-settings", "machine_same.json;process_compat.json"],
        |s| {
            assert_eq!(s["printable_height"], "220");
            assert_eq!(s["layer_height"], "0.24");
            assert_eq!(s["nozzle_temperature"], json!(["200"]));
            assert_eq!(s["filament_settings_id"], json!(["Synthetic PLA"]));
            assert_eq!(s["enable_prime_tower"], "1");
            assert_eq!(
                s["different_settings_to_system"],
                json!(["", "nozzle_temperature", ""])
            );
            assert_eq!(s["inherits_group"], json!(["", "ParentFila", ""]));
        },
    );
}

#[test]
fn export_3mf_replaces_only_settings_in_saved_non_bbl_project() {
    verify_export(
        false,
        &["--load-settings", "machine_same.json;process_compat.json"],
        |s| {
            assert_eq!(s["printable_height"], "220");
            assert_eq!(s["layer_height"], "0.24");
            assert_eq!(s["nozzle_temperature"], json!(["200"]));
            assert_eq!(s["different_settings_to_system"], json!(["", ""]));
            assert_eq!(s["inherits_group"], json!(["", ""]));
            assert!(s.get("upward_compatible_machine").is_none());
        },
    );
}

#[test]
fn export_3mf_replaces_only_filaments_in_saved_bbl_project() {
    verify_export(true, &["--load-filaments", "filament_petg.json"], |s| {
        assert_eq!(s["printable_height"], "200");
        assert_eq!(s["layer_height"], "0.2");
        assert_eq!(s["nozzle_temperature"], json!(["235"]));
        assert_eq!(s["filament_settings_id"], json!(["Synthetic PETG"]));
        assert_eq!(s["enable_prime_tower"], "0");
        assert_eq!(
            s["different_settings_to_system"],
            json!(["wall_loops;enable_prime_tower", "", "layer_change_gcode"])
        );
        assert_eq!(
            s["inherits_group"],
            json!(["ParentProc", "", "Synthetic single nozzle"])
        );
    });
}

#[test]
fn export_3mf_replaces_only_filaments_in_saved_non_bbl_project() {
    verify_export(false, &["--load-filaments", "filament_petg.json"], |s| {
        assert_eq!(s["printable_height"], "200");
        assert_eq!(s["layer_height"], "0.2");
        assert_eq!(s["nozzle_temperature"], json!(["235"]));
        assert_eq!(s["enable_prime_tower"], "1");
        assert_eq!(
            s["different_settings_to_system"],
            json!(["wall_loops", "", "layer_change_gcode"])
        );
        assert_eq!(
            s["inherits_group"],
            json!(["ParentProc", "", "Synthetic single nozzle"])
        );
        assert!(s.get("upward_compatible_machine").is_none());
    });
}

#[test]
fn export_3mf_replaces_only_machine_in_saved_projects() {
    for (bbl, preset, height) in [
        (true, "machine_same.json", "220"),
        (true, "machine_compat.json", "240"),
        (false, "machine_same.json", "220"),
        (false, "machine_compat.json", "240"),
    ] {
        verify_export(bbl, &["--load-settings", preset], |s| {
            assert_eq!(s["printable_height"], height);
            assert_eq!(s["layer_height"], "0.2");
            assert_eq!(s["nozzle_temperature"], json!(["200"]));
            assert_eq!(s["enable_prime_tower"], "1");
            assert_eq!(
                s["different_settings_to_system"],
                if bbl {
                    json!(["wall_loops", "nozzle_temperature", ""])
                } else {
                    json!(["wall_loops", ""])
                }
            );
        });
    }
}

#[test]
fn export_3mf_replaces_only_process_in_saved_projects() {
    for bbl in [true, false] {
        verify_export(bbl, &["--load-settings", "process_compat.json"], |s| {
            assert_eq!(s["printable_height"], "200");
            assert_eq!(s["layer_height"], "0.24");
            assert_eq!(s["nozzle_temperature"], json!(["200"]));
            assert_eq!(s["enable_prime_tower"], "1");
            assert_eq!(
                s["different_settings_to_system"],
                if bbl {
                    json!(["", "nozzle_temperature", "layer_change_gcode"])
                } else {
                    json!(["", "nozzle_temperature"])
                }
            );
        });
    }
}

#[test]
fn export_3mf_combines_single_setting_kind_with_filament_in_saved_projects() {
    for bbl in [true, false] {
        for (setting, height, layer) in [
            ("machine_same.json", "220", "0.2"),
            ("process_compat.json", "200", "0.24"),
        ] {
            verify_export(
                bbl,
                &[
                    "--load-settings",
                    setting,
                    "--load-filaments",
                    "filament_petg.json",
                ],
                |s| {
                    assert_eq!(s["printable_height"], height);
                    assert_eq!(s["layer_height"], layer);
                    assert_eq!(s["nozzle_temperature"], json!(["235"]));
                    assert_eq!(s["enable_prime_tower"], if bbl { "0" } else { "1" });
                },
            );
        }
    }
}

#[test]
fn export_3mf_passes_through_saved_projects_without_preset_flags() {
    for bbl in [true, false] {
        verify_export(bbl, &[], |s| {
            assert_eq!(s["printable_height"], "200");
            assert_eq!(s["layer_height"], "0.2");
            assert_eq!(s["nozzle_temperature"], json!(["200"]));
            assert_eq!(s["enable_prime_tower"], "1");
            assert_eq!(
                s["different_settings_to_system"],
                if bbl {
                    json!(["wall_loops", "nozzle_temperature", "layer_change_gcode"])
                } else {
                    json!(["wall_loops", "nozzle_temperature"])
                }
            );
        });
    }
}

#[test]
#[cfg(unix)]
fn export_3mf_rejects_incompatible_process_only_on_saved_bbl_project() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let work = runner.work_dir();
    prepare_saved_project(runner.bin(), work, true);
    let flags = ["--load-settings", "process_incompat.json"];
    let native = run_export(runner.bin(), work, &flags, "saved.3mf", "export.3mf");
    assert_eq!(native.status.code(), Some(239), "{work:?}");
    assert!(!work.join("export.3mf").exists(), "{work:?}");

    let ares = run_export(
        Path::new(env!("CARGO_BIN_EXE_ares")),
        work,
        &flags,
        "saved.3mf",
        "export.3mf",
    );
    assert_eq!(ares.status.code(), native.status.code(), "{work:?}");
    assert_eq!(ares.stdout, native.stdout, "{work:?}");
    assert_eq!(ares.stderr, native.stderr, "{work:?}");
    assert!(!work.join("export.3mf").exists(), "{work:?}");
}

fn verify_export(bbl: bool, flags: &[&str], check_settings: impl FnOnce(&Value)) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let work = runner.work_dir();
    prepare_saved_project(runner.bin(), work, bbl);

    let native = run_export(runner.bin(), work, flags, "saved.3mf", "export.3mf");
    assert!(
        native.status.success(),
        "Native failed in {work:?}: {}",
        String::from_utf8_lossy(&native.stderr)
    );
    fs::rename(work.join("export.3mf"), work.join("native.3mf")).unwrap();
    let expected = archive_members(&work.join("native.3mf"));
    let settings = expected
        .iter()
        .find(|(name, _)| name == "Metadata/project_settings.config")
        .unwrap();
    check_settings(&serde_json::from_slice(&settings.1).unwrap());
    assert!(
        !expected
            .iter()
            .any(|(name, _)| name == "Metadata/plate_1.json")
    );

    let ares = Path::new(env!("CARGO_BIN_EXE_ares"));
    let first = run_export(ares, work, flags, "saved.3mf", "export.3mf");
    assert!(
        first.status.success(),
        "Ares failed in {work:?}: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let actual = archive_members(&work.join("export.3mf"));
    assert_eq!(expected.len(), actual.len(), "{work:?}");
    for ((name, expected_bytes), (actual_name, actual_bytes)) in expected.iter().zip(actual) {
        assert_eq!(name, &actual_name, "{work:?}");
        assert!(
            expected_bytes == &actual_bytes,
            "{name} differs in {work:?}"
        );
    }
    fs::rename(work.join("export.3mf"), work.join("ares.3mf")).unwrap();
    assert!(
        run_export(ares, work, flags, "saved.3mf", "export.3mf")
            .status
            .success(),
        "{work:?}"
    );
    assert!(fs::read(work.join("ares.3mf")).unwrap() == fs::read(work.join("export.3mf")).unwrap());
}

fn prepare_saved_project(orca: &Path, work: &Path, bbl: bool) {
    const P1: &str = "Synthetic single nozzle";
    const P2: &str = "Synthetic single nozzle V2";
    fs::write(work.join("mesh.3mf"), synthetic_3mf()).unwrap();
    for (file, kind, mut body) in [
        (
            "m0.json",
            "machine",
            machine(P1, "200", "; START_M0", "G92 E0"),
        ),
        (
            "p0.json",
            "process",
            process("Synthetic walls", &[P1, P2], "0.2"),
        ),
        (
            "f0.json",
            "filament",
            filament("Synthetic PLA", "SYNTH_PLA", "200"),
        ),
        (
            "machine_same.json",
            "machine",
            machine(P1, "220", "; START_M1", "G92 E0 ; M1"),
        ),
        (
            "machine_compat.json",
            "machine",
            machine(P2, "240", "; START_M2", "G92 E0 ; M2"),
        ),
        (
            "process_compat.json",
            "process",
            process("Synthetic walls V2", &[P1], "0.24"),
        ),
        (
            "process_incompat.json",
            "process",
            process("Synthetic walls Incompat", &["Other Printer"], "0.28"),
        ),
        (
            "filament_petg.json",
            "filament",
            filament("Synthetic PETG", "SYNTH_PETG", "235"),
        ),
    ] {
        body["from"] = json!("system");
        body["type"] = json!(kind);
        body["instantiation"] = json!("true");
        fs::write(work.join(file), serde_json::to_vec(&body).unwrap()).unwrap();
    }
    let init = run_export(
        orca,
        work,
        &[
            "--load-settings",
            "m0.json;p0.json",
            "--load-filaments",
            "f0.json",
        ],
        "mesh.3mf",
        "initial.3mf",
    );
    assert!(init.status.success(), "initial export failed in {work:?}");
    let mut saved = ZipWriter::new(fs::File::create(work.join("saved.3mf")).unwrap());
    for (name, mut bytes) in archive_members(&work.join("initial.3mf")) {
        if !bbl && name == "3D/3dmodel.model" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .split_inclusive('\n')
                .filter(|l| {
                    !l.contains("name=\"Application\"") && !l.contains("name=\"OrcaSlicer\"")
                })
                .collect::<String>()
                .into_bytes();
        } else if name == "Metadata/project_settings.config" {
            let mut config: Value = serde_json::from_slice(&bytes).unwrap();
            config["enable_prime_tower"] = json!("1");
            config["different_settings_to_system"] =
                json!(["wall_loops", "nozzle_temperature", "layer_change_gcode"]);
            config["inherits_group"] = json!(["ParentProc", "ParentFila", P1]);
            bytes = serde_json::to_vec(&config).unwrap();
        }
        saved
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut saved, &bytes).unwrap();
    }
    saved.finish().unwrap();
}

fn run_export(
    bin: &Path,
    dir: &Path,
    flags: &[&str],
    input: &str,
    output: &str,
) -> std::process::Output {
    Command::new(bin)
        .current_dir(dir)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .args(["--arrange", "0", "--orient", "0"])
        .args(flags)
        .args(["--export-3mf", output, input])
        .output()
        .unwrap()
}

fn machine(name: &str, height: &str, start: &str, layer_change: &str) -> Value {
    json!({
        "name": name, "printer_technology": "FFF", "nozzle_diameter": ["0.4"],
        "printer_extruder_id": ["1"], "printer_extruder_variant": ["Direct Drive Standard"],
        "printable_area": ["0x0", "200x0", "200x200", "0x200"], "printable_height": height,
        "machine_start_gcode": start, "layer_change_gcode": layer_change
    })
}

fn process(name: &str, compatible: &[&str], layer_height: &str) -> Value {
    json!({
        "name": name, "compatible_printers": compatible, "initial_layer_print_height": "0.2",
        "layer_height": layer_height, "bottom_shell_layers": "0", "top_shell_layers": "0",
        "enable_prime_tower": "1", "sparse_infill_density": "0%"
    })
}

fn filament(name: &str, filament_id: &str, temperature: &str) -> Value {
    json!({
        "name": name, "filament_id": filament_id, "filament_diameter": ["1.75"],
        "filament_is_support": ["0"], "filament_extruder_variant": ["Direct Drive Standard"],
        "nozzle_temperature": [temperature]
    })
}
