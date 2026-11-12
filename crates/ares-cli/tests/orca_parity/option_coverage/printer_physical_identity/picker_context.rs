//! `TimelapsePosPicker.cpp:38–39,155–159,406–414,515–552`: height and target context.

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

fn project_with_setting(key: &str, value: &str) -> (Vec<u8>, serde_json::Value) {
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
            settings[key] = json!(value);
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
            let derived_template = settings["time_lapse_gcode"].clone();
            let mut restored = settings;
            match before.get(key) {
                Some(value) => restored[key] = value.clone(),
                None => {
                    restored.as_object_mut().unwrap().remove(key);
                }
            }
            assert_eq!(restored, before);
            provenance = Some(json!({
                "kind":"synthetic repository-derived picker control",
                "source_fixture":"tests/parity/printer-physical-identity/P0.3mf",
                "source_fixture_sha256":sha(include_bytes!("../../../../../../tests/parity/printer-physical-identity/P0.3mf")),
                "parent_identity_project_sha256":parent_sha,
                "changed_archive_entry":"Metadata/project_settings.config",
                "changed_fields":[key],
                "parent_template_sha256":sha(before["time_lapse_gcode"].as_str().unwrap().as_bytes()),
                "derived_template":derived_template,
                "farthest_producer_overridden":key == "farthest_point_timelapse"
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
    let (project, provenance) = project_with_setting("time_lapse_gcode", HEIGHT_TEMPLATE);
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

#[test]
fn absent_target_retains_camera_score() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (project, provenance) = project_with_setting("farthest_point_timelapse", "0");
    let case = NativeCase::slice(&runner, &project, "picker-no-target-");
    fs::write(
        case.root.join("provenance.json"),
        serde_json::to_vec_pretty(&provenance).unwrap(),
    )
    .unwrap();
    let pictures = case
        .native
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("M9711 M0 E"))
        .collect::<Vec<_>>();
    assert_eq!(pictures.len(), 50, "{:?}", case.root);
    assert!(pictures.iter().all(|line| line.contains(" X")));
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}
