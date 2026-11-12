//! `TimelapsePosPicker.cpp:38–39,515–524,155–159`: current-layer height context.

use super::{COMPLETE, input};
use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const HEIGHT_TEMPLATE: &str = "; PICKER_HEIGHT_SAFE {if has_timelapse_safe_pos}1{else}0{endif}\n";

fn height_context_project() -> (Vec<u8>, serde_json::Value) {
    let project = input::project(COMPLETE, &["0", "1"]);
    let parent_sha = sha(&project);
    let mut reader = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut provenance = None;
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let before: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let mut settings = before.clone();
            settings["time_lapse_gcode"] = json!(HEIGHT_TEMPLATE);
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
            let mut restored = settings;
            restored["time_lapse_gcode"] = before["time_lapse_gcode"].clone();
            assert_eq!(restored, before);
            provenance = Some(json!({
                "kind":"synthetic repository-derived height-gap control",
                "source_fixture":"tests/parity/printer-physical-identity/P0.3mf",
                "source_fixture_sha256":sha(include_bytes!("../../../../../../tests/parity/printer-physical-identity/P0.3mf")),
                "parent_identity_project_sha256":parent_sha,
                "changed_archive_entry":"Metadata/project_settings.config",
                "changed_fields":["time_lapse_gcode"],
                "parent_template_sha256":sha(before["time_lapse_gcode"].as_str().unwrap().as_bytes()),
                "derived_template":HEIGHT_TEMPLATE,
                "farthest_producer_overridden":false
            }));
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    (writer.finish().unwrap().into_inner(), provenance.unwrap())
}

#[test]
fn current_layer_height_gap() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (project, provenance) = height_context_project();
    let case = NativeCase::slice(&runner, &project, "picker-height-gap-");
    fs::write(
        case.root.join("provenance.json"),
        serde_json::to_vec_pretty(&provenance).unwrap(),
    )
    .unwrap();
    let flags = case
        .native
        .lines()
        .filter_map(|line| line.strip_prefix("; PICKER_HEIGHT_SAFE "))
        .collect::<Vec<_>>();
    assert_eq!(flags.len(), 50, "{:?}", case.root);
    assert_eq!(flags[..24], ["0"; 24]);
    assert_eq!(flags[24..], ["1"; 26]);
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(api, cli, "{:?}: raw API/CLI bytes", case.root);
}
