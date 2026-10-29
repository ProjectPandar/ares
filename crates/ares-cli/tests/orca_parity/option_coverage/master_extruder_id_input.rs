//! ToolOrdering.cpp:1150-1207; FilamentGroup.cpp:40-49: TPU selects master.
use super::*;
use crate::runner::ExportedCase;
use std::io::{Cursor, Read, Write};

pub(super) fn project(runner: &OrcaRunner, label: &str, master: &str, mode: &str) -> ExportedCase {
    let mut exported =
        tool_context::mapping(runner, label, &serde_json::json!(["1", "2"])).unwrap();
    let fields = serde_json::json!({
        "master_extruder_id": master,
        "filament_map_mode": mode,
        "filament_type": ["TPU", "PLA"],
        "filament_printable": ["3", "3"]
    });
    let mut reader = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings
                .as_object_mut()
                .unwrap()
                .extend(fields.as_object().unwrap().clone());
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let project = writer.finish().unwrap().into_inner();
    let before = crate::runner::application::settings(&exported.project).unwrap();
    let mut after = crate::runner::application::settings(&project).unwrap();
    for (key, value) in fields.as_object().unwrap() {
        assert_eq!(after[key], *value, "{label}: {key}");
        match before.get(key) {
            Some(original) => {
                after.insert(key.clone(), original.clone());
            }
            None => {
                after.remove(key);
            }
        }
    }
    assert_eq!(after, before, "{label}: undeclared settings changed");
    let mut patched = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    assert_eq!(reader.len(), patched.len());
    for index in 0..reader.len() {
        let mut original = reader.by_index(index).unwrap();
        if original.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut actual = patched.by_name(original.name()).unwrap();
        let mut a = Vec::new();
        let mut b = Vec::new();
        original.read_to_end(&mut a).unwrap();
        actual.read_to_end(&mut b).unwrap();
        assert_eq!(a, b, "{label}: {}", original.name());
    }
    exported.project = project;
    let result = runner
        .bind_filament_map_mode(exported, &serde_json::json!(mode))
        .unwrap();
    for (key, value) in fields.as_object().unwrap() {
        crate::runner::application::verify_value(&result.project, key, value).unwrap();
    }
    result
}
