//! Section oracle for WipeTower.cpp:719–722 and both wipe-tower writers.
//! The original power-loss whole-output assertion remains a separate strict gate.

use std::{
    fs,
    io::{Cursor, Read, Write},
};

use serde_json::{Value, json};

use super::{EMPTY_KLIPPER, slice_native};
use crate::runner::OrcaRunner;

fn marlin_control() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(EMPTY_KLIPPER)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["gcode_flavor"] = json!("marlin");
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

fn tower_flushes(gcode: &str) -> Vec<&str> {
    let mut inside = false;
    let mut flushes = Vec::new();
    for line in gcode.lines() {
        match line {
            "; CP TOOLCHANGE START" | "; CP PRIMING START" => inside = true,
            "; CP TOOLCHANGE END" | "; CP PRIMING END" => inside = false,
            "M400" | "G4 S0" if inside => flushes.push(line),
            _ => {}
        }
    }
    flushes
}

fn check(project: Vec<u8>, flavor: &str, command: &str) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native slicing oracle");
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("wipe-tower-queue-flush-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    eprintln!("{flavor}: artifacts {directory:?}");
    fs::write(directory.join("input.3mf"), &project).unwrap();
    let native = slice_native(&runner, &directory, &project, "native-");
    fs::write(directory.join("orca.gcode"), &native).unwrap();
    let processed = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_processed(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &processed.gcode).unwrap();
    let ares = std::str::from_utf8(&processed.gcode).unwrap();
    let native_flushes = tower_flushes(&native);
    let ares_flushes = tower_flushes(ares);
    let whole_output =
        crate::golden::compare_ordered_bytes_generator_only(native.as_bytes(), &processed.gcode);
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "scope": "CP priming/tool-change queue-flush commands only",
            "source": "WipeTower.cpp:719-722",
            "flavor": flavor,
            "native_flushes": native_flushes,
            "ares_flushes": ares_flushes,
            "whole_output_comparison_not_waived": whole_output,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(native.contains(&format!("; gcode_flavor = {flavor}\n")));
    assert!(
        !native_flushes.is_empty(),
        "Native must exercise tower flushes"
    );
    assert!(native_flushes.iter().all(|line| *line == command));
    assert_eq!(ares_flushes, native_flushes, "{flavor}: {directory:?}");
}

#[test]
fn klipper_wipe_tower_queue_flush_matches_native() {
    check(EMPTY_KLIPPER.to_vec(), "klipper", "M400");
}

#[test]
fn marlin_wipe_tower_queue_flush_keeps_g4_s0() {
    check(marlin_control(), "marlin", "G4 S0");
}
