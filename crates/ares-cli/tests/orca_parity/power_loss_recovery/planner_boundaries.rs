//! OrcaSlicer 34c8e78e, tests/fff_print/test_wipe_tower.cpp:198–289.
use std::{
    fs,
    io::{Cursor, Read, Write},
};

use super::{EMPTY_KLIPPER, slice_native};
use crate::runner::OrcaRunner;
use serde_json::{Value, json};

fn project(
    flavor: &str,
    tower_type: &str,
    interface: bool,
    cooldown: bool,
    monitoring: bool,
) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(EMPTY_KLIPPER)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["gcode_flavor"] = json!(flavor);
            settings["wipe_tower_type"] = json!(tower_type);
            settings["change_filament_gcode"] = json!("; TOOLCHANGE_MARK\n");
            settings["enable_tower_interface_features"] = json!(if interface { "1" } else { "0" });
            settings["enable_tower_interface_cooldown_during_tower"] =
                json!(if cooldown { "1" } else { "0" });
            settings["nozzle_temperature_range_high"] = json!(["250", "250"]);
            if monitoring {
                settings["printer_notes"] = json!("PRINTER_MODEL_MK4 MMU");
            }
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn regions(gcode: &str) -> String {
    let mut inside = false;
    let mut result = String::new();
    for line in gcode.lines() {
        if line == "; WIPE_TOWER_START" {
            inside = true;
        }
        if line == "; WIPE_TOWER_END" {
            inside = false;
        }
        if inside {
            result.push_str(line);
            result.push('\n');
        }
    }
    assert!(!result.is_empty(), "slice must exercise the tower");
    result
}

fn commands(tower: &str) -> Vec<&str> {
    tower
        .lines()
        .filter(|line| {
            *line == "M400"
                || *line == "G4 S0"
                || *line == "; TOOLCHANGE_MARK"
                || line.starts_with("M104 ")
                || line.starts_with("M109 ")
                || line.starts_with("M591 ")
        })
        .collect()
}

fn assert_klipper_order(tower: &str, unload: bool) {
    let mut synced = false;
    let mut previous = "";
    let mut changes = 0;
    for line in tower.lines() {
        if previous == "M400"
            && !["M104", "M109", "M591"]
                .iter()
                .any(|command| line.starts_with(command))
        {
            assert!(unload && !synced, "redundant boundary sync before {line}");
            synced = true;
        }
        if line == "; TOOLCHANGE_MARK" {
            assert_eq!(synced, unload, "unload must finish before the macro");
            synced = false;
            changes += 1;
        }
        if line.starts_with("M591 ") {
            assert_eq!(previous, "M400");
        }
        assert_ne!(line, "G4 S0");
        assert!(
            !line.starts_with("G1 Y"),
            "tower move bypassed coordinate processing"
        );
        previous = line;
    }
    assert!(changes > 0);
}

fn unsynced_cooldowns(tower: &str) -> usize {
    let mut previous = "";
    tower
        .lines()
        .filter(|line| {
            let cooldown = *line == "M104 S220" && previous != "M400";
            previous = line;
            cooldown
        })
        .count()
}

fn check(flavor: &str, tower_type: &str, interface: bool, cooldown: bool, monitoring: bool) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native slicing oracle");
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("tower-planner-boundaries-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("artifacts {directory:?}");
    let input = project(flavor, tower_type, interface, cooldown, monitoring);
    fs::write(directory.join("input.3mf"), &input).unwrap();
    let native = slice_native(&runner, &directory, &input, "native-");
    fs::write(directory.join("native.gcode"), &native).unwrap();
    let processed = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_processed(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &processed.gcode).unwrap();
    let native_tower = regions(&native);
    let ares_tower = regions(std::str::from_utf8(&processed.gcode).unwrap());
    let native_commands = commands(&native_tower);
    let ares_commands = commands(&ares_tower);
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "source": "34c8e78e; tests/fff_print/test_wipe_tower.cpp:198-289",
        "flavor": flavor, "tower_type": tower_type, "interface": interface,
        "cooldown_during_tower": cooldown, "monitoring": monitoring,
        "scope": "tower planner, temperature and monitoring command order",
        "native_commands": native_commands, "ares_commands": ares_commands,
        "native_unsynced_cooldowns": unsynced_cooldowns(&native_tower),
        "ares_unsynced_cooldowns": unsynced_cooldowns(&ares_tower),
        "whole_output_comparison_not_waived": crate::golden::compare_ordered_bytes_generator_only(native.as_bytes(), &processed.gcode)
    })).unwrap()).unwrap();
    if flavor == "klipper" {
        assert_klipper_order(&native_tower, tower_type == "type2");
        if interface {
            assert!(unsynced_cooldowns(&native_tower) > 0);
        }
        if monitoring {
            assert!(native_tower.contains("M400\nM591 "));
        }
    } else {
        assert!(native_commands.contains(&"G4 S0"));
    }
    assert_eq!(ares_commands, native_commands, "{directory:?}");
    if flavor == "klipper" {
        assert_klipper_order(&ares_tower, tower_type == "type2");
    }
    assert_eq!(
        unsynced_cooldowns(&ares_tower),
        unsynced_cooldowns(&native_tower),
        "{directory:?}"
    );
}

#[test]
fn klipper_type2_keeps_unload_and_temperature_sync() {
    check("klipper", "type2", false, false, false);
}
#[test]
fn klipper_type1_omits_boundary_sync() {
    check("klipper", "type1", false, false, false);
}
#[test]
fn klipper_interface_cooldown_after_tower_is_unsynced() {
    check("klipper", "type2", true, false, false);
}
#[test]
fn klipper_interface_cooldown_during_tower_is_unsynced() {
    check("klipper", "type2", true, true, false);
}
#[test]
fn klipper_monitoring_keeps_sync() {
    check("klipper", "type2", false, false, true);
}
#[test]
fn marlin_type2_keeps_boundary_sync() {
    check("marlin", "type2", false, false, false);
}
