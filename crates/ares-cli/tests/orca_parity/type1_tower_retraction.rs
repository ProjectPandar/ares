//! Print.cpp:5332–5333; GCode/WipeTower.cpp:2206–2207,4356–4389,5359–5382.
//! Compares the complete tower-writer retract/restoration sequence, not whole G-code.
use crate::runner::{OrcaRunner, application, native_api_cli::NativeCase};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const BASE: &[u8] = include_bytes!("../../../../tests/parity/bbl-type1-tower/enabled.3mf");

pub(super) fn project(overridden: bool) -> Vec<u8> {
    assert_eq!(
        crate::runner::native_api_cli::sha(BASE),
        "ab00b9040c3ccc1c10490a220880b87108aed7edd3a19926b8e7967f1287e506"
    );
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["retraction_length"] = json!(["0.8"]);
            settings["retraction_speed"] = json!(["30"]);
            settings["filament_retraction_length"] = if overridden {
                json!(["4", "4"])
            } else {
                json!(["nil", "nil"])
            };
            settings["filament_retraction_speed"] = if overridden {
                json!(["17", "17"])
            } else {
                json!(["nil", "nil"])
            };
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        } else if file.name() == "3D/3dmodel.model" {
            let mut model = String::from_utf8(bytes).unwrap();
            for y in [134, 122] {
                let before = format!("1 0 0 0 1 0 0 0 1 128 {y} 5");
                let after = format!("1 0 0 0 1 0 0 0 0.02 128 {y} 0.1");
                assert_eq!(model.matches(&before).count(), 1);
                model = model.replace(&before, &after);
            }
            bytes = model.into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

// Select every E-only G1 inside WIPE_TOWER_START/END, except the
// CP TOOLCHANGE START preamble before CP_TOOLCHANGE_WIPE: that preamble
// emits ordinary travel retraction and user templates, not tower paths.
// Keep both entry and gap-wall moves, in their emitted order, without
// filtering by expected extrusion length, speed, sign, or decimal format.
fn sequence(text: &str) -> Vec<&str> {
    let mut tower = false;
    let mut toolchange_preamble = false;
    let mut entry = false;
    let mut counts = [0_usize; 2];
    let mut result = Vec::new();
    for line in text.lines() {
        match line {
            "; CP TOOLCHANGE START" => toolchange_preamble = true,
            "; CP TOOLCHANGE END" => toolchange_preamble = false,
            "; WIPE_TOWER_START" => {
                assert!(!tower, "nested tower block");
                tower = true;
                entry = false;
            }
            "; WIPE_TOWER_END" => {
                assert!(tower, "unmatched tower end");
                tower = false;
            }
            _ if line.starts_with("; CP_TOOLCHANGE_WIPE ") => {
                toolchange_preamble = false;
                entry = true;
            }
            _ => {}
        }
        if tower && !toolchange_preamble && line.starts_with("G1 E") {
            counts[usize::from(entry)] += 1;
            result.push(line);
        }
    }
    assert!(!tower, "unterminated tower block");
    assert!(
        counts.iter().all(|&count| count > 0),
        "entry and wall must both retract"
    );
    assert_eq!(result.len() % 2, 0, "paired tower retractions");
    for pair in result.chunks(2) {
        let negative = pair[0].strip_prefix("G1 E-").unwrap();
        assert_eq!(
            pair[1].strip_prefix("G1 E"),
            Some(negative),
            "exact restoration"
        );
    }
    result
}

fn check(overridden: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = project(overridden);
    let case = NativeCase::slice(&runner, &input, "type1-tower-retraction-");
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let api = String::from_utf8(api).unwrap();
    let native = sequence(&case.native);
    let actual = sequence(&api);
    assert!(!native.is_empty(), "Native must emit tower retractions");
    assert_eq!(native.len(), actual.len(), "complete selected line count");
    let retract = if overridden {
        "G1 E-4.0000 F1020"
    } else {
        "G1 E-0.8000 F1800"
    };
    assert!(native.chunks(2).all(|pair| pair[0] == retract));
    fs::write(
        case.root.join("sequences.json"),
        serde_json::to_vec_pretty(&json!({
            "native": native, "api": actual, "whole_stream_equality_claimed": false,
            "input_sha256": crate::runner::native_api_cli::sha(&input),
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        native, actual,
        "complete tower retract/restoration sequence at {:?}",
        case.root
    );
}

#[test]
fn type1_tower_retract_unretract_sequence_uses_filament_overrides() {
    check(true);
}

#[test]
fn type1_tower_retract_unretract_sequence_inherits_machine_when_nil() {
    check(false);
}
