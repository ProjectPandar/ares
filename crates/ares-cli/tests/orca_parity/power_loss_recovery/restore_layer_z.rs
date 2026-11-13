//! OrcaSlicer 39408fa1f, tests/fff_print/test_wipe_tower.cpp:659.
use std::{
    fs,
    io::{Cursor, Read, Write},
};

use super::{EMPTY_KLIPPER, slice_native};
use crate::runner::OrcaRunner;
use serde_json::{Value, json};

fn project(
    tower_type: &str,
    two_nozzles: bool,
    custom_macro: bool,
    high_nozzle_entry: bool,
) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(EMPTY_KLIPPER)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        match file.name() {
            "Metadata/project_settings.config" => {
                let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
                settings["top_surface_filament_id"] = json!("2");
                settings["enable_prime_tower"] = json!("1");
                settings["wipe_tower_type"] = json!(tower_type);
                settings["wipe_tower_no_sparse_layers"] = json!("1");
                settings["wipe_tower_x"] = json!(["30"]);
                settings["wipe_tower_y"] = json!(["30"]);
                settings["layer_height"] = json!("0.2");
                settings["initial_layer_print_height"] = json!("0.2");
                settings["change_filament_gcode"] = json!(if custom_macro {
                    "; TOOLCHANGE_MARK\n"
                } else {
                    ""
                });
                if high_nozzle_entry {
                    settings["filament_end_gcode"] = json!([
                        "G1 Z10\n{position[2] = 10}\n",
                        "G1 Z10\n{position[2] = 10}\n"
                    ]);
                }
                settings["gcode_comments"] = json!("1");
                settings["timelapse_type"] = json!("0");
                settings["enable_wrapping_detection"] = json!("0");
                settings["raft_layers"] = json!("0");
                settings["use_relative_e_distances"] = json!("1");
                settings["filament_map_mode"] = json!("Manual");
                if two_nozzles {
                    settings["single_extruder_multi_material"] = json!("0");
                    settings["nozzle_diameter"] = json!(["0.4", "0.4"]);
                    settings["printer_extruder_id"] = json!(["1", "2"]);
                    settings["printer_extruder_variant"] =
                        json!(["Direct Drive Standard", "Direct Drive Standard"]);
                    settings["extruder_printable_height"] = json!(["0", "0"]);
                    settings["filament_map"] = json!(["1", "2"]);
                    settings["flush_volumes_matrix"] =
                        json!(["0", "247", "529", "0", "0", "247", "529", "0"]);
                    settings["flush_multiplier"] = json!(["0.3", "0.3"]);
                }
                bytes = serde_json::to_vec_pretty(&settings).unwrap();
            }
            "3D/3dmodel.model" => {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace("110 116 5", "125 125 5")
                    .replace("110 104 5", "155 125 1")
                    .into_bytes();
            }
            "3D/Objects/cube10.stl_2.model" => {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace("z=\"-5\"", "z=\"-1\"")
                    .replace("z=\"5\"", "z=\"1\"")
                    .into_bytes();
            }
            "Metadata/model_settings.config" => {
                bytes = String::from_utf8(bytes)
                    .unwrap()
                    .replace(
                        "key=\"extruder\" value=\"2\"",
                        "key=\"extruder\" value=\"1\"",
                    )
                    .replace("Auto For Flush", "Manual")
                    .into_bytes();
            }
            _ => {}
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
struct Heights {
    changes: Vec<(f64, f64)>,
    tower_extrusions: Vec<(f64, f64)>,
    lifts: usize,
    maximum_object_z: f64,
    nozzle_entry_descents: usize,
}

fn heights(gcode: &str) -> Heights {
    let mut result = Heights {
        changes: Vec::new(),
        tower_extrusions: Vec::new(),
        lifts: 0,
        maximum_object_z: 0.0,
        nozzle_entry_descents: 0,
    };
    let mut z = 0.0;
    let mut in_tower = false;
    let mut compacted_descend = false;
    for line in gcode.lines() {
        if line.contains("Travel to a Wipe Tower")
            || line.contains("Move to nozzle change start pos")
        {
            compacted_descend = false;
        }
        if line.contains("Descend to compacted wipe tower z") {
            compacted_descend = true;
        }
        if (line.starts_with("; NOZZLE_CHANGE_START") || line.starts_with("; Nozzle change start"))
            && compacted_descend
        {
            result.nozzle_entry_descents += 1;
        }
        if line.starts_with(";TYPE:") || line.starts_with("; FEATURE: ") {
            in_tower = line.contains("Prime tower");
        }
        let mut words = line.split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        let is_move = matches!(command, "G0" | "G1");
        let mut xy = false;
        let mut extrusion = 0.0;
        for word in words.take_while(|word| is_move && !word.starts_with(';')) {
            match word.as_bytes()[0] {
                b'Z' => z = word[1..].parse().unwrap(),
                b'X' | b'Y' => xy = true,
                b'E' => extrusion = word[1..].parse::<f64>().unwrap(),
                _ => {}
            }
        }
        if extrusion > 0.0 && xy {
            if !in_tower {
                result.maximum_object_z = result.maximum_object_z.max(z);
            } else if result.maximum_object_z > 2.0 + 1e-4 && z <= 10.0 + 1e-4 {
                result.tower_extrusions.push((z, result.maximum_object_z));
            }
        }
        if line.contains("; TOOLCHANGE_MARK")
            || command.starts_with('T') && command.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
        {
            result.changes.push((z, result.maximum_object_z));
        }
        if line.contains("Restore layer Z before toolchange") {
            result.lifts += 1;
        }
    }
    result
}

fn assert_heights(heights: &Heights, two_nozzles: bool, custom_macro: bool) {
    assert!(heights.maximum_object_z >= 10.0 - 1e-4, "{heights:?}");
    if !custom_macro && !two_nozzles {
        assert_eq!(heights.lifts, 0, "{heights:?}");
    } else {
        let above_short: Vec<_> = heights
            .changes
            .iter()
            .filter(|(_, printed)| *printed > 2.0 + 1e-4)
            .collect();
        assert!(!above_short.is_empty(), "{heights:?}");
        for (change_z, printed_z) in above_short {
            assert!(change_z >= &(printed_z - 1e-4), "{heights:?}");
        }
    }
    assert!(!heights.tower_extrusions.is_empty(), "{heights:?}");
    for (tower_z, printed_z) in &heights.tower_extrusions {
        assert!(tower_z < &(printed_z - 1e-4), "{heights:?}");
    }
}

fn check(tower_type: &str, two_nozzles: bool, custom_macro: bool, high_nozzle_entry: bool) {
    let runner = OrcaRunner::from_env()
        .unwrap()
        .expect("Native slicing oracle");
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("tower-restore-layer-z-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    eprintln!("artifacts {directory:?}");
    let input = project(tower_type, two_nozzles, custom_macro, high_nozzle_entry);
    fs::write(directory.join("input.3mf"), &input).unwrap();
    let native = slice_native(&runner, &directory, &input, "native-");
    fs::write(directory.join("native.gcode"), &native).unwrap();
    let native_heights = heights(&native);
    fs::write(
        directory.join("native-heights.json"),
        serde_json::to_vec_pretty(&native_heights).unwrap(),
    )
    .unwrap();
    assert_heights(&native_heights, two_nozzles, custom_macro);
    if high_nozzle_entry {
        assert!(native_heights.nozzle_entry_descents > 0);
    }
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
    let ares_heights = heights(std::str::from_utf8(&processed.gcode).unwrap());
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
        "source": "39408fa1f; tests/fff_print/test_wipe_tower.cpp:659",
        "tower_type": tower_type, "two_nozzles": two_nozzles, "custom_macro": custom_macro,
        "high_nozzle_entry": high_nozzle_entry,
        "native_heights": native_heights, "ares_heights": ares_heights,
        "whole_output_comparison_not_waived": crate::golden::compare_ordered_bytes_generator_only(native.as_bytes(), &processed.gcode)
    })).unwrap()).unwrap();
    assert_heights(&ares_heights, two_nozzles, custom_macro);
    if high_nozzle_entry {
        assert!(ares_heights.nozzle_entry_descents > 0);
    }
}

#[test]
fn type1_single_nozzle_custom_macro_starts_at_object_z() {
    check("type1", false, true, false);
}
#[test]
fn type1_single_nozzle_empty_macro_stays_at_tower_z() {
    check("type1", false, false, false);
}
#[test]
fn type1_two_nozzles_custom_macro_starts_at_object_z() {
    check("type1", true, true, false);
}
#[test]
fn type1_two_nozzles_empty_macro_starts_at_object_z() {
    check("type1", true, false, false);
}
#[test]
fn type2_single_nozzle_custom_macro_starts_at_object_z() {
    check("type2", false, true, false);
}
#[test]
fn type2_single_nozzle_empty_macro_stays_at_tower_z() {
    check("type2", false, false, false);
}
#[test]
fn type2_two_nozzles_custom_macro_starts_at_object_z() {
    check("type2", true, true, false);
}
#[test]
fn type2_two_nozzles_empty_macro_starts_at_object_z() {
    check("type2", true, false, false);
}

#[test]
fn type1_two_nozzles_high_entry_descends_before_nozzle_wipe() {
    check("type1", true, true, true);
}
