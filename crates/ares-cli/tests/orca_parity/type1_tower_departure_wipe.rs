//! Print.cpp:5333; WipeTower.cpp:2208,3985,5296,1355–1365; GCode.cpp:1652–1657.
//! Complete first tower departure wipe block parity, not whole-stream parity.
use crate::runner::{OrcaRunner, application, native_api_cli::NativeCase};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

fn project(overridden: bool) -> Vec<u8> {
    let base = super::type1_tower_retraction::project(false);
    let mut reader = zip::ZipArchive::new(Cursor::new(base)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        for local in ["/home/", "/tmp/", "file://", "C:\\", "D:\\"] {
            assert!(!text.contains(local), "local path in {}", file.name());
        }
        if file.name() == "3D/3dmodel.model" {
            assert!(text.contains("<metadata name=\"CreationDate\"></metadata>"));
            assert!(text.contains("<metadata name=\"ModificationDate\"></metadata>"));
        }
        if file.name() == "Metadata/model_settings.config" {
            assert!(text.contains("key=\"source_file\" value=\"cube0.stl\""));
            assert!(text.contains("key=\"source_file\" value=\"cube1.stl\""));
        }
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["timelapse_type"] = json!("1");
            settings["wipe_distance"] = if overridden {
                json!(["0", "0"])
            } else {
                json!(["5", "5"])
            };
            settings["filament_wipe_distance"] = if overridden {
                json!(["5", "5"])
            } else {
                json!(["nil", "nil"])
            };
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default()
                    .last_modified_time(zip::DateTime::default())
                    .compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

// Select the first tower result's entire departure wipe, bounded by its end
// and the next ordinary XY travel. Never select an object wipe encountered
// later, and never filter moves by coordinates, extrusion, feedrate or count.
fn departure(text: &str) -> Vec<&str> {
    let lines: Vec<_> = text.lines().collect();
    let tower = lines
        .iter()
        .position(|&line| line == "; WIPE_TOWER_START")
        .unwrap();
    let end = tower
        + lines[tower..]
            .iter()
            .position(|&line| line == "; WIPE_TOWER_END")
            .unwrap();
    let tail = &lines[end + 1..];
    let travel = tail
        .iter()
        .position(|line| {
            (line.starts_with("G0 ") || line.starts_with("G1 "))
                && (line.contains(" X") || line.contains(" Y"))
                && !line.contains('E')
        })
        .unwrap();
    let Some(start) = tail[..travel]
        .iter()
        .position(|&line| line == "; WIPE_START")
    else {
        return Vec::new();
    };
    let stop = start
        + tail[start..]
            .iter()
            .position(|&line| line == "; WIPE_END")
            .unwrap();
    tail[start..=stop].to_vec()
}

fn check(overridden: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = project(overridden);
    let case = NativeCase::slice(&runner, &input, "type1-tower-departure-wipe-");
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
    let native = departure(&case.native);
    let actual = departure(&api);
    fs::write(
        case.root.join("departure-wipe.json"),
        serde_json::to_vec_pretty(&json!({
            "native": native, "api": actual, "whole_stream_equality_claimed": false,
            "input_sha256": crate::runner::native_api_cli::sha(&input),
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(!native.is_empty(), "Native departure wipe must be nonempty");
    assert!(
        native
            .iter()
            .any(|line| line.starts_with("G1 ") && line.contains(" X") && line.contains('E'))
    );
    assert_eq!(
        native.len(),
        actual.len(),
        "complete departure wipe line count at {:?}",
        case.root
    );
    assert_eq!(
        native, actual,
        "complete departure wipe sequence at {:?}",
        case.root
    );
}

#[test]
fn type1_tower_departure_wipe_uses_filament_distance_override() {
    check(true);
}

#[test]
fn type1_tower_departure_wipe_inherits_machine_distance_when_nil() {
    check(false);
}
