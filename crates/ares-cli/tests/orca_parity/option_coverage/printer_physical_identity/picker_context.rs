//! `TimelapsePosPicker.cpp:38–39,515–524,155–159`: current-layer height context.

use super::{COMPLETE, input};
use crate::{OrcaRunner, runner::native_api_cli::NativeCase};
use std::io::{Cursor, Read, Write};

fn without_farthest_target() -> Vec<u8> {
    let project = input::project(COMPLETE, &["0", "1"]);
    let mut reader = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["farthest_point_timelapse"] = serde_json::json!("0");
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
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

#[test]
fn current_layer_height_gap() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = NativeCase::slice(&runner, &without_farthest_target(), "picker-height-gap-");
    let pictures = case
        .native
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("M9711 M0 E"))
        .collect::<Vec<_>>();
    assert_eq!(pictures.len(), 50, "{:?}", case.root);
    assert!(pictures[..24].iter().all(|line| !line.contains(" X")));
    assert!(pictures[24..].iter().all(|line| line.contains(" X")));
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(api, cli, "{:?}: raw API/CLI bytes", case.root);
}
