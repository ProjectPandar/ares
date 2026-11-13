//! `GCode.cpp:1164–1178`, `GCode/WipeTower.cpp:53–57`: tower entry eligibility.
use crate::runner::OrcaRunner;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

fn project(tower_type: &str, setting: Option<(&str, &str)>, wrapping_region: bool) -> Vec<u8> {
    let input = super::project(tower_type, false, true, false);
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            let end = if wrapping_region {
                settings["wrapping_exclude_area"] = json!(["5x5", "15x5", "15x15", "5x15"]);
                "; END_FILAMENT_AT {position[2]}\n; END_FILAMENT_ID {filament_extruder_id}\n"
            } else {
                "; END_FILAMENT_AT {position[2]}\n"
            };
            settings["filament_end_gcode"] = json!([end, end]);
            if let Some((key, value)) = setting {
                settings[key] = json!(value);
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

#[derive(Debug, serde::Serialize)]
struct Entry {
    printed_z: f64,
    end_filament_z: f64,
    travelled_to_tower: bool,
}

fn entries(gcode: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut z: f64 = 0.0;
    let mut printed_z: f64 = 0.0;
    let mut in_tower = false;
    let mut travelled_to_tower = false;
    for line in gcode.lines() {
        if line.starts_with(";TYPE:") || line.starts_with("; FEATURE: ") {
            in_tower = line.contains("Prime tower");
        }
        if line.contains("Travel to a Wipe Tower") {
            travelled_to_tower = true;
        }
        if let Some(value) = line.strip_prefix("; END_FILAMENT_AT ") {
            entries.push(Entry {
                printed_z,
                end_filament_z: value.parse().unwrap(),
                travelled_to_tower,
            });
        }
        let mut words = line.split_ascii_whitespace();
        if !matches!(words.next(), Some("G0" | "G1")) {
            continue;
        }
        let mut xy = false;
        let mut extrusion = 0.0;
        for word in words.take_while(|word| !word.starts_with(';')) {
            match word.as_bytes()[0] {
                b'Z' => z = word[1..].parse().unwrap(),
                b'X' | b'Y' => xy = true,
                b'E' => extrusion = word[1..].parse::<f64>().unwrap(),
                _ => {}
            }
        }
        if !in_tower && xy && extrusion > 0.0 {
            printed_z = printed_z.max(z);
            travelled_to_tower = false;
        }
    }
    entries
}

fn assert_entry(gcode: &str, sparse_skipped: bool) {
    let heights = super::heights(gcode);
    assert!(heights.maximum_object_z >= 10.0 - 1e-4, "{heights:?}");
    if sparse_skipped {
        let entries = entries(gcode);
        // append_tcr travels before the end macro only for is_finish_first.
        let from_object: Vec<_> = entries
            .iter()
            .filter(|entry| entry.printed_z > 2.0 + 1e-4 && !entry.travelled_to_tower)
            .collect();
        assert!(!from_object.is_empty(), "{entries:?}");
        for entry in from_object {
            assert!(entry.end_filament_z >= entry.printed_z - 1e-4, "{entry:?}");
        }
    } else {
        assert_eq!(heights.lifts, 0, "{heights:?}");
        assert!(!heights.tower_extrusions.is_empty(), "{heights:?}");
        for (tower_z, printed_z) in heights.tower_extrusions {
            assert!(
                tower_z >= printed_z - 1e-4,
                "tower {tower_z}, object {printed_z}"
            );
        }
    }
}

fn check(tower_type: &str, setting: Option<(&str, &str)>, wrapping_region: bool) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native slicing oracle");
    let directory = tempfile::Builder::new()
        .prefix("tower-entry-eligibility-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    eprintln!("artifacts {directory:?}");
    let input = project(tower_type, setting, wrapping_region);
    fs::write(directory.join("input.3mf"), &input).unwrap();
    let native = super::slice_native(&runner, &directory, &input, "native-");
    fs::write(directory.join("native.gcode"), &native).unwrap();
    fs::write(
        directory.join("native-entries.json"),
        serde_json::to_vec_pretty(&entries(&native)).unwrap(),
    )
    .unwrap();
    if wrapping_region {
        for marker in ["; END_FILAMENT_ID 0", "; END_FILAMENT_ID 1"] {
            assert!(native.lines().any(|line| line == marker), "{marker}");
        }
    }
    assert_entry(&native, setting.is_none());
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
    let ares = std::str::from_utf8(&processed.gcode).unwrap();
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "source": "GCode.cpp:1164-1178; GCode/WipeTower.cpp:53-57", "tower_type": tower_type, "setting": setting, "wrapping_region": wrapping_region,
        "native_entries": entries(&native), "ares_entries": entries(ares),
        "native_heights": super::heights(&native), "ares_heights": super::heights(ares),
        "whole_output_comparison_not_waived": crate::golden::compare_ordered_bytes_generator_only(native.as_bytes(), &processed.gcode)
    })).unwrap()).unwrap();
    assert_entry(ares, setting.is_none());
}

#[test]
fn type1_non_finish_first_keeps_object_z_until_tower_travel() {
    check("type1", None, false);
}
#[test]
fn type1_smooth_timelapse_disables_sparse_compaction() {
    check("type1", Some(("timelapse_type", "1")), false);
}
#[test]
fn type1_wrapping_detection_disables_sparse_compaction() {
    check("type1", Some(("enable_wrapping_detection", "1")), false);
}
#[test]
fn type2_smooth_timelapse_disables_sparse_compaction() {
    check("type2", Some(("timelapse_type", "1")), false);
}
#[test]
fn type2_wrapping_detection_disables_sparse_compaction() {
    check("type2", Some(("enable_wrapping_detection", "1")), false);
}
#[test]
fn type1_wrapping_region_with_two_used_filaments_omits_extra_wall() {
    check("type1", Some(("enable_wrapping_detection", "1")), true);
}
