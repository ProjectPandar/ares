//! `GCode.cpp:6689-6692, 6710`: a tower printing a smooth timelapse keeps the template at the
//! layer change, even on a printer the traditional inserts would otherwise take.

use std::{
    fs,
    io::{Cursor, Read, Write},
};

use serde_json::{Value, json};

use crate::runner::{OrcaRunner, native_api_cli::NativeCase, repo_root};

const MARKER: &str = "; SMOOTH_TOWER_TEMPLATE";

/// The tracked smooth timelapse tower project with a template and an on-bed tower.
fn project_with_template() -> Vec<u8> {
    let tracked = repo_root().join("tests/parity/validation/timelapse-unprintable-gcode.3mf");
    let tracked = fs::read(tracked).unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(tracked)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["time_lapse_gcode"] = json!(MARKER);
            settings["wipe_tower_y"] = json!(["150"]);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

#[test]
fn smooth_tower_template_renders_only_at_layer_change_like_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = NativeCase::slice(&runner, &project_with_template(), "smooth-tower-template-");
    assert_eq!(super::observation::gate_phase(&case.native), "direct");
    let lines = case.native.lines().collect::<Vec<_>>();
    let layers = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == ";LAYER_CHANGE")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert!(!layers.is_empty());
    assert_eq!(
        lines.iter().filter(|line| **line == MARKER).count(),
        layers.len(),
        "one render per layer"
    );
    case.compare(ares_core::GenerationMetadata::default());
}
