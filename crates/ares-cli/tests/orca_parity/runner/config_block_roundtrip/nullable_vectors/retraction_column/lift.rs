//! GCodeWriter.cpp:909-948 and GCode.cpp:10866: printing-filament lift settings.

use super::{repo_root, sha};
use crate::runner::native_api_cli::NativeCase;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

fn project(key: &str, value: &str, filament: usize) -> Vec<u8> {
    let source = fs::read(repo_root().join("tests/parity/semm-ramming/R1.3mf")).unwrap();
    let mut updates = json!({
        "enable_prime_tower":"0", "enable_filament_ramming":"0",
        "filament_self_index":["1","2"],
        "filament_extruder_variant":["Direct Drive Standard","Direct Drive Standard"],
        "z_hop":["0.6"], "filament_z_hop":["nil","nil"],
        "z_hop_types":["Normal Lift"], "filament_z_hop_types":["nil","nil"],
        "retract_lift_above":["0"], "retract_lift_below":["0"],
        "retract_lift_enforce":["All Surfaces"],
        "filament_retract_lift_above":["nil","nil"],
        "filament_retract_lift_below":["nil","nil"],
        "filament_retract_lift_enforce":["nil","nil"],
        "layer_height":"0.2", "initial_layer_print_height":"0.2",
        "machine_end_gcode":"", "before_layer_change_gcode":"G92 E0",
        "sparse_infill_filament_id":"0", "internal_solid_filament_id":"0",
        "top_surface_filament_id":"0", "bottom_surface_filament_id":"0"
    });
    updates[key] = json!(["nil", value]);
    let settings = super::super::input::settings_variant(&source, updates.as_object().unwrap());
    let mut archive = zip::ZipArchive::new(Cursor::new(&settings)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).unwrap();
            for previous in [1, 2] {
                xml = xml.replace(
                    &format!("key=\"extruder\" value=\"{previous}\""),
                    &format!("key=\"extruder\" value=\"{filament}\""),
                );
            }
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

fn lift_layers(gcode: &str) -> Vec<String> {
    let mut layer = None;
    let mut layers = Vec::new();
    for line in gcode.lines() {
        if let Some(z) = line.strip_prefix(";Z:") {
            layer = Some((z, z.parse::<f64>().unwrap()));
        }
        let Some((label, height)) = layer else {
            continue;
        };
        if (line.starts_with("G1 ") || line.starts_with("G0 "))
            && line
                .split(';')
                .next()
                .unwrap()
                .split_whitespace()
                .any(|word| {
                    word.strip_prefix('Z')
                        .is_some_and(|z| z.parse::<f64>().unwrap() > height + 0.3)
                })
            && layers.last().is_none_or(|previous| previous != label)
        {
            layers.push(label.to_owned());
        }
    }
    layers
}

fn check(key: &str, value: &str, filament: usize) {
    let Some(runner) = crate::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(key, value, filament);
    let label = format!("{key}-filament-{filament}");
    let reference = NativeCase::slice(&runner, &project, &format!("filament-lift-{label}-"));
    let replay = NativeCase::slice(&runner, &project, &format!("filament-lift-{label}-replay-"));
    let original = crate::golden::normalize_one_generator_line(
        reference.native.as_bytes(),
        crate::golden::GeneratorKind::Orca,
    )
    .unwrap();
    let repeated = crate::golden::normalize_one_generator_line(
        replay.native.as_bytes(),
        crate::golden::GeneratorKind::Orca,
    )
    .unwrap();
    let native_replay = crate::golden::first_difference(&original, &repeated)
        .map_or(Ok(()), |difference| Err(difference.to_string()));
    fs::write(
        reference.root.join("native-replay.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":sha(&project), "replay_root":replay.root,
            "native_replay_comparison":native_replay,
        }))
        .unwrap(),
    )
    .unwrap();
    native_replay.unwrap();
    for premise in [
        "; nozzle_diameter = 0.4",
        "; filament_self_index = 1,2",
        "; filament_map = 1,1",
    ] {
        assert!(reference.native.lines().any(|line| line == premise));
    }
    let native_layers = lift_layers(&reference.native);
    let first = native_layers.first().unwrap().parse::<f64>().unwrap();
    let last = native_layers.last().unwrap().parse::<f64>().unwrap();
    if filament == 1 {
        assert!(first <= 0.2 && last > 1.0);
    } else {
        match key {
            "filament_retract_lift_above" => assert!(first >= 1.0 && last > 1.0),
            // A layer-change lift is scheduled at the preceding writer height.
            "filament_retract_lift_below" => assert!(first <= 0.2 && last <= 1.2),
            "filament_retract_lift_enforce" => assert!(first <= 0.2 && last <= 0.2),
            _ => unreachable!(),
        }
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api_layers = lift_layers(std::str::from_utf8(&api).unwrap());
    fs::write(
        reference.root.join("lift-layer-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label, "input_sha256":sha(&project),
            "native_sha256":sha(reference.native.as_bytes()), "api_sha256":sha(&api),
            "native_lift_layers":native_layers, "api_lift_layers":api_layers,
            "strict_whole_output_comparison":crate::golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(), &api,
            ),
            "whole_output_parity_claimed":false,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(api_layers, native_layers, "{label}; {:?}", reference.root);
}

#[test]
fn first_filament_keeps_default_lift_height() {
    check("filament_retract_lift_above", "1", 1);
}

#[test]
fn second_filament_uses_its_minimum_lift_height() {
    check("filament_retract_lift_above", "1", 2);
}

#[test]
fn second_filament_uses_its_maximum_lift_height() {
    check("filament_retract_lift_below", "1", 2);
}

#[test]
fn first_filament_keeps_all_surface_lifts() {
    check("filament_retract_lift_enforce", "Bottom Only", 1);
}

#[test]
fn second_filament_uses_its_bottom_only_lifts() {
    check("filament_retract_lift_enforce", "Bottom Only", 2);
}
