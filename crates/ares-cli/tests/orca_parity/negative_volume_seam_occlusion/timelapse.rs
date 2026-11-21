use std::io::{Cursor, Read, Write};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    OrcaRunner,
    runner::{application, native_api_cli::NativeCase},
};

const SNAPSHOT: &str = "; SKIPTYPE: timelapse";
const LAYER: &str = "; CHANGE_LAYER\n";

fn wall_zero() -> Vec<u8> {
    const SETTINGS: &str = "Metadata/model_settings.config";
    let mut archive = ZipArchive::new(Cursor::new(super::NEGATIVE_TARGET)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    let mut settings = String::new();
    archive
        .by_name(SETTINGS)
        .unwrap()
        .read_to_string(&mut settings)
        .unwrap();
    let settings = settings.replacen(
        "<object id=\"3\">",
        "<object id=\"3\">\n    <metadata key=\"wall_loops\" value=\"0\"/>",
        1,
    );
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == SETTINGS {
            output
                .start_file(SETTINGS, SimpleFileOptions::default())
                .unwrap();
            output.write_all(settings.as_bytes()).unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}

fn check(label: &str, placement: impl Fn(&str) -> bool) -> Option<(NativeCase, String)> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let project = wall_zero();
    let case = NativeCase::slice(&runner, &project, label);
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 11, 18, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let api = String::from_utf8(api).unwrap();
    assert!(placement(&case.native), "Native {label}: {:?}", case.root);
    assert!(placement(&api), "Ares API {label}: {:?}", case.root);
    Some((case, api))
}

fn deposits(line: &str) -> bool {
    let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
    matches!(words.next(), Some("G1" | "G2" | "G3"))
        && words.clone().any(|word| word.starts_with(['X', 'Y']))
        && words.any(|word| {
            word.strip_prefix('E')
                .and_then(|e| e.parse::<f64>().ok())
                .is_some_and(|e| e > 0.0)
        })
}

#[test]
fn layer_start_snapshot_without_photo_head_target() {
    let _ = check("timelapse-before-first-deposition-", |gcode| {
        let layer = gcode.split(LAYER).nth(1).unwrap();
        assert!(layer.starts_with("; Z_HEIGHT: 0.2\n"));
        let lines = layer.lines().collect::<Vec<_>>();
        let first_deposition = lines.iter().position(|line| deposits(line)).unwrap();
        let Some(snapshot) = lines.iter().position(|&line| line == SNAPSHOT) else {
            return false;
        };
        lines[..snapshot].contains(&"G1 Z.6 F30000") && snapshot < first_deposition
    });
}

#[test]
fn layer_end_snapshot_when_inline_misses_target() {
    let Some((case, api)) = check("timelapse-after-layer-deposition-", |gcode| {
        let layer = gcode.split(LAYER).nth(4).unwrap();
        assert!(layer.starts_with("; Z_HEIGHT: 0.8\n"));
        let lines = layer.lines().collect::<Vec<_>>();
        let last_deposition = lines.iter().rposition(|line| deposits(line)).unwrap();
        let Some(snapshot) = lines.iter().position(|&line| line == SNAPSHOT) else {
            return false;
        };
        if snapshot <= last_deposition {
            return false;
        }
        let tail = &lines[last_deposition + 1..snapshot];
        let lift = tail.iter().position(|line| {
            line.starts_with("G1 ") && line.split_ascii_whitespace().any(|word| word == "Z1.2")
        });
        let Some(lift) = lift else {
            return false;
        };
        tail[..lift].contains(&"; WIPE_START") && tail[lift + 1..].contains(&"M625")
    }) else {
        return;
    };
    let snapshots = |gcode: &str| {
        gcode
            .split(LAYER)
            .skip(1)
            .map(|layer| layer.lines().filter(|&line| line == SNAPSHOT).count())
            .collect::<Vec<_>>()
    };
    assert_eq!(snapshots(&case.native), vec![1; 50]);
    assert_eq!(snapshots(&api), snapshots(&case.native), "{:?}", case.root);
}
