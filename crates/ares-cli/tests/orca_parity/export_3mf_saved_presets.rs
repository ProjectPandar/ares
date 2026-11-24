use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};
use zip::{ZipWriter, write::SimpleFileOptions};

use super::export_3mf::{archive_members, synthetic_3mf};
use crate::OrcaRunner;

#[test]
fn export_3mf_replaces_presets_in_saved_bbl_project() {
    export_with_explicit_presets(true);
}

#[test]
fn export_3mf_replaces_presets_in_saved_non_bbl_project() {
    export_with_explicit_presets(false);
}

fn export_with_explicit_presets(bbl: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let work = runner.work_dir();
    let ares = Path::new(env!("CARGO_BIN_EXE_ares"));
    fs::write(work.join("mesh.3mf"), synthetic_3mf()).unwrap();
    write_presets(work, "0.2", "200");
    export(runner.bin(), work, "mesh.3mf", "initial.3mf");
    let mut saved = ZipWriter::new(fs::File::create(work.join("saved.3mf")).unwrap());
    for (name, mut bytes) in archive_members(&work.join("initial.3mf")) {
        if !bbl && name == "3D/3dmodel.model" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .split_inclusive('\n')
                .filter(|line| {
                    !line.contains("name=\"Application\"") && !line.contains("name=\"OrcaSlicer\"")
                })
                .collect::<String>()
                .into_bytes();
        }
        saved
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut saved, &bytes).unwrap();
    }
    saved.finish().unwrap();

    write_presets(work, "0.24", "216");
    export(runner.bin(), work, "saved.3mf", "export.3mf");
    fs::rename(work.join("export.3mf"), work.join("native.3mf")).unwrap();
    let expected = archive_members(&work.join("native.3mf"));
    let settings = expected
        .iter()
        .find(|(name, _)| name == "Metadata/project_settings.config")
        .unwrap();
    let settings: Value = serde_json::from_slice(&settings.1).unwrap();
    assert_eq!(settings["layer_height"], "0.24");
    assert_eq!(settings["nozzle_temperature"], json!(["216"]));
    assert!(
        !expected
            .iter()
            .any(|(name, _)| name == "Metadata/plate_1.json")
    );

    export(ares, work, "saved.3mf", "export.3mf");
    let actual = archive_members(&work.join("export.3mf"));
    assert_eq!(expected.len(), actual.len(), "{work:?}");
    // There is only one split model member, so relative object order cannot vary.
    for ((name, expected_bytes), (actual_name, actual_bytes)) in expected.iter().zip(actual) {
        assert_eq!(name, &actual_name, "{work:?}");
        assert!(
            expected_bytes == &actual_bytes,
            "{name} differs in {work:?}"
        );
    }
    fs::rename(work.join("export.3mf"), work.join("ares.3mf")).unwrap();
    export(ares, work, "saved.3mf", "export.3mf");
    assert!(fs::read(work.join("ares.3mf")).unwrap() == fs::read(work.join("export.3mf")).unwrap());
}

fn export(bin: &Path, dir: &Path, input: &str, output: &str) {
    let result = Command::new(bin)
        .current_dir(dir)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .args([
            "--arrange",
            "0",
            "--orient",
            "0",
            "--load-settings",
            "machine.json;process.json",
            "--load-filaments",
            "filament.json",
            "--export-3mf",
            output,
            input,
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{bin:?} failed in {dir:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn write_presets(dir: &Path, height: &str, temperature: &str) {
    for (kind, mut preset) in [
        (
            "machine",
            json!({
                "name": "Synthetic single nozzle", "printer_technology": "FFF",
                "nozzle_diameter": ["0.4"], "printer_extruder_id": ["1"],
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printable_area": ["0x0", "200x0", "200x200", "0x200"],
                "printable_height": "200", "layer_change_gcode": "G92 E0"
            }),
        ),
        (
            "process",
            json!({
                "name": "Synthetic walls", "compatible_printers": ["Synthetic single nozzle"],
                "initial_layer_print_height": "0.2", "layer_height": height,
                "bottom_shell_layers": "0", "top_shell_layers": "0",
                "enable_prime_tower": "0", "sparse_infill_density": "0%"
            }),
        ),
        (
            "filament",
            json!({
                "name": "Synthetic PLA", "filament_id": "SYNTH_PLA", "filament_diameter": ["1.75"],
                "filament_is_support": ["0"], "filament_extruder_variant": ["Direct Drive Standard"],
                "nozzle_temperature": [temperature]
            }),
        ),
    ] {
        preset["from"] = json!("system");
        preset["type"] = json!(kind);
        preset["instantiation"] = json!("true");
        fs::write(
            dir.join(format!("{kind}.json")),
            serde_json::to_vec(&preset).unwrap(),
        )
        .unwrap();
    }
}
